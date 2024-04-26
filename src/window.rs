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
use gettextrs::gettext;
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
        pub open_folder_icon: TemplateChild<gtk::Button>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub open_top_icon: TemplateChild<gtk::Button>,
        #[template_child]
        pub folder_icon_content: TemplateChild<adw::ButtonContent>,
        #[template_child]
        pub top_icon_content: TemplateChild<adw::ButtonContent>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GtkTestWindow {
        const NAME: &'static str = "GtkTestWindow";
        type Type = super::GtkTestWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
            klass.install_action("app.generate_icon", None, move |win, _, _| {
                win.button_clicked();
            });
            klass.install_action("app.open_folder_icon", None, move |win, _, _| {
                glib::spawn_future_local(clone!(@weak win => async move {
                    win.open_file_chooser_gtk(0).await;
                }));
            });
            klass.install_action("app.open_top_icon", None, move |win, _, _| {
                glib::spawn_future_local(clone!(@weak win => async move {
                    win.open_file_chooser_gtk(1).await;
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

    pub fn button_clicked(&self) {
        println!("Button Pressed");
        self.imp().toast_overlay.add_toast(adw::Toast::new("generated"));
    }

    pub async fn open_file_chooser_gtk(&self,what_button:usize) {
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        let filter = gtk::FileFilter::new();
        filter.add_mime_type("image/*");
        filters.append(&filter);
        let dialog = gtk::FileDialog::builder()
                .title(gettext("Open Document"))
                .modal(true)
                .filters(&filters)
                .build();
        let file = dialog.open_future(Some(self)).await;
        match file {
            Ok(x) => {println!("{:#?}",&x.path().unwrap());
                    match what_button {
                        0 => {self.imp().open_folder_icon.set_tooltip_text(Some("The currently set folder image"));
                                self.imp().folder_icon_content.set_icon_name("image-x-generic-symbolic");
                                self.imp().folder_icon_content.set_label(&Self::get_file_name(&x,Some(8),true));},
                        _ => {self.imp().open_top_icon.set_tooltip_text(Some("The currently set top image"));
                                self.imp().top_icon_content.set_icon_name("image-x-generic-symbolic");
                                self.imp().top_icon_content.set_label(&Self::get_file_name(&x,Some(8),true));},
                        }
                    },
            Err(y) => println!("{:#?}",y),
        };

    }

    fn get_file_name(filename: &gio::File, slice: Option<usize>,show_extension: bool) -> String{
        let name = filename.basename().unwrap().into_os_string().into_string().unwrap();
        // name[..6].into
        match slice{
            Some(x) => {
                let period_split:Vec<&str> = name.split(".").collect();
                let file_extension = format!(".{}",period_split.last().unwrap());
                let name_no_extension = name.replace(&file_extension,"");
                let mut substring = String::from(&name_no_extension [..x/2]);
                substring.push_str("...");
                substring.push_str(&name_no_extension[name_no_extension.len()-(x/2)..]);
                if show_extension{
                    substring.push_str(&file_extension);
                }
                substring
            },
            None => name
        }

    }

    #[template_callback]
    fn handle_button_clicked() {
        // Set the label to "Hello World!" after the button has been clicked on
        println!("test");
    }
}

