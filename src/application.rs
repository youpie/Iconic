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

use adw::prelude::AdwDialogExt;
use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};
use std::cell::OnceCell;

use crate::config::VERSION;
use crate::GtkTestWindow;
use crate::glib::WeakRef;

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
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
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

    // fn main_window(&self) -> GtkTestWindow {
    //     self.imp().window.get().expect("could not get window").upgrade().expect("could not upgrade")
    // }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        // let button_press_action = gio::ActionEntry::builder("button_clicked")
        //     .activate(move |app: &Self, _, _| {
        //         app.handle_button_clicked();
        //     }).build();
        self.add_action_entries([quit_action, about_action]);
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.button_clicked", &["<primary>p"]);
        println!("test");
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutDialog::builder()
            .application_name("gtk_test")
            .application_icon("com.emphisia.gtk")
            .developer_name("Youpie")
            .version(VERSION)
            .developers(vec!["Youpie"])
            .copyright("© 2024 Youpie")
            .build();
        about.present(&window);
    }
}

