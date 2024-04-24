/* window.rs
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

use crate::glib::clone;
use adw::subclass::prelude::*;
use ashpd::desktop::file_chooser::{Choice,  SelectedFiles};
use ashpd::WindowIdentifier;
use gtk::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/emphisia/gtk/window.ui")]
    pub struct GtkTestWindow {
        // Template widgets
        #[template_child]
        pub toolbar: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub button1: TemplateChild<gtk::Button>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub button2: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GtkTestWindow {
        const NAME: &'static str = "GtkTestWindow";
        type Type = super::GtkTestWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
            klass.install_action("app.button_clicked", None, move |win, _, _| {
                win.button_clicked();
            });
            klass.install_action("app.file_picker", None, move |win, _, _| {
                glib::spawn_future_local(clone!(@weak win => async move {
                    // Get native of button for window identifier
                //     let native = win.native().expect("Need to be able to get native.");
                    // Get window identifier so that the dialog will be modal to the main window
                //     let identifier = WindowIdentifier::from_native(&native).await;
                //     let request = UserInformation::request()
                //         .reason("App would like to access user information.")
                //         .identifier(identifier)
                //         .send()
                //         .await;

                //     if let Ok(response) = request.and_then(|r| r.response()) {
                //         println!("User name: {}", response.name());
                //     } else {
                //         println!("Could not access user information.")
                //     }
                    win.open_file_chooser().await;
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GtkTestWindow {}
    impl WidgetImpl for GtkTestWindow {}
    impl WindowImpl for GtkTestWindow {}
    impl ApplicationWindowImpl for GtkTestWindow {}
    impl AdwApplicationWindowImpl for GtkTestWindow {}
}

glib::wrapper! {
    pub struct GtkTestWindow(ObjectSubclass<imp::GtkTestWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

#[gtk::template_callbacks]
impl GtkTestWindow {
    pub fn new<P: IsA<adw::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    #[template_callback]
    pub fn button_clicked(&self) {
        println!("Button Pressed");
        self.imp().toast_overlay.add_toast(adw::Toast::new("TOATS"));
        self.imp().button1.set_label("JE MOEDER");
    }

    #[template_callback]
    pub async fn open_file_chooser(&self) {
        let native = self
            .imp()
            .button2
            .native()
            .expect("Need to be able to get native.");
        // Get window identifier so that the dialog will be modal to the main window
        let identifier = WindowIdentifier::from_native(&native).await;
        let files = SelectedFiles::open_file()
            .title("open a file to read")
            .accept_label("open")
            .modal(true)
            .multiple(true)
            .identifier(identifier)
            .choice(
                Choice::new("encoding", "Encoding", "latin15")
                    .insert("utf8", "Unicode (UTF-8)")
                    .insert("latin15", "Western"),
            )
            // A trick to have a checkbox
            .choice(Choice::boolean("re-encode", "Re-encode", false))
            .send()
            .await;

        println!("{:#?}", files);
    }

    #[template_callback]
    fn handle_button_clicked() {
        // Set the label to "Hello World!" after the button has been clicked on
        println!("test");
    }
}

