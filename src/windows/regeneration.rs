use crate::objects::file::File;
use crate::GtkTestWindow;

use adw::{prelude::*, subclass::prelude::*};
use gtk::gdk::RGBA;
use gtk::gio;
use image::*;
use log::*;
use std::fs;
use std::path::PathBuf;

type GenResult<T> = Result<T, Box<dyn std::error::Error>>;

impl GtkTestWindow {
    pub fn store_top_image_in_cache(
        &self,
        file: &File,
        original_file: &gio::File,
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
        if file.extension == "image/svg+xml" {
            let new_file = gio::File::for_path(file_path);
            let filestream = new_file.open_readwrite(gio::Cancellable::NONE).unwrap();
            let test = filestream.output_stream();
            let buffer = original_file.load_bytes(gio::Cancellable::NONE).unwrap();
            test.write_bytes(&buffer.0, gio::Cancellable::NONE).unwrap();
        } else {
            file.dynamic_image.save(file_path)?;
        }
        Ok(())
    }

    pub async fn regenerate_icons(&self) -> GenResult<()> {
        let imp = self.imp();
        let data_path = self.get_data_path();
        for file in fs::read_dir(data_path).unwrap() {
            let current_file = file?;
            let file_name = current_file.file_name();
            let file_path = current_file.path();
            debug!("File found: {:?}", file_name);
            let file_name_str = file_name.to_str().unwrap().to_string();
            let mut file_properties = file_name_str.split("-");
            if file_properties.nth(0).unwrap_or("folder") != "folder_new" {
                warn!("File not supported for regeneration");
                continue;
            }
            let properties_list: Vec<&str> = file_properties.into_iter().collect();
            if !(properties_list[0].parse::<usize>().unwrap() != 0) {
                warn!("Non-default image, not converting");
                continue;
            }

            debug!("properties list: {:?}", properties_list);
            let current_accent_color = self.get_accent_color_and_dialog();
            let bottom_image_path = PathBuf::from(format!(
                "/app/share/folder_icon/folders/folder_{}.svg",
                &current_accent_color
            ));
            let hash = properties_list[10].split(".").nth(0).unwrap();
            let mut top_image_path = self.get_cache_path().join("top_images");
            top_image_path.push(hash);
            self.set_properties(properties_list);
            let generated_image = self
                .generate_image(
                    File::from_path(bottom_image_path, 1024, 255)?.dynamic_image,
                    File::from_path(top_image_path, 1024, 255)?.dynamic_image,
                    imageops::FilterType::Gaussian,
                )
                .await;
            generated_image.save(file_path)?;
        }
        Ok(())
    }

    fn set_properties(&self, properties: Vec<&str>) {
        let imp = self.imp();
        imp.x_scale.set_value(properties[1].parse().unwrap());
        imp.y_scale.set_value(properties[2].parse().unwrap());
        imp.size.set_value(properties[3].parse().unwrap());
        imp.monochrome_switch
            .set_active(properties[4].parse::<usize>().unwrap() != 0);
        imp.threshold_scale
            .set_value(properties[5].parse().unwrap());
        // imp.monochrome_color.set_rgba(&self.get_default_color());
        imp.monochrome_invert
            .set_active(properties[9].parse::<usize>().unwrap() != 0);
    }

    // fn create_top_image(&self, properties: Vec<&str>, top_image: DynamicImage) -> DynamicImage {
    //     self.to_monochrome(top_image, properties[5].parse().unwrap(), color)
    // }
}
