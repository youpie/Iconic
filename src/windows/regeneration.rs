use crate::objects::errors::IntoResult;
use crate::objects::file::File;
use crate::{objects::errors::show_error_popup, GtkTestWindow, RUNTIME};

use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk::gdk::RGBA;
use gtk::gio;
use image::*;
use log::*;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

type GenResult<T> = Result<T, Box<dyn std::error::Error>>;

impl GtkTestWindow {
    pub fn store_top_image_in_cache(
        &self,
        file: &File,
        original_file: Option<&gio::File>,
    ) -> GenResult<()> {
        let imp = self.imp();
        if !imp.settings.boolean("store-top-in-cache") {
            debug!("Top cache is disabled");
            return Ok(());
        }
        if imp.settings.boolean("manual-bottom-image-selection") {
            debug!("Non-default bottom image");
            return Ok(());
        }
        //create folder inside cache
        let cache_path = self.get_cache_path().join("top_images");
        if !cache_path.exists() {
            debug!("Top icon cache file does not yet exist, creating");
            fs::create_dir(&cache_path)?;
        }
        let file_name = file.hash;
        let mut file_path = cache_path.clone();
        file_path.push(file_name.to_string());
        debug!("File path: {:?}", file_path);
        debug!("File name: {:?}", file.filename);
        match file_path.exists() {
            true => {
                debug!("file already exists with name");
                return Ok(());
            }
            false => {
                debug!("File does not yet exist, creating");
                fs::File::create(&file_path)?;
            }
        };
        // The dynamic image is quite a lot bigger than the original file (often), so only if there is no original file (pasted images) use the dynam icimage
        // TODO check if the dynamic image or original file bigger is
        let new_file = gio::File::for_path(&file_path);
        let filestream = new_file.open_readwrite(gio::Cancellable::NONE)?;
        let test = filestream.output_stream();
        match original_file {
            Some(file) => {
                let buffer = file.load_bytes(gio::Cancellable::NONE)?;
                test.write_bytes(&buffer.0, gio::Cancellable::NONE)?;
            }
            None => {
                file.dynamic_image
                    .save_with_format(file_path, ImageFormat::Png)?;
            }
        }
        Ok(())
    }

