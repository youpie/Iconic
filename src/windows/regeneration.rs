use crate::objects::file::File;
use crate::GtkTestWindow;

use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk::gdk::RGBA;
use gtk::gio;
use image::*;
use log::*;
use std::fs;
use std::path::PathBuf;
use tokio;

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
        // SVG's are often very small in size, so if it is an SVG. Save that image. Otherwise store the dynamic image.
        // I do not know how this code below works, but it does. So I am not touching it
        // TODO implement async?
        if file.extension == "image/svg+xml" && original_file != None {
            let new_file = gio::File::for_path(file_path);
            let filestream = new_file.open_readwrite(gio::Cancellable::NONE).unwrap();
            let test = filestream.output_stream();
            let buffer = original_file
                .unwrap()
                .load_bytes(gio::Cancellable::NONE)
                .unwrap();
            test.write_bytes(&buffer.0, gio::Cancellable::NONE).unwrap();
        } else {
            file.dynamic_image
                .save_with_format(file_path, ImageFormat::Png)?;
        }
        Ok(())
    }

    /*
    This function regenerates icon, it replaces all images that were dragged and dropped with ones of the correct system accent color.
    It is currently incredibly slow, but it does work.
    After I added the animation, it got only more ugly. But the animation looks nice :)*/
    pub async fn regenerate_icons(&self) -> GenResult<()> {
        let imp = self.imp();
        let data_path = self.get_data_path();
        let files: fs::ReadDir = fs::read_dir(&data_path).unwrap();
        let mut compatible_files: Vec<fs::DirEntry> = vec![];
        for file in files {
            let current_file = file?;
            let file_name = current_file.file_name();
            debug!("File found: {:?}", file_name);
            let file_name_str = file_name.to_str().unwrap().to_string();
            let mut file_properties = file_name_str.split("-");
            // imp.regeneration_progress
            //     .set_fraction(imp.regeneration_progress.fraction() + step_size);
            if file_properties.nth(0).unwrap_or("folder") != "folder_new" {
                warn!("File not supported for regeneration");
                continue;
            }
            let properties_list: Vec<&str> = file_properties.into_iter().collect();
            if !(properties_list[0].parse::<usize>().unwrap() != 0) {
                warn!("Non-default image, not converting");
                continue;
            }
            let hash = properties_list[11].split(".").nth(0).unwrap();
            let mut top_image_path = self.get_cache_path().join("top_images");
            top_image_path.push(hash);

            if !top_image_path.exists() {
                warn!("Top image file not found");
                continue;
            }
            compatible_files.push(current_file);
        }
        let files_n = compatible_files.len();
        let step_size = 1.0 / files_n as f64;
        let mut file_index: usize = 0;
        for file in compatible_files {
            file_index += 1;
            self.progress_animation(step_size);
            self.file_progress_indicator(file_index, files_n);
            let file_name = file.file_name();
            let file_path = file.path();
            let file_name_str = file_name.to_str().unwrap().to_string();
            let file_properties = file_name_str.split("-");
            let properties_list: Vec<&str> = file_properties.into_iter().collect();
            debug!("properties list: {:?}", properties_list);
            let current_accent_color = self.get_accent_color_and_dialog();
            let bottom_image_path = PathBuf::from(format!(
                "/app/share/folder_icon/folders/folder_{}.svg",
                &current_accent_color
            ));
            let hash = properties_list[12].split(".").nth(0).unwrap();
            let mut top_image_path = self.get_cache_path().join("top_images");
            top_image_path.push(hash);
            let top_image_file = tokio::task::spawn_blocking(move || {
                File::from_path(top_image_path, 512, 0).map_err(|err| err.to_string())
            })
            .await??
            .dynamic_image;
            let top_image =
                self.create_top_image_for_generation(properties_list.clone(), top_image_file);
            self.set_properties(properties_list)?;
            debug!(
                "Creating top icon succesful, now creating bottom icon {:?}",
                bottom_image_path
            );
            let bottom_image_file = tokio::task::spawn_blocking(move || {
                File::from_path(bottom_image_path, 1024, 0).map_err(|err| err.to_string())
            })
            .await??
            .dynamic_image;
            let generated_image = self
                .generate_image(bottom_image_file, top_image, imageops::FilterType::Gaussian)
                .await;
            tokio::task::spawn_blocking(move || generated_image.save(file_path)).await??;
        }
        Ok(())
    }

    fn set_properties(&self, properties: Vec<&str>) -> GenResult<()> {
        let imp = self.imp();
        imp.x_scale.set_value(properties[2].parse()?);
        imp.y_scale.set_value(properties[3].parse()?);
        imp.size.set_value(properties[4].parse()?);
        imp.monochrome_switch
            .set_active(properties[5].parse::<usize>()? != 0);
        imp.threshold_scale.set_value(properties[6].parse()?);
        // imp.monochrome_color.set_rgba(&self.get_default_color());
        imp.monochrome_invert
            .set_active(properties[10].parse::<usize>()? != 0);
        Ok(())
    }

    fn create_top_image_for_generation(
        &self,
        properties: Vec<&str>,
        top_image: DynamicImage,
    ) -> DynamicImage {
        let color = match properties[10] {
            "false" => RGBA::new(
                properties[7].parse().unwrap(),
                properties[8].parse().unwrap(),
                properties[9].parse().unwrap(),
                1.0,
            ),
            _ => self.get_default_color(),
        };
        match properties[5] {
            "1" => self.to_monochrome(top_image, properties[5].parse().unwrap(), color),
            _ => top_image,
        }
    }

    fn progress_animation(&self, step_size: f64) {
        let imp = self.imp();
        debug!("Starting animation");
        let target =
            adw::PropertyAnimationTarget::new(&imp.regeneration_progress.to_owned(), "fraction");
        adw::TimedAnimation::builder()
            .target(&target)
            .widget(&imp.regeneration_progress.to_owned())
            .value_from(imp.regeneration_progress.fraction())
            .value_to(imp.regeneration_progress.fraction() + step_size)
            .duration(600)
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
