use std::path::PathBuf;

use adw::subclass::prelude::*;

use image::DynamicImage;

use crate::{objects::properties::MaskType, window::IconicWindow};

impl IconicWindow {
    pub fn get_mask_path(&self) -> Option<PathBuf> {
        let imp = self.imp();
        let properties = imp.file_properties.borrow().clone();
        properties
            .bottom_image_type
            .is_strict_compatible()
            .map(|_| self.load_default_mask())
    }

    // TODO Fix custom mask implementation
    pub fn serve_mask(&self, mask: DynamicImage) -> Option<DynamicImage> {
        let imp = self.imp();
        let mask_setting = imp.file_properties.borrow().mask.clone();
        match mask_setting {
            MaskType::Automatic => Some(mask),
            MaskType::Disabled => None,
            MaskType::Custom(_) => None,
        }
    }

    fn load_default_mask(&self) -> PathBuf {
        let mut path = self.get_built_in_bottom_icon_path("None");
        path.set_file_name("mask.svg");
        path
    }

    pub fn apply_mask_to_top_image(top_image: DynamicImage, mask: DynamicImage) -> DynamicImage {
        let mask_pixels = mask.to_rgba8();
        let mut top_image_pixels = top_image.to_rgba8();
        for (x, y, pixel) in top_image_pixels.enumerate_pixels_mut() {
            let mask_pixel = mask_pixels.get_pixel(x, y);
            let mask_pixel_luminance = (0.299 * mask_pixel[0] as f32
                + 0.587 * mask_pixel[1] as f32
                + 0.114 * mask_pixel[2] as f32) as u8;
            if pixel[3] > mask_pixel_luminance {
                pixel[3] = mask_pixel_luminance;
            };
        }

        DynamicImage::ImageRgba8(top_image_pixels)
    }
}
