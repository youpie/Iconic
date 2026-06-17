use adw::{prelude::*, subclass::prelude::*};
use image::{DynamicImage, GenericImageView};

use crate::{
    GenResult,
    objects::{
        file::File,
        properties::{BottomImageType, FileProperties},
    },
    window::IconicWindow,
};

impl IconicWindow {
    /// Return the mask of the supplied image, assumes the bottom_image is supplied
    pub fn get_mask(&self, image: DynamicImage) -> GenResult<DynamicImage> {
        let imp = self.imp();

        let properties = imp.file_properties.borrow().clone();

        let mask = match properties.bottom_image_type {
            BottomImageType::Custom(_) => self.auto_generate_mask(image),
            _ => self.load_default_mask()?,
        };
        *imp.image_mask.write().unwrap() = mask.clone();
        Ok(mask)
    }

    fn load_default_mask(&self) -> GenResult<DynamicImage> {
        let mut path = self.get_built_in_bottom_icon_path("None");
        path.set_file_name("mask.svg");
        let mask_file = File::from_path(path, 1024, 0)
            .map_err(|e| format!("Failed to get mask {}", e.to_string()))?;
        Ok(self.auto_generate_mask(mask_file.dynamic_image))
    }

    fn auto_generate_mask(&self, image: DynamicImage) -> DynamicImage {
        let bottom_image_pixels = image.to_rgba8();
        let mut mask_pixels = bottom_image_pixels.clone();

        for (x, y, pixel) in bottom_image_pixels.enumerate_pixels() {
            let mask_pixel = mask_pixels.get_pixel_mut(x, y);
            *mask_pixel = image::Rgba([pixel[3], pixel[3], pixel[3], 255u8]);
        }
        DynamicImage::ImageRgba8(mask_pixels)
    }

    fn apply_mask_to_top_image(
        &self,
        top_image: DynamicImage,
        mask: DynamicImage,
    ) -> Option<DynamicImage> {
        let mask = if mask.dimensions() != top_image.dimensions() {
            let dimensions = top_image.dimensions();
            mask.resize(
                dimensions.0,
                dimensions.1,
                image::imageops::FilterType::Nearest,
            )
        } else {
            mask
        };

        let mask_pixels = mask.to_rgb8();
        let mut top_image_pixels = top_image.to_rgba8();

        for (x, y, pixel) in top_image_pixels.enumerate_pixels_mut() {
            let mask_pixel = mask_pixels.get_pixel(x, y);
            let mask_pixel_luminance = (0.299 * mask_pixel[0] as f32
                + 0.587 * mask_pixel[1] as f32
                + 0.114 * mask_pixel[2] as f32) as u8;
            pixel[3] = mask_pixel_luminance;
        }

        Some(DynamicImage::ImageRgba8(top_image_pixels))
    }
}
