use crate::objects::errors::IntoResult;
use crate::objects::file::File;
use crate::{GtkTestWindow, objects::errors::show_error_popup};

use adw::TimedAnimation;
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::{gettext, ngettext};
use gtk::gdk::RGBA;
use gtk::gio;
use image::*;
use log::*;
use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::sync::Arc;

type GenResult<T> = Result<T, Box<dyn std::error::Error>>;

enum FilenameProperty {
    FileName = 0,
    DefaultBottomImage,
    XScale,
    YScale,
    ZoomVal,
    MonochromeSelected,
    MonochromeThreshold,
    MonochromeRed,
    MonochromeGreen,
    MonochromeBlue,
    MonochromeInverted,
    DefaultMonochromeColor,
    Hash,
}

impl GtkTestWindow {
    pub fn store_top_image_in_cache(&self, file: &File) -> GenResult<()> {
        let imp = self.imp();
        if !imp.settings.boolean("store-top-in-cache") {
            debug!("Top cache is disabled");
            return Ok(());
        }
        if imp.settings.boolean("manual-bottom-image-selection") {
            debug!("Non-default bottom image");
            return Ok(());
        }
        if imp.temp_image_loaded.get() {
            debug!("Temporary bottom image loaded");
            return Ok(());
        }
        //create folder inside cache
        let cache_path = Self::get_cache_path().join("top_images");
        if !cache_path.exists() {
            debug!("Top icon cache dir does not yet exist, creating");
            fs::create_dir(&cache_path)?;
        }
        let file_name = file.hash;
        let mut file_path = cache_path.clone();
        file_path.push(file_name.to_string());
        debug!("File path: {:?}", file_path);
        debug!("File name: {:?}", file.filename);
        match file_path.exists() {
            true => {
                debug!("File already exists with name");
                return Ok(());
            }
            false => {
                debug!("File does not yet exist, creating");
                fs::File::create(&file_path)?;
            }
        };
        // Only if the orignal image path is present, and it has not been shrunk
        // Save the original, else save the generated dynamic image
        let new_file = gio::File::for_path(&file_path);
        let filestream = new_file.open_readwrite(gio::Cancellable::NONE)?;
        let test = filestream.output_stream();
        if let Some(original_file) = &file.files {
            if !file.dynamic_image_resized {
                info!("Saving original image to cache");
                let buffer = original_file.load_bytes(gio::Cancellable::NONE)?;
                test.write_bytes(&buffer.0, gio::Cancellable::NONE)?;
                return Ok(());
            }
        }
        info!("Saving dynamic image to cache");
        file.dynamic_image
            .save_with_format(file_path, ImageFormat::Jpeg)?;
        Ok(())
    }

    //This function regenerates icon, it replaces all images that were dragged and dropped with ones of the correct system accent color.
    pub async fn regenerate_icons(&self) -> GenResult<()> {
        let imp = self.imp();
        let id = *imp.regeneration_lock.borrow();
        let _iconic_busy = Arc::clone(&imp.app_busy);
        imp.toast_overlay
            .add_toast(adw::Toast::new(&gettext("Regenerating icons")));
        let data_path = self.get_data_path();
        let mut incompatible_files_n: u32 = 0;
        let compatible_files =
            self.find_regeneratable_icons(data_path, &mut incompatible_files_n)?;
        imp.regeneration_revealer.set_reveal_child(true);
        imp.regeneration_osd.set_fraction(0.0);
        imp.regeneration_osd_second.set_fraction(0.0);
        let files_n = compatible_files.len();
        let mut last_animation = None;
        let mut last_animation_second = None;
        let step_size = 1.0 / files_n as f64;
        let mut regeneration_errors = vec![];
        for file in compatible_files {
            if *imp.regeneration_lock.borrow() != id {
                error!("Stopping regeneration");
                break;
            }
            let path = &file.path();
            match self.regenerate_and_save_single_icon(file).await {
                Ok(_) => (),
                Err(error) => {
                    error!("Error while generating {:?}: {}", path, &error.to_string());
                    regeneration_errors.push(error)
                }
            }
            last_animation = Some(self.progress_animation(
                step_size,
                last_animation,
                imp.regeneration_osd.clone(),
            ));
            last_animation_second = Some(self.progress_animation(
                step_size,
                last_animation_second,
                imp.regeneration_osd_second.clone(),
            ));
        }
        if !regeneration_errors.is_empty() {
            show_error_popup(
                &self,
                &format!(
                    "{} {}",
                    regeneration_errors.len(),
                    &ngettext(
                        "file failed to regenerate\nview logs for more information",
                        "files failed to regenerate\nview logs for more information",
                        regeneration_errors.len() as u32
                    )
                ),
                true,
                None,
            );
        }
        imp.toast_overlay.add_toast(adw::Toast::new(&gettext(
            "Regeneration sucessful, restart nautilus",
        )));
        imp.regeneration_revealer.set_reveal_child(false);
        self.close_iconic_busy_popup();
        Ok(())
    }

