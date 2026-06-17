use image::{DynamicImage, GenericImageView};

use crate::window::IconicWindow;

impl IconicWindow {
    pub fn apply_mask(&self, top_image: DynamicImage, mask: DynamicImage) -> DynamicImage {
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

        DynamicImage::ImageRgba8(top_image_pixels)
    }
}
