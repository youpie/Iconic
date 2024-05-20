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
use gtk::{gio, glib,gdk};
use image::*;
use crate::objects::file::File;
use std::cell::RefCell;
use cairo::*;
use std::io::*;
use std::time::Duration;
use std::thread;


mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/nl/emphisia/icon/window.ui")]
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
        #[template_child]
        pub generate_icon: TemplateChild<gtk::Button>,
        #[template_child]
        pub image_view: TemplateChild<gtk::Picture>,

        pub folder_image_file: RefCell<Option<File>>,
        pub top_image_file: RefCell<Option<File>>,
    }

    impl Default for GtkTestWindow {
        fn default() -> Self {
            Self{
                toolbar: TemplateChild::default(),
                header_bar: TemplateChild::default(),
                label: TemplateChild::default(),
                open_folder_icon: TemplateChild::default(),
                toast_overlay: TemplateChild::default(),
                open_top_icon: TemplateChild::default(),
                folder_icon_content: TemplateChild::default(),
                top_icon_content: TemplateChild::default(),
                image_view: TemplateChild::default(),
                generate_icon: TemplateChild::default(),
                folder_image_file: RefCell::new(None),
                top_image_file: RefCell::new(None),
            }
        }
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
            klass.install_action("app.open_folder_icon", None, move |win, _, a| {
            println!("{:#?}",a);
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
        let imp = self.imp();
        imp.generate_icon.set_sensitive(false);
        imp.toast_overlay.add_toast(adw::Toast::new("generated"));
        println!("{}",imp.folder_image_file.borrow().as_ref().unwrap().path_str());
        let mut base = image::open(imp.folder_image_file.borrow().as_ref().unwrap().path_str()).expect("kon bovenste file niet openen");
        let top_image = image::open(imp.top_image_file.borrow().as_ref().unwrap().path_str()).unwrap();
        imageops::overlay(&mut base, &top_image,0,0);
        base.save("/tmp/overlayed_image.png").unwrap();
       // let image1 = image::io::Reader::open(self.imp().folder_image_file.borrow().unwrap().path);
        //let texture = gdk::Texture::from_resource(&self.imp().folder_image_file.borrow().unwrap().path_str());
        imp.image_view.set_file(Some(&gio::File::for_path("/tmp/overlayed_image.png")));
        imp.generate_icon.set_sensitive(true);
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
                                self.imp().folder_icon_content.set_label(&self.get_file_name(x,
            														&self.imp().folder_image_file, Some(8),true));},
                        _ => {self.imp().open_top_icon.set_tooltip_text(Some("The currently set top image"));
                                self.imp().top_icon_content.set_icon_name("image-x-generic-symbolic");
                                self.imp().top_icon_content.set_label(&self.get_file_name(x,
                                									&self.imp().top_image_file,Some(8),true));},
                        }
                    },
            Err(y) => println!("{:#?}",y),
        };

    }

    fn get_file_name(&self, filename: gio::File, file: &RefCell<Option<File>>, slice: Option<usize>,show_extension: bool) -> String{
        file.replace(Some(File::new(filename)));
        let file = file.borrow().clone().unwrap();
        println!("{:#?}",file.name);
        match slice{
            Some(x) => {
                let mut substring = String::from(&file.name [..x/2]);
                substring.push_str("...");
                substring.push_str(&file.name[file.name.len()-(x/2)..]);
		        if show_extension{
		            substring.push_str(&file.extension);
		        }
                substring
            },
            None => String::from(format!("{}{}",&file.name,&file.extension))
        }

    }

    #[template_callback]
    fn handle_button_clicked() {
        // Set the label to "Hello World!" after the button has been clicked on
        println!("test");
    }
}

