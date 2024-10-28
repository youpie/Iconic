use adw::{prelude::*, subclass::prelude::*};
use gtk::{gdk, glib};
use image::*;

use crate::GtkTestWindow;

impl GtkTestWindow {
    pub async fn render_to_screen(&self) {
        let imp = self.imp();
        let base = imp
            .bottom_image_file
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .thumbnail
            .clone();
        let top_image = match self.imp().monochrome_switch.state() {
            false => imp
                .top_image_file
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .thumbnail
                .clone(),
            true => self.to_monochrome(
                imp.top_image_file
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .thumbnail
                    .clone(),
                imp.threshold_scale.value() as u8,
                imp.monochrome_color.rgba(),
            ),
        };
        let texture = self.dynamic_image_to_texture(
            &self
                .generate_image(base, top_image, imageops::FilterType::Nearest)
                .await,
        );
        imp.image_view.set_paintable(Some(&texture));
        imp.image_view.queue_draw();
    }

    pub fn to_monochrome(
        &self,
        image: DynamicImage,
        threshold: u8,
        color: gdk::RGBA,
    ) -> DynamicImage {
        // Convert the image to RGBA8
        let rgba_img = image.to_rgba8();
        // Define a threshold value
        let threshold = threshold; // Adjust the threshold value as needed

        // Create a new image buffer for the monochrome image
        let mut mono_img: RgbaImage = ImageBuffer::new(rgba_img.width(), rgba_img.height());
        let switch_state = self.imp().monochrome_invert.is_active();
        // Apply the threshold to create a black and white image, keeping the alpha channel
        for (x, y, pixel) in rgba_img.enumerate_pixels() {
            let rgba = pixel.0;
            let luma = 0.299 * rgba[0] as f32 + 0.587 * rgba[1] as f32 + 0.114 * rgba[2] as f32;
            if !switch_state {
                let mono_pixel = if luma >= threshold as f32 && rgba[3] > 0 {
                    Rgba([
                        (color.red() * 255.0) as u8,
                        (color.green() * 255.0) as u8,
                        (color.blue() * 255.0) as u8,
                        rgba[3] as u8,
                    ]) // White with original alpha
                } else {
                    Rgba([0u8, 0u8, 0u8, 0u8]) // Black with original alpha
                };
                mono_img.put_pixel(x, y, mono_pixel);
            } else {
                let mono_pixel = if luma >= threshold as f32 && rgba[3] > 0 {
                    Rgba([0u8, 0u8, 0u8, 0u8]) // Black with original alpha
                } else {
                    Rgba([
                        (color.red() * 255.0) as u8,
                        (color.green() * 255.0) as u8,
                        (color.blue() * 255.0) as u8,
                        rgba[3] as u8,
                    ]) // White with original alpha
                };
                mono_img.put_pixel(x, y, mono_pixel);
            }
        }

        // Convert the monochrome RgbaImage to DynamicImage
        DynamicImage::ImageRgba8(mono_img)
    }

    pub async fn generate_image(
        &self,
        base_image: image::DynamicImage,
        top_image: image::DynamicImage,
        filter: imageops::FilterType,
    ) -> DynamicImage {
        let imp = self.imp();
        imp.stack.set_visible_child_name("stack_main_page");
        // imp.image_saved.replace(false);
        // imp.save_button.set_sensitive(true);
        let (tx_texture, rx_texture) = async_channel::bounded(1);
        let tx_texture1 = tx_texture.clone();
        let coordinates = (
            (imp.x_scale.value() + 50.0) as i64,
            (imp.y_scale.value() + 50.0) as i64,
        );
        let scale: f32 = imp.size.value() as f32;
        gio::spawn_blocking(move || {
            let mut base = base_image;
            let top = top_image;
            let base_dimension: (i64, i64) =
                ((base.dimensions().0).into(), (base.dimensions().1).into());
            let top = GtkTestWindow::resize_image(top, base.dimensions(), scale, filter);
            let top_dimension: (i64, i64) = (
                (top.dimensions().0 / 2).into(),
                (top.dimensions().1 / 2).into(),
            );
            let final_coordinates: (i64, i64) = (
                ((base_dimension.0 * coordinates.0) / 100) - top_dimension.0,
                ((base_dimension.1 * coordinates.1) / 100) - top_dimension.1,
            );
            imageops::overlay(
                &mut base,
                &top,
                final_coordinates.0.into(),
                final_coordinates.1.into(),
            );
            tx_texture1.send_blocking(base)
        });

        let texture = glib::spawn_future_local(async move { rx_texture.recv().await.unwrap() });
        let image = texture.await.unwrap();
        imp.generated_image.replace(Some(image.clone()));
        image
    }

    pub fn resize_image(
        image: DynamicImage,
        dimensions: (u32, u32),
        slider_position: f32,
        filter: imageops::FilterType,
    ) -> DynamicImage {
        let width: f32 = dimensions.0 as f32;
        let height: f32 = dimensions.1 as f32;
        let scale_factor: f32 = (slider_position + 10.0) / 10.0;
        let new_width: u32 = (width / scale_factor) as u32;
        let new_height: u32 = (height / scale_factor) as u32;
        image.resize(new_width, new_height, filter)
    }
}
