use gio::{prelude::SettingsExt, subclass::prelude::ObjectSubclassIsExt};
use gtk::{gdk::RGBA, prelude::WidgetExt};
use hex::FromHex;
use log::*;

use crate::{settings::settings::PreferencesDialog, window::GtkTestWindow};

impl PreferencesDialog {
    pub fn show_color_options(&self) {
        let imp = self.imp();
        let color_setting = imp.settings.string("selected-accent-color") == "Custom"
            && !imp.use_system_color.is_active();
        // imp.reveal_custom_colors.set_reveal_child(color_setting);
        imp.primary_color_row.set_visible(color_setting);
        imp.secondary_color_row.set_visible(color_setting);
    }

    pub fn rgba_to_hex(&self, rgba: RGBA) -> String {
        let red = format!("{:02X?}", (rgba.red() * 255.0) as u8);
        let green = format!("{:02X?}", (rgba.green() * 255.0) as u8);
        let blue = format!("{:02X?}", (rgba.blue() * 255.0) as u8);

        let hex = format!("{}{}{}", red, green, blue);
        debug!("{}", &hex);
        hex
    }

    pub fn hex_to_rgba(hex: String) -> RGBA {
        let decoded = <[u8; 3]>::from_hex(hex).unwrap_or([255, 255, 255]);
        GtkTestWindow::to_rgba(decoded[0], decoded[1], decoded[2])
    }
}