    /*
    This function regenerates icon, it replaces all images that were dragged and dropped with ones of the correct system accent color.
    It is currently incredibly slow, but it does work.
    After I added the animation, it got only more ugly. But the animation looks nice :)*/
    pub async fn regenerate_icons(&self, delay: bool) -> GenResult<()> {
        let imp = self.imp();
        let data_path = self.get_data_path();
        let mut incompatible_files_n: u32 = 0;
        let compatible_files =
            self.find_regeneratable_icons(data_path, &mut incompatible_files_n)?;
        imp.regeneration_revealer.set_reveal_child(true);
        imp.regeneration_osd.set_fraction(0.0);
        let previous_control_visibility = imp.x_scale.is_visible();
        self.user_control_visibilty(false);
        imp.stack.set_visible_child_name("stack_main_page");
        let files_n = compatible_files.len();
        if files_n == 0 && delay {
            show_error_popup(
                &self,
                &format!(
                    "{}{}{}",
                    gettext("All "),
                    incompatible_files_n,
                    gettext(" files are not compatible for regeneration")
                ),
                true,
                None,
            );
        }
        let step_size = 1.0 / files_n as f64;
        for file in compatible_files {
            info!("Loading new file");

            self.progress_animation(step_size);
            let file_name = file.file_name();
            let file_path = file.path();
            let file_name = file_name.to_str().into_result()?.to_string();
            let file_properties = file_name.split("-");
            let properties_list: Vec<&str> = file_properties.into_iter().collect();
            info!("properties list: {:?}", properties_list);
            let current_accent_color = self.get_accent_color_and_show_dialog();
            let bottom_image_path = PathBuf::from(format!(
                "/app/share/folder_icon/folders/folder_{}.svg",
                &current_accent_color
            ));
            let hash = properties_list[12].split(".").nth(0).into_result()?;
            let mut top_image_path = self.get_cache_path().join("top_images");
            top_image_path.push(hash);
            info!("Loading top image file");
            let top_image_file = RUNTIME
                .spawn_blocking(move || {
                    File::from_path(top_image_path, 1024, 0).map_err(|err| err.to_string())
                })
                .await??
                .dynamic_image;
            self.set_properties(properties_list.clone())?;
            let top_image =
                self.create_top_image_for_generation(properties_list, top_image_file)?;
            info!(
                "Creating top icon succesful, now creating bottom icon {:?}",
                bottom_image_path
            );
            let bottom_image_file = RUNTIME
                .spawn_blocking(move || {
                    File::from_path(bottom_image_path, 1024, 0).map_err(|err| err.to_string())
                })
                .await??
                .dynamic_image;
            //self.image_animation(false);
            // if delay {
            //     RUNTIME
            //         .spawn_blocking(move || {
            //             std::thread::sleep(Duration::from_millis(200));
            //         })
            //         .await?;
            // }
            info!("Generating image");
            let generated_image = self
                .generate_image(bottom_image_file, top_image, imageops::FilterType::Gaussian)
                .await;
            info!("Setting texture");
            //let pixbuf = self.dynamic_image_to_texture(&generated_image);
            //imp.regeneration_image_view.set_paintable(Some(&pixbuf));
            //imp.regeneration_image_view.queue_draw();
            info!("Updating indicators");
            //self.file_progress_indicator(file_index, files_n);
            info!("Image animation");
            //self.image_animation(true);
            info!("Saving image");
            match RUNTIME
                .spawn_blocking(move || {
                    generated_image.save_with_format(file_path, ImageFormat::Png)
                })
                .await?
            {
                Ok(_) => info!("Saving Succesful"),
                Err(x) => error!("Saving failed: {:?}", x),
            };
            info!("Waiting");
            // if delay {
            //     RUNTIME
            //         .spawn_blocking(move || {
            //             std::thread::sleep(Duration::from_millis(400));
            //         })
            //         .await?; //I worked really hard on my animation but the app is too fast in production. But it is my own app and I can do what I want
            // }
        }
        imp.regeneration_revealer.set_reveal_child(false);
        self.user_control_visibilty(previous_control_visibility);
        self.default_sliders();
        self.reset_colors();
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
            let mut file_properties = file_name_str.split("-");
            // imp.regeneration_progress
            //     .set_fraction(imp.regeneration_progress.fraction() + step_size);
            if file_properties.nth(0).unwrap_or("folder") != "folder_new" {
                warn!("File not supported for regeneration");
                continue;
            }
            let properties_list: Vec<&str> = file_properties.into_iter().collect();
            if !(properties_list[0].parse::<usize>()? != 0) {
                warn!("Non-default image, not converting");
                continue;
            }
            let hash = properties_list[11].split(".").nth(0).into_result()?;
            let mut top_image_path = self.get_cache_path().join("top_images");
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

    fn set_properties(&self, properties: Vec<&str>) -> GenResult<()> {
        let imp = self.imp();
        imp.x_scale.set_value(properties[2].parse()?);
        imp.y_scale.set_value(properties[3].parse()?);
        imp.size.set_value(properties[4].parse()?);
        imp.monochrome_switch
            .set_active(properties[5].parse::<usize>()? != 0);
        imp.threshold_scale.set_value(properties[6].parse()?);
        imp.monochrome_color.set_rgba(&self.current_accent_rgba()?);
        imp.monochrome_invert
            .set_active(properties[10].parse::<usize>()? != 0);
        Ok(())
    }

    fn create_top_image_for_generation(
        &self,
        properties: Vec<&str>,
        top_image: DynamicImage,
    ) -> GenResult<DynamicImage> {
        let color = match properties[10] {
            "false" => RGBA::new(
                properties[7].parse()?,
                properties[8].parse()?,
                properties[9].parse()?,
                1.0,
            ),
            _ => self.current_accent_rgba()?,
        };
        match properties[5] {
            "1" => Ok(self.to_monochrome(top_image, properties[6].parse()?, color)),
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

    fn progress_animation(&self, step_size: f64) {
        let imp = self.imp();
        let target =
            adw::PropertyAnimationTarget::new(&imp.regeneration_osd.to_owned(), "fraction");
        adw::TimedAnimation::builder()
            .target(&target)
            .widget(&imp.regeneration_osd.to_owned())
            .value_from(imp.regeneration_osd.fraction())
            .value_to(imp.regeneration_osd.fraction() + step_size)
            .duration(20)
            .easing(adw::Easing::EaseInOutCubic)
            .build()
            .play();
    }

    fn image_animation(&self, increase: bool) {
        let (start, end) = match increase {
            true => (0.0, 1.0),
            false => (1.0, 0.0),
        };
        let imp = self.imp();
        debug!("Starting animation");
        let target =
            adw::PropertyAnimationTarget::new(&imp.regeneration_image_view.to_owned(), "opacity");
        adw::TimedAnimation::builder()
            .target(&target)
            .widget(&imp.regeneration_image_view.to_owned())
            .value_from(start)
            .value_to(end)
            .duration(200)
            .easing(adw::Easing::EaseInOutCubic)
            .build()
            .play();
        debug!("Animation done ");
    }

    fn file_progress_indicator(&self, file_index: usize, total_files: usize) {
        let imp = self.imp();
        let file_string = gettext("File");
        imp.regeneration_file
            .set_label(&format!("{} {}/{}", file_string, file_index, total_files));
    }
}
