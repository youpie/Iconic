use crate::objects::file::File;
use crate::GtkTestWindow;

use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use log::*;
use std::env;
use std::fs;
use std::path::PathBuf;

impl GtkTestWindow {
    pub fn store_top_image_in_cache(&self, file: &File, original_file: &gio::File) {
        let imp = self.imp();
        // if !imp.settings.boolean("store-top-in-cache") {
        //     debug!("Top cache is disabled");
        //     return ();
        // }
        if imp.settings.boolean("manual-bottom-image-selection") {
            debug!("Non-default bottom image");
            return ();
        }
        //create folder inside cache
        let cache_path = self.get_cache_path().join("top_images");
        debug!("{:?}", cache_path)
    }
}
