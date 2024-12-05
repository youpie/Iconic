use std::error::Error;

use adw::prelude::{AdwDialogExt, AlertDialogExt, AlertDialogExtManual};
use gettextrs::gettext;
use gio::{
    glib,
    prelude::{SettingsExt, SettingsExtManual},
    subclass::prelude::ObjectSubclassIsExt,
};
use log::*;

use crate::GtkTestWindow;

impl GtkTestWindow {
    pub fn drag_and_drop_information_dialog(&self) {
        let imp = self.imp();
        if imp.settings.boolean("drag-and-drop-popup-shown") {
            return ();
        }
        const RESPONSE_OK: &str = "OK";
        let dialog = adw::AlertDialog::builder()
                .heading(&gettext("Drag and drop"))
                .body(&gettext("Did you know that it is possible to drag the folder image straight out of Iconic and drop it into nautilus' property window.\nNo need to save!"))
                .default_response(RESPONSE_OK)
                .build();
        dialog.add_response(RESPONSE_OK, &gettext("OK"));
        dialog.present(Some(self));
        let _ = imp.settings.set("drag-and-drop-popup-shown", true);
    }

    pub async fn top_or_bottom_popup(&self) -> Option<bool> {
        let dnd_switch_state = self.imp().settings.boolean("default-dnd-activated");
        let dnd_radio_state = self.imp().settings.string("default-dnd-action");
        debug!("radio button state: {}", dnd_radio_state);
        if dnd_switch_state {
            return match dnd_radio_state.as_str() {
                "top" => Some(true),
                "bottom" => Some(false),
                _ => None,
            };
        }
        const RESPONSE_TOP: &str = "TOP";
        const RESPONSE_BOTTOM: &str = "BOTTOM";
        let load_question: &str =
            &gettext("Do you want to load this image to the top or bottom layer?");
        let disable_hint: &str = &gettext("Hint: You can disable this pop-up in the settings");
        let dialog = adw::AlertDialog::builder()
            .heading(gettext("Select layer"))
            .body(format!("{load_question} \n <sub> <span foreground=\"#9A9996\"> {disable_hint}</span> </sub>"))
            .body_use_markup(true)
            .default_response(RESPONSE_TOP)
            .build();
        dialog.add_response(RESPONSE_TOP, &gettext("Top"));
        dialog.add_response(RESPONSE_BOTTOM, &gettext("Bottom"));
        dialog.set_response_appearance(RESPONSE_TOP, adw::ResponseAppearance::Suggested);

        match &*dialog.clone().choose_future(self).await {
            RESPONSE_TOP => Some(true),
            RESPONSE_BOTTOM => Some(false),
            _ => None,
        }
    }

    pub async fn confirm_save_changes(&self) -> Result<glib::Propagation, ()> {
        const RESPONSE_CANCEL: &str = "cancel";
        const RESPONSE_DISCARD: &str = "discard";
        const RESPONSE_SAVE: &str = "save";
        let dialog = adw::AlertDialog::builder()
            .heading(gettext("Save Changes?"))
            .body(gettext("Open image contain unsaved changes. Changes which are not saved will be permanently lost"))
            .close_response(RESPONSE_CANCEL)
            .default_response(RESPONSE_SAVE)
            .build();
        dialog.add_response(RESPONSE_CANCEL, &gettext("Cancel"));
        dialog.add_response(RESPONSE_DISCARD, &gettext("Discard"));
        dialog.set_response_appearance(RESPONSE_DISCARD, adw::ResponseAppearance::Destructive);
        dialog.add_response(RESPONSE_SAVE, &gettext("Save"));
        dialog.set_response_appearance(RESPONSE_SAVE, adw::ResponseAppearance::Suggested);

        match &*dialog.clone().choose_future(self).await {
            RESPONSE_CANCEL => {
                dialog.close();
                Ok(glib::Propagation::Stop)
            }
            RESPONSE_DISCARD => Ok(glib::Propagation::Proceed),
            RESPONSE_SAVE => match self.open_save_file_dialog().await {
                Ok(saved) => match saved {
                    true => Ok(glib::Propagation::Proceed),
                    false => Ok(glib::Propagation::Stop),
                },
                Err(error) => {
                    self.show_error_popup(&error.to_string(), true, Some(error));
                    Ok(glib::Propagation::Stop)
                }
            },
            _ => unreachable!(),
        }
    }

    pub fn get_accent_color_and_dialog(&self) -> String {
        let imp = self.imp();
        let accent_color = format!("{:?}", adw::StyleManager::default().accent_color());
        if !imp.settings.boolean("accent-color-popup-shown")
            && accent_color != imp.settings.string("previous-system-accent-color")
        {
            const RESPONSE_OK: &str = "OK";
            let dialog = adw::AlertDialog::builder()
                .heading(&gettext("Accent color changed"))
                .body(&gettext("The system accent color has been changed, Iconic has automatically changed the color of the folder.\nIf you do not want this, you can turn this off in the settings"))
                .default_response(RESPONSE_OK)
                .build();
            dialog.add_response(RESPONSE_OK, &gettext("OK"));
            dialog.present(Some(self));
            let _ = imp.settings.set("accent-color-popup-shown", true);
        }
        accent_color
    }

    pub fn drag_and_drop_regeneration_popup(&self) {
        let imp = self.imp();
        if !imp.settings.boolean("regeneration-hint-shown") {
            const RESPONSE_OK: &str = "OK";
            let dialog = adw::AlertDialog::builder()
                .heading(&gettext("Regenerating Icons"))
                .body(&gettext("If you drag and drop icons and change your accent color. It is then possible to regenerate the images by pressing \"regenerate\" in the menu or by pressing ctrl+R"))
                .default_response(RESPONSE_OK)
                .build();
            dialog.add_response(RESPONSE_OK, &gettext("OK"));
            dialog.present(Some(self));
            let _ = imp.settings.set("regeneration-hint-shown", true);
        }
    }

    pub fn show_error_popup(
        &self,
        message: &str,
        show: bool,
        error: Option<Box<dyn Error + '_>>,
    ) -> Option<adw::AlertDialog> {
        const RESPONSE_OK: &str = "OK";
        let error_text: &str = &gettext("Error");
        let dialog = adw::AlertDialog::builder()
            .heading(format!(
                "<span foreground=\"red\"><b>⚠ {error_text}</b></span>"
            ))
            .heading_use_markup(true)
            .body(message)
            .default_response(RESPONSE_OK)
            .build();
        dialog.add_response(RESPONSE_OK, &gettext("OK"));
        match error {
            Some(ref x) => error!("An error has occured: \"{:?}\"", x),
            None => error!("An error has occured: \"{}\"", message),
        };
        match show {
            true => {
                dialog.present(Some(self));
                None
            }
            false => Some(dialog),
        }
    }
}