    async fn regenerate_and_save_single_icon(&self, file: DirEntry) -> GenResult<()> {
        let file_name = file.file_name();
        let file_path = file.path();
        let file_name = file_name.to_str().into_result()?.to_string();
        let file_properties = file_name.split("-");
        let properties_list: Vec<&str> = file_properties.into_iter().collect();
        let current_accent_color = self.get_accent_color_and_show_dialog();
        let bottom_image_path = PathBuf::from(format!(
            "/app/share/folder_icon/folders/folder_{}.svg",
            &current_accent_color
        ));
        let hash = properties_list[12].split(".").nth(0).into_result()?;
        let mut top_image_path = Self::get_cache_path().join("top_images");
        top_image_path.push(hash);
        let top_image_file = gio::spawn_blocking(move || {
            File::from_path(top_image_path, 1024, 0).map_err(|err| err.to_string())
        })
        .await
        .unwrap()?
        .dynamic_image;
        let slider_values = self.get_properties(properties_list.clone())?;
        let top_image = self.create_top_image_for_generation(properties_list, top_image_file)?;
        let bottom_image_file = gio::spawn_blocking(move || {
            File::from_path(bottom_image_path, 1024, 0).map_err(|err| err.to_string())
        })
        .await
        .unwrap()?
        .dynamic_image;
        info!("Generating image");
        let generated_image = self
            .generate_image(
                bottom_image_file,
                top_image,
                imageops::FilterType::Gaussian,
                slider_values.0,
                slider_values.1,
                slider_values.2,
            )
            .await;
        info!("Saving image");
        match gio::spawn_blocking(move || {
            generated_image.save_with_format(file_path, ImageFormat::Png)
        })
        .await
        .unwrap()
        {
            Ok(_) => info!("Saving Succesful"),
            Err(x) => error!("Saving failed: {:?}", x),
        };
        Ok(())
    }

    fn find_regeneratable_icons(
        &self,
        dir: PathBuf,
        incompatible_files: &mut u32,
    ) -> GenResult<Vec<fs::DirEntry>> {
        let mut regeneratable: Vec<fs::DirEntry> = vec![];
        let files: fs::ReadDir = fs::read_dir(&dir)?;
        for file in files {
            *incompatible_files += 1;
            let current_file = file?;
            let file_name = current_file.file_name();
            debug!("File found: {:?}", file_name);
            let file_name_str = file_name.to_str().into_result()?.to_string();
            let file_properties: Vec<&str> = file_name_str.split("-").collect();
            if file_properties[FilenameProperty::FileName as usize] != "folder_new" {
                info!("File not supported for regeneration");
                continue;
            }
            if !(file_properties[FilenameProperty::DefaultBottomImage as usize].parse::<usize>()?
                != 0)
            {
                info!("Non-default image, not converting");
                continue;
            }
            let hash = file_properties[FilenameProperty::Hash as usize]
                .split(".")
                .nth(0)
                .into_result()?;
            let mut top_image_path = Self::get_cache_path().join("top_images");
            top_image_path.push(hash);

            if !top_image_path.exists() {
                warn!("Top image file not found");
                continue;
            }
            *incompatible_files -= 1;
            regeneratable.push(current_file);
        }
        Ok(regeneratable)
    }

