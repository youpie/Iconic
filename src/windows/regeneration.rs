use crate::objects::file::File;
use crate::GtkTestWindow;

use adw::{prelude::*, subclass::prelude::*};
use gtk::gio;
use log::*;
use std::fs;

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
}
