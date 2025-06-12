use crate::objects::errors::IntoResult;
use crate::objects::file::File;
use crate::{GtkTestWindow, objects::errors::show_error_popup};

use adw::TimedAnimation;
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::{gettext, ngettext};
use gio::glib;
use gtk::gdk::RGBA;
use gtk::gio;
use image::*;
use log::*;
use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::sync::Arc;

type GenResult<T> = Result<T, Box<dyn std::error::Error>>;

// The properties of the to be regenerated file, are stored in the filename
// To get the properties, split the filename at the '-'.
// This enum provides a more visual method of knowing at what index each property is..
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
    pub fn check_if_regeneration_needed(&self) -> bool {
        let imp = self.imp();
        let previous_accent: String = imp.settings.string("previous-system-accent-color").into();
        let current_accent = self.get_accent_color_and_show_dialog();
        // error!("previous {previous_accent} current {current_accent}");
        let id = imp.regeneration_lock.get();
        imp.regeneration_lock.replace(id + 1);
        if previous_accent != current_accent && imp.settings.boolean("automatic-regeneration") {
            glib::spawn_future_local(glib::clone!(
                #[weak(rename_to = win)]
                self,
                async move {
                    match win.regenerate_icons().await {
                        Ok(_) => info!("Regeneration succesfull!"),
                        Err(x) => {
                            error!("{}", x.to_string());
                        }
                    };
                }
            ));
            return true;
        }
        false
    }

    pub fn store_top_image_in_cache(&self, file: &File) -> GenResult<()> {
        if !self.current_file_uses_compatible_bottom_image() {
            info!("Current file does not use a compatible bottom image. no use caching the file");
            return Ok(());
        }
        //create folder inside cache, if it does not yet exist
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

    // This function regenerates icon, it replaces all images that were dragged and dropped with ones of the correct system accent color.
    pub async fn regenerate_icons(&self) -> GenResult<()> {
        let imp = self.imp();
        let id = imp.regeneration_lock.get();
        // First set iconic as busy. By getting a Arc reference
        // I doubt this is the best approach, but Hey it works!
        let _iconic_busy = Arc::clone(&imp.app_busy);
        let data_path = self.get_data_path();
        let mut incompatible_files_n: u32 = 0;
        let compatible_files =
            self.find_regeneratable_icons(data_path, &mut incompatible_files_n)?;

        // Stop if there are no files to regenerate
        match compatible_files.len() {
            0 => {
                imp.toast_overlay
                    .add_toast(adw::Toast::new(&gettext("Nothing to regenerate")));
                return Ok(());
            }
            _ => {
                imp.toast_overlay
                    .add_toast(adw::Toast::new(&gettext("Regenerating icons")));
            }
        }
        imp.regeneration_revealer.set_reveal_child(true);
        imp.regeneration_osd.set_fraction(0.0);
        imp.regeneration_osd_second.set_fraction(0.0);
        let files_n = compatible_files.len();
        let mut last_animation = None;
        let mut last_animation_second = None; // Second here means 2nd not sec
        let step_size = 1.0 / files_n as f64;
        let mut regeneration_errors = vec![];

        for file in compatible_files {
            // In the regeneration lock, a value is saved, if it has changed.
            // It means a new regeneration instance has started. So stop this one
            // This is done to prevent two instances from fighting if for example
            // The accent color is changed during regeneration
            if imp.regeneration_lock.get() != id {
                error!("Stopping regeneration");
                break;
            }
            let name = &file.file_name();
            match self.regenerate_and_save_single_icon(file).await {
                Ok(_) => (),
                Err(error) => {
                    error!("Error while generating {:?}: {}", &name, &error.to_string());
                    // Everytime an error occurs. Push this to the errors list
                    regeneration_errors.push(error)
                }
            }
            // Update the progress bars
            // I need two as the animation needs to play on both welcome and main screen which is not possible
            // with only one animation. Unless i'm missing something
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
        // If the errors list is not empty
        // Show a pop up with the ammount of errors
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

    // This function regenerates a single compatible icon
    async fn regenerate_and_save_single_icon(&self, file: DirEntry) -> GenResult<()> {
        // Get path and filename of icon saved in datadir
        let file_name = file.file_name();
        let file_path = file.path();
        let file_name = file_name.to_str().into_result()?.to_string();

        // The settings of the specific icon are specified in the filename.
        let file_properties = file_name.split("-");
        let properties_list: Vec<&str> = file_properties.into_iter().collect();

        // Get the current accent color
        let current_accent_color = self.get_accent_color_and_show_dialog();

        // Icons that are compatible for regeneration are only allowed to use default folder images.
        // So when regenerating icons, you need the folder which is the same color as the current accent color
        let bottom_image_path = PathBuf::from(format!(
            "/app/share/folder_icon/folders/folder_{}.svg",
            &current_accent_color
        ));
        // The filename of the cached top image is equal to it's hash
        // So by looking in the cache folder, at the file of the same name as the hash,
        // the file can be found
        let hash = properties_list[FilenameProperty::Hash as usize]
            .split(".")
            .nth(0)
            .into_result()?;
        let mut top_image_path = Self::get_cache_path().join("top_images");
        top_image_path.push(hash);
        let top_image_file = gio::spawn_blocking(move || {
            File::from_path(top_image_path, 1024, 0).map_err(|err| err.to_string())
        })
        .await
        .unwrap()?
        .dynamic_image;

        let slider_values = self.get_regeneration_slider_values(properties_list.clone())?;
        // Create the top image
        let top_image = self.set_correct_monochrome_values_based_on_regeneration_properties(
            properties_list,
            top_image_file,
        )?;
        let bottom_image_file = gio::spawn_blocking(move || {
            File::from_path(bottom_image_path, 1024, 0).map_err(|err| err.to_string())
        })
        .await
        .unwrap()?
        .dynamic_image;
        info!("Generating image");
        // Using the generic generate_image function. The icon can faithfully be recreated
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

    // Search in the list of stored icons to see which ones are valid for regeneration
    fn find_regeneratable_icons(
        &self,
        dir: PathBuf,
        incompatible_files: &mut u32,
    ) -> GenResult<Vec<fs::DirEntry>> {
        let mut regeneratable: Vec<fs::DirEntry> = vec![];
        // Walk the directory and loop over every file
        let files: fs::ReadDir = fs::read_dir(&dir)?;
        for file in files {
            let current_file = file?;
            let file_name = current_file.file_name();
            debug!("File found: {:?}", file_name);
            let file_name_str = file_name.to_str().into_result()?.to_string();
            let file_properties: Vec<&str> = file_name_str.split("-").collect();
            // If the first property is not "folder_new" it's using an format for file properties
            // Which iconic no longer supports
            // It would have been better to use a version number
            if file_properties[FilenameProperty::FileName as usize] != "folder_new" {
                info!("File not supported for regeneration");
                *incompatible_files += 1;
                continue;
            }
            // If the image does not use the Default bottom image
            // It is not valid for regeneration
            if !(file_properties[FilenameProperty::DefaultBottomImage as usize].parse::<usize>()?
                != 0)
            {
                info!("Non-default image, not converting");
                *incompatible_files += 1;
                continue;
            }
            // Get the hash, which is the filename of the cached top image
            let hash = file_properties[FilenameProperty::Hash as usize]
                .split(".")
                .nth(0)
                .into_result()?;
            let mut top_image_path = Self::get_cache_path().join("top_images");
            top_image_path.push(hash);

            // If that top image does not exist, just mark it as not valid for regeneration
            if !top_image_path.exists() {
                warn!("Top image file not found");
                *incompatible_files += 1;
                continue;
            }
            regeneratable.push(current_file);
        }
        Ok(regeneratable)
    }

    // Get the X, Y, Zoom values from the regeneration file properties
    fn get_regeneration_slider_values(&self, properties: Vec<&str>) -> GenResult<(f64, f64, f64)> {
        let x_scale: f64 = properties[FilenameProperty::XScale as usize].parse()?;
        let y_scale: f64 = properties[FilenameProperty::YScale as usize].parse()?;
        let size: f64 = properties[FilenameProperty::ZoomVal as usize].parse()?;
        Ok((x_scale, y_scale, size))
    }

    // Create the top image based on the properties of the to-be regenerated icon
    // I am really bad at function names
    fn set_correct_monochrome_values_based_on_regeneration_properties(
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
        let is_default = self.current_file_uses_compatible_bottom_image() as u8;
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

    fn current_file_uses_compatible_bottom_image(&self) -> bool {
        let imp = self.imp();
        !imp.settings.boolean("manual-bottom-image-selection")
            && imp.settings.string("selected-accent-color").as_str() == "None"
            && !imp.temp_image_loaded.get()
    }
}
