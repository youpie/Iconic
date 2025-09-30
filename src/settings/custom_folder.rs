use gio::{prelude::SettingsExt, subclass::prelude::ObjectSubclassIsExt};
use gtk::prelude::WidgetExt;

use crate::settings::settings::PreferencesDialog;

impl PreferencesDialog {
    pub fn show_color_options(&self) {
        let imp = self.imp();
        let color_setting = imp.settings.string("selected-accent-color") == "Custom"
            && !imp.use_system_color.is_active();
        // imp.reveal_custom_colors.set_reveal_child(color_setting);
        imp.primary_color_row.set_visible(color_setting);
        imp.secondary_color_row.set_visible(color_setting);
    }
}
