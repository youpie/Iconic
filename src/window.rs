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
use gtk::gdk_pixbuf::Pixbuf;
use adw::prelude::AlertDialogExt;
use adw::prelude::AlertDialogExtManual;
use std::sync::{Arc, Mutex};
use std::fs;
use resvg::tiny_skia::{Pixmap};
use resvg::usvg::{Tree, Options};

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
        pub image_view: TemplateChild<gtk::Picture>,
        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub x_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub y_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub size: TemplateChild<gtk::Scale>,
        #[template_child]
        pub top_icon_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub folder_icon_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub image_loading_spinner: TemplateChild<gtk::Spinner>,

        pub folder_image_file: Arc<Mutex<Option<File>>>,
        pub top_image_file: Arc<Mutex<Option<File>>>,
        pub file_created: RefCell<bool>,
        pub image_saved: RefCell<bool>,
        pub final_image: RefCell<Option<DynamicImage>>,
        pub signals: RefCell<Vec<glib::SignalHandlerId>>,
    }

    impl Default for GtkTestWindow {
        fn default() -> Self {
            Self{
                toolbar: TemplateChild::default(),
                header_bar: TemplateChild::default(),
                open_folder_icon: TemplateChild::default(),
                toast_overlay: TemplateChild::default(),
                open_top_icon: TemplateChild::default(),
                folder_icon_content: TemplateChild::default(),
                top_icon_content: TemplateChild::default(),
                image_view: TemplateChild::default(),
                save_button: TemplateChild::default(),
                folder_image_file: Arc::new(Mutex::new(None)),
                top_image_file: Arc::new(Mutex::new(None)),
                image_saved: RefCell::new(true),
                final_image: RefCell::new(None),
                file_created: RefCell::new(false),
                signals: RefCell::new(vec![]),
                x_scale: TemplateChild::default(),
                y_scale: TemplateChild::default(),
                size: TemplateChild::default(),
                top_icon_row: TemplateChild::default(),
                folder_icon_row: TemplateChild::default(),
                stack: TemplateChild::default(),
                image_loading_spinner: TemplateChild::default(),
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
                glib::spawn_future_local(clone!(@weak win => async move {
                    win.button_clicked().await;
                }));
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
            klass.install_action("app.save_button", None, move |win, _, _| {
                glib::spawn_future_local(clone!(@weak win => async move {
                    win.save_file().await;
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GtkTestWindow {}
    impl WidgetImpl for GtkTestWindow {}
    impl WindowImpl for GtkTestWindow {
        fn close_request(&self) -> glib::Propagation {
            if !self.image_saved.borrow().clone(){
                let window = self.obj();
                return match glib::MainContext::default()
                    .block_on(async move { window.confirm_save_changes().await })
                {
                    Ok(p) => p,
                    _ => {
                        glib::Propagation::Stop
                    }
                };
            }

            self.parent_close_request()
        }
    }
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
        let win = glib::Object::builder::<GtkTestWindow>()
            .property("application", application)
            .build();
        win.imp().save_button.set_sensitive(false);
        win.imp().x_scale.add_mark(0.0, gtk::PositionType::Top, None);
        win.imp().y_scale.add_mark(0.0, gtk::PositionType::Bottom, None);
        win.imp().stack.set_visible_child_name("stack_main_page");
        win.load_svg("/home/youpie/Afbeeldingen/folder.svg");
        win
    }

    fn setup_update (&self){

        self.imp().x_scale.connect_value_changed(clone!(@weak self as this => move |_| {
        glib::spawn_future_local(clone!(@weak this => async move {
                this.button_clicked().await;}));
            }));
        self.imp().y_scale.connect_value_changed(clone!(@weak self as this => move |_| {
        glib::spawn_future_local(clone!(@weak this => async move {
                this.button_clicked().await;}));
            }));
        self.imp().size.connect_value_changed(clone!(@weak self as this => move |_| {
        glib::spawn_future_local(clone!(@weak this => async move {
                this.button_clicked().await;}));
            }));
    }

    async fn confirm_save_changes(&self) -> Result<glib::Propagation, ()> {
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
                Ok(glib::Propagation::Stop)
            }
            RESPONSE_DISCARD => Ok(glib::Propagation::Proceed),
            RESPONSE_SAVE => {
                match self.save_file().await{
                    true => Ok(glib::Propagation::Proceed),
                    false => Ok(glib::Propagation::Stop)
                }

            }
            _ => unreachable!(),
        }
    }


    async fn button_clicked(&self) {
        let imp = self.imp();
        let base = imp.folder_image_file.lock().unwrap().as_ref().unwrap().thumbnail.clone();
        let top_image = imp.top_image_file.lock().unwrap().as_ref().unwrap().thumbnail.clone();
        let texture = GtkTestWindow::dynamic_image_to_texture(&self.generate_image(base, top_image,imageops::FilterType::Nearest).await);
        imp.image_view.set_paintable(Some(&texture));
    }

    async fn save_file(&self) -> bool{
        let imp = self.imp();
        let file_name = "folder.png";
        let file_chooser = gtk::FileDialog::builder()
            .initial_name(file_name)
            .modal(true)
            .build();
        let file = file_chooser.save_future(Some(self)).await;
        match file {
            Ok(file) => {
                self.imp().stack.set_visible_child_name("stack_saving_page");
                imp.image_saved.replace(true);
                let base_image = imp.folder_image_file.lock().unwrap().as_ref().unwrap().dynamic_image.clone();
                let top_image = imp.top_image_file.lock().unwrap().as_ref().unwrap().dynamic_image.clone();
                let generated_image = self.generate_image(base_image, top_image,imageops::FilterType::Gaussian).await;
                let _ = gio::spawn_blocking(move ||{
                    let _ = generated_image.save(file.path().unwrap());
                }).await;
                self.imp().stack.set_visible_child_name("stack_main_page");
                imp.toast_overlay.add_toast(adw::Toast::new("saved file"));
            }
            Err(_) => {
                imp.toast_overlay.add_toast(adw::Toast::new("File not saved"));
                return false;
            }
        };
        true
    }


    async fn generate_image (&self, base_image: image::DynamicImage, top_image: image::DynamicImage, filter: imageops::FilterType) -> DynamicImage{
        let button = self.imp();
        self.imp().save_button.set_sensitive(true);
        button.image_saved.replace(false);
        let (tx_texture, rx_texture) = async_channel::bounded(1);
        let tx_texture1 = tx_texture.clone();
        let coordinates = ((button.x_scale.value()+50.0) as i64,(button.y_scale.value()+50.0) as i64);
        let scale: f32 = button.size.value() as f32;
        gio::spawn_blocking(move ||{
            let mut base = base_image;
            let top = top_image;
            let base_dimension: (i64,i64)  = ((base.dimensions().0).into(),(base.dimensions().1).into());
            let top = GtkTestWindow::resize_image(top,base.dimensions(),scale, filter);
            let top_dimension: (i64,i64) = ((top.dimensions().0/2).into(),(top.dimensions().1/2).into());
            let final_coordinates: (i64,i64) = (((base_dimension.0*coordinates.0)/100)-top_dimension.0,((base_dimension.1*coordinates.1)/100)-top_dimension.1);
            imageops::overlay(&mut base, &top,final_coordinates.0.into(),final_coordinates.1.into());
            tx_texture1.send_blocking(base)
        });

        let texture = glib::spawn_future_local(clone!(@weak-allow-none button => async move {
            rx_texture.recv().await.unwrap()
        }));
        let image = texture.await.unwrap();
        button.final_image.replace(Some(image.clone()));
        image
    }

    fn resize_image (image: DynamicImage, dimensions: (u32,u32), slider_position: f32, filter: imageops::FilterType) -> DynamicImage{
        let width: f32 = dimensions.0 as f32;
        let height: f32 = dimensions.1 as f32;
        let scale_factor: f32 = (slider_position + 10.0) / 10.0;
        let new_width: u32 = (width/scale_factor) as u32;
        let new_height: u32 = (height/scale_factor) as u32;
        image.resize(new_width, new_height, filter)
    }

    pub async fn open_file_chooser_gtk(&self,what_button:usize) {
        let imp = self.imp();
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
                        0 => {imp.open_folder_icon.set_tooltip_text(Some("The currently set folder image"));
                                imp.folder_icon_content.set_icon_name("image-x-generic-symbolic");
                                imp.folder_icon_row.set_property("subtitle",&self.get_file_name(x,
            														&imp.folder_image_file).await);},
                        _ => {imp.open_top_icon.set_tooltip_text(Some("The currently set top image"));
                                imp.top_icon_content.set_icon_name("image-x-generic-symbolic");
                                imp.top_icon_row.set_property("subtitle",&self.get_file_name(x,
            														&imp.top_image_file).await);},
                        }
                    },
            Err(y) => println!("{:#?}",y),
        };
        if imp.top_image_file.lock().unwrap().as_ref() != None && imp.folder_image_file.lock().unwrap().as_ref() != None {
            self.setup_update();
            self.button_clicked().await;
        }
        else if imp.folder_image_file.lock().unwrap().as_ref() != None {
            imp.image_view.set_paintable(Some(&GtkTestWindow::dynamic_image_to_texture(&imp.folder_image_file.lock().unwrap().as_ref().unwrap().thumbnail)));
        }
    }

    async fn get_file_name(&self, filename: gio::File, file: &Arc<Mutex<Option<File>>>) -> String{
        self.imp().image_loading_spinner.set_spinning(true);
        let _ = gio::spawn_blocking(clone!(@weak file => move ||{
            file.lock().expect("oh noes").replace(File::new(filename));
        })).await;
        let file = file.lock().unwrap().clone().unwrap();
        self.imp().image_loading_spinner.set_spinning(false);
        println!("{:#?}",file.name);
        format!("{}{}",file.name,file.extension)
    }

    fn dynamic_image_to_texture(dynamic_image: &DynamicImage) -> gdk::Texture {
        let rgba_image = dynamic_image.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        let pixels = rgba_image.into_raw(); // Get the raw pixel data
        // Create Pixbuf from raw pixel data
        let pixbuf = Pixbuf::from_bytes(
            &glib::Bytes::from(&pixels),
            gtk::gdk_pixbuf::Colorspace::Rgb,
            true,  // has_alpha
            8,     // bits_per_sample
            width as i32,
            height as i32,
            width as i32 * 4, // rowstride
        );
        gdk::Texture::for_pixbuf(&pixbuf)
    }

    fn load_svg(&self,path: &str) {
        // Load the SVG file content
        let svg_data = fs::read(path).expect("Failed to read SVG file");

        // Create an SVG tree
        let opt = Options::default();
        let rtree = Tree::from_data(&svg_data, &opt).expect("Failed to parse SVG data");

        // Specify the output dimensions (you can adjust these as needed)
        let width = rtree.size().width();
        let height = rtree.size().height();

        // Desired dimensions
        let desired_width = 1024.0;
        let desired_height = 1024.0;

        // Calculate the scale factor
        let scale_x = desired_width / width;
        let scale_y = desired_height / height;
        let scale = scale_x.min(scale_y); // Maintain aspect ratio

        // Create a Pixmap to render into
        let mut pixmap = Pixmap::new(desired_width as u32, desired_height as u32).expect("Failed to create Pixmap");

        // Render the SVG tree to the Pixmap
        let _ = resvg::render(
            &rtree,
            usvg::Transform::from_scale(scale, scale),
            &mut pixmap.as_mut()
        );

        self.imp().folder_image_file.lock().unwrap().replace(File::from_dynamicimage(self.pixmap_to_image(&pixmap)));
        self.imp().image_view.set_paintable(Some(&GtkTestWindow::dynamic_image_to_texture(&self.imp().folder_image_file.lock().unwrap().as_ref().unwrap().thumbnail)));
    }

    fn pixmap_to_image(&self, pixmap: &Pixmap) -> DynamicImage {
        // Create an empty RgbaImage with the same dimensions as the Pixmap.
        let mut img = RgbaImage::new(pixmap.width(), pixmap.height());

        // Iterate over each pixel in the Pixmap.
        for y in 0..pixmap.height() {
            for x in 0..pixmap.width() {
                // Get the pixel at (x, y). `pixel` returns an Option, we unwrap it safely because we know (x, y) is valid.
                if let Some(pixel) = pixmap.pixel(x, y) {
                    // Copy the pixel's RGBA data into the RgbaImage.
                    img.put_pixel(x, y, Rgba([pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]));
                }
            }
        }

        // Convert the RgbaImage to a DynamicImage.
        DynamicImage::ImageRgba8(img)
    }

}


