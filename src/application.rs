/* application.rs
 *
 * Copyright 2024 Youpie
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::config::{APP_ICON, VERSION};
use crate::glib::WeakRef;
use crate::settings::settings::PreferencesDialog;
use crate::GtkTestWindow;
use adw::prelude::AdwDialogExt;
use adw::subclass::prelude::*;
use ashpd::desktop::open_uri::OpenFileRequest;
use ashpd::url;
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::License;
use gtk::{gio, glib};
use std::cell::OnceCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct GtkTestApplication {
        pub window: OnceCell<WeakRef<GtkTestWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GtkTestApplication {
        const NAME: &'static str = "GtkTestApplication";
        type Type = super::GtkTestApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for GtkTestApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.setup_accels();
        }
    }

    impl ApplicationImpl for GtkTestApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = GtkTestWindow::new(&*application);
                window.upcast()
            };

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for GtkTestApplication {}
    impl AdwApplicationImpl for GtkTestApplication {}
}

glib::wrapper! {
    pub struct GtkTestApplication(ObjectSubclass<imp::GtkTestApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GtkTestApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        let settings_action = gio::ActionEntry::builder("preferences")
            .activate(move |app: &Self, _, _| app.show_preferences_dialog())
            .build();
        let open_action = gio::ActionEntry::builder("open top")
            .activate(move |app: &Self, _, _| app.open_function())
            .build();
        let open_folder_action = gio::ActionEntry::builder("open folder")
            .activate(move |app: &Self, _, _| app.open_folder_function())
            .build();
        let paste_action = gio::ActionEntry::builder("paste")
            .activate(move |app: &Self, _, _| app.paste_image())
            .build();
        let donate_action = gio::ActionEntry::builder("donate")
            .activate(move |app: &Self, _, _| app.donate())
            .build();
        self.add_action_entries([
            quit_action,
            about_action,
            settings_action,
            open_action,
            open_folder_action,
            paste_action,
            donate_action,
        ]);
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.save_button", &["<primary>s"]);
        self.set_accels_for_action("app.open_top_icon", &["<primary>o"]);
        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("app.select_folder", &["<primary><shift>o"]);
        self.set_accels_for_action("app.paste", &["<primary>v"]);
        self.set_accels_for_action("app.regeneration", &["<primary>r"]);
    }

    fn show_preferences_dialog(&self) {
        let preferences = PreferencesDialog::new();
        let window = self.active_window().unwrap();

        adw::prelude::AdwDialogExt::present(&preferences, Some(&window));
    }

    fn open_function(&self) {
        self.activate_action("app.open_top_icon", None);
    }

    fn open_folder_function(&self) {
        self.activate_action("app.select_folder", None);
    }

    fn donate(&self) {
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = _win)]
            self,
            async move {
                let uri = url::Url::parse("https://ko-fi.com/youpie").unwrap();
                //let root = win.native().unwrap();
                //let identifier = WindowIdentifier::from_native(&root).await;
                let request = OpenFileRequest::default();
                request.send_uri(&uri).await.unwrap();
            }
        ));
    }

    fn paste_image(&self) {
        self.activate_action("app.paste", None);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();

        let about = adw::AboutDialog::builder()
            .application_name("Iconic")
            .application_icon(APP_ICON)
            .developer_name("Youpie")
            .developers(vec![
                "Youpie https://github.com/youpie",
                "Qustio https://github.com/Qustio",
            ])
            // Translators: This should not be translated, Please enter your credits here instead (format: "Name https://example.com" or "Name <email@example.com>", no quotes)
            .translator_credits(gettext("translator-credits"))
            .version(VERSION)
            .issue_url("https://github.com/youpie/Iconic/issues")
            .website("https://github.com/youpie/Iconic")
            .license_type(License::Gpl30)
            .copyright("© 2024 YoupDeGamerNL")
            .build();
        about.present(Some(&window));
    }
}
