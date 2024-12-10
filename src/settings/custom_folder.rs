use gio::{prelude::SettingsExt, subclass::prelude::ObjectSubclassIsExt};
use gtk::prelude::WidgetExt;
use log::*;

use crate::{settings::settings::PreferencesDialog, window::GtkTestWindow};

impl PreferencesDialog {
    pub fn show_color_options(&self) {
        let imp = self.imp();
        let color_setting = imp.settings.string("selected-accent-color") == "Custom"
            && !imp.use_system_color.is_active();
        imp.primary_color_row.set_visible(color_setting);
        imp.secondary_color_row.set_visible(color_setting);
    }

    pub fn show_reset_primary(&self) {
        let imp = self.imp();
        let default_rgba = GtkTestWindow::to_rgba(164, 202, 238);
        let is_not_default_rgb = imp.primary_folder_color.rgba() != default_rgba;
        imp.reset_color_primary.set_visible(is_not_default_rgb);
    }

    pub fn show_reset_secondary(&self) {
        let imp = self.imp();
        let default_rgba = GtkTestWindow::to_rgba(67, 141, 230);
        let is_not_default_rgb = imp.secondary_folder_color.rgba() != default_rgba;
        imp.reset_color_secondary.set_visible(is_not_default_rgb);
    }
}
