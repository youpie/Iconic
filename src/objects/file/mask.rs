use std::path::PathBuf;

use image::{DynamicImage, imageops};

use crate::GenResult;

type MainMask = DynamicImage;
type ThumbnailMask = DynamicImage;

impl super::file::File {
    pub fn auto_generate_mask(image: &DynamicImage) -> DynamicImage {
        let bottom_image_pixels = image.to_rgba8();
        let mut mask_pixels = bottom_image_pixels.clone();

        for (x, y, pixel) in bottom_image_pixels.enumerate_pixels() {
            let mask_pixel = mask_pixels.get_pixel_mut(x, y);
            *mask_pixel = image::Rgba([pixel[3], pixel[3], pixel[3], 255u8]);
        }
        DynamicImage::ImageRgba8(mask_pixels)
    }

    pub(super) fn get_masks(
        main_size: u32,
        thumbnail_size: u32,
        image: &DynamicImage,
        mask_path: Option<PathBuf>,
    ) -> GenResult<(MainMask, ThumbnailMask)> {
        let image_mask = if let Some(path) = mask_path {
            Self::load_file(&gio::File::for_path(path), main_size)
                .map_err(|e| format!("Failed to load path: {}", e.to_string()))?
                .0
        } else {
            Self::auto_generate_mask(&image)
        };
        let thumbnail_mask = image_mask.clone().resize(
            thumbnail_size,
            thumbnail_size,
            imageops::FilterType::Nearest,
        );
        Ok((image_mask, thumbnail_mask))
    }
}