    fn get_properties(&self, properties: Vec<&str>) -> GenResult<(f64, f64, f64)> {
        let x_scale: f64 = properties[FilenameProperty::XScale as usize].parse()?;
        let y_scale: f64 = properties[FilenameProperty::YScale as usize].parse()?;
        let size: f64 = properties[FilenameProperty::ZoomVal as usize].parse()?;
        Ok((x_scale, y_scale, size))
    }

    fn create_top_image_for_generation(
        &self,
        properties: Vec<&str>,
        top_image: DynamicImage,
    ) -> GenResult<DynamicImage> {
        let color = match properties[FilenameProperty::DefaultMonochromeColor as usize] {
            "false" => RGBA::new(
                properties[FilenameProperty::MonochromeRed as usize].parse()?,
                properties[FilenameProperty::MonochromeGreen as usize].parse()?,
                properties[FilenameProperty::MonochromeBlue as usize].parse()?,
                1.0,
            ),
            _ => self.current_accent_rgba()?,
        };
        match properties[FilenameProperty::MonochromeSelected as usize] {
            "1" => Ok(self.to_monochrome(
                top_image,
                properties[FilenameProperty::MonochromeThreshold as usize].parse()?,
                color,
                Some(
                    properties[FilenameProperty::MonochromeInverted as usize].parse::<usize>()?
                        != 0,
                ),
            )),
            _ => Ok(top_image),
        }
    }

    fn current_accent_rgba(&self) -> GenResult<RGBA> {
        let imp = self.imp();
        let accent_color = self.get_accent_color_and_show_dialog();
        Ok(imp
            .default_color
            .borrow()
            .get(&accent_color)
            .into_result()?
            .clone())
    }

    fn progress_animation(
        &self,
        step_size: f64,
        previous_animation: Option<TimedAnimation>,
        progress_bar: gtk::ProgressBar,
    ) -> TimedAnimation {
        let target = adw::PropertyAnimationTarget::new(&progress_bar, "fraction");
        let _ = previous_animation.is_some_and(|animation| {
            animation.skip();
            false
        });
        let animation = adw::TimedAnimation::builder()
            .target(&target)
            .widget(&progress_bar)
            .value_from(progress_bar.fraction())
            .value_to(progress_bar.fraction() + step_size)
            .duration(200)
            .easing(adw::Easing::EaseInOutCubic)
            .build();
        animation.play();
        animation
    }

    /* This function is used to create a string with all properties applied to the current image.
    This makes it possible to completely recreate the image if the top image is still available
    */
    pub fn create_image_properties_string(&self) -> String {
        let imp = self.imp();
        let is_default = (!imp.settings.boolean("manual-bottom-image-selection")
            && imp.settings.string("selected-accent-color").as_str() == "None"
            && !imp.temp_image_loaded.get()) as u8;
        let x_scale_val = imp.x_scale.value();
        let y_scale_val = imp.y_scale.value();
        let zoom_val = imp.size.value();
        let is_monochrome = imp.monochrome_switch.is_active() as u8;
        let monochrome_slider = imp.threshold_scale.value();
        let monochrome_red_val = imp.monochrome_color.rgba().red().to_string();
        let monochrome_green_val = imp.monochrome_color.rgba().green().to_string();
        let monochrome_blue_val = imp.monochrome_color.rgba().blue().to_string();
        let monochrome_inverted = imp.monochrome_invert.is_active() as u8;
        let is_default_monochrome = imp.monochrome_color.rgba() == self.get_default_color();
        debug!("is default? {}", is_default_monochrome);
        let combined_string = format!(
            "{}-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}",
            is_default,
            x_scale_val,
            y_scale_val,
            zoom_val,
            is_monochrome,
            monochrome_slider,
            monochrome_red_val,
            monochrome_green_val,
            monochrome_blue_val,
            monochrome_inverted,
            is_default_monochrome
        );
        debug!("{}", &combined_string);
        combined_string
    }
}
