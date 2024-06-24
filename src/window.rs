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
use std::path::PathBuf;
use std::env;
use std::fs;

use crate::config::{APP_ID, PROFILE};

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
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub open_top_icon: TemplateChild<gtk::Button>,
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
        pub scale_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub image_loading_spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub monochrome_action_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub monochrome_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub threshold_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub monochrome_color: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub reset_color: TemplateChild<gtk::Button>,

        pub folder_image_file: Arc<Mutex<Option<File>>>,
        pub top_image_file: Arc<Mutex<Option<File>>>,
        pub file_created: RefCell<bool>,
        pub image_saved: RefCell<bool>,
        pub final_image: RefCell<Option<DynamicImage>>,
        pub signals: RefCell<Vec<glib::SignalHandlerId>>,
        pub settings: gio::Settings,
        pub count: RefCell<i32>,
    }

    impl Default for GtkTestWindow {
        fn default() -> Self {
            Self{
                toolbar: TemplateChild::default(),
                header_bar: TemplateChild::default(),
                toast_overlay: TemplateChild::default(),
                open_top_icon: TemplateChild::default(),
                image_view: TemplateChild::default(),
                save_button: TemplateChild::default(),
                threshold_scale: TemplateChild::default(),
                reset_color: TemplateChild::default(),
                monochrome_action_row: TemplateChild::default(),
                monochrome_color: TemplateChild::default(),
                scale_row: TemplateChild::default(),
                monochrome_switch: TemplateChild::default(),
                folder_image_file: Arc::new(Mutex::new(None)),
                top_image_file: Arc::new(Mutex::new(None)),
                image_saved: RefCell::new(true),
                final_image: RefCell::new(None),
                file_created: RefCell::new(false),
                signals: RefCell::new(vec![]),
                x_scale: TemplateChild::default(),
                y_scale: TemplateChild::default(),
                size: TemplateChild::default(),
                stack: TemplateChild::default(),
                image_loading_spinner: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                count: RefCell::new(0),
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
                    win.render_to_screen().await;
                }));
            });
            klass.install_action("app.open_top_icon", None, move |win, _, _| {
                glib::spawn_future_local(clone!(@weak win => async move {
                    win.load_top_icon().await;
                }));
            });
            klass.install_action("app.save_button", None, move |win, _, _| {
                glib::spawn_future_local(clone!(@weak win => async move {
                    win.save_file().await;
                }));
            });
            klass.install_action("app.monochrome_switch", None, move |win, _, _| {
                win.enable_monochrome_expand();
            });
            klass.install_action("app.reset_color", None, move |win, _, _| {
                win.reset_colors();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GtkTestWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }
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
        win.imp().stack.set_visible_child_name("stack_welcome_page");
        win.imp().monochrome_action_row.set_property("enable_expansion",false);
        win.imp().reset_color.set_visible(false);
        glib::spawn_future_local(glib::clone!(@weak win as window => async move {
            let cache_file_name: &str = &window.imp().settings.string("folder-cache-name");
            let path = window.check_chache_icon(cache_file_name).await;
            window.load_folder_icon(&path.into_os_string().into_string().unwrap());
        }));

        win.setup_settings();
        win
    }

    fn setup_settings (&self){
        let update_folder = glib::clone!(@weak self as window => move |_: &gio::Settings, setting:&str| {
             let path: &str = &window.imp().settings.string(setting);

             window.load_folder_icon(path);
        });

        let resize_folder = glib::clone!(@weak self as win => move |_: &gio::Settings, _:&str| {
            let path: &str = &win.imp().settings.string("folder-svg-path");
            win.load_folder_icon(path);
        });

        let reload_thumbnails = glib::clone!(@weak self as win => move |_: &gio::Settings, _:&str| {
            let path: &str = &win.imp().settings.string("folder-svg-path");
            win.load_folder_icon(path);
            win.imp().reset_color.set_visible(true);
        });

        self.imp().settings.connect_changed(Some("folder-svg-path"), update_folder.clone());
        self.imp().settings.connect_changed(Some("svg-render-size"), resize_folder.clone());
        self.imp().settings.connect_changed(Some("thumbnail-size"), reload_thumbnails.clone());
    }

    fn setup_update (&self){
        self.imp().save_button.set_sensitive(true);
        self.imp().x_scale.connect_value_changed(clone!(@weak self as this => move |_| {
        glib::spawn_future_local(clone!(@weak this => async move {
                this.render_to_screen().await;}));
            }));
        self.imp().y_scale.connect_value_changed(clone!(@weak self as this => move |_| {
        glib::spawn_future_local(clone!(@weak this => async move {
                this.render_to_screen().await;}));
            }));
        self.imp().size.connect_value_changed(clone!(@weak self as this => move |_| {
        glib::spawn_future_local(clone!(@weak this => async move {
                this.render_to_screen().await;}));
            }));
        self.imp().threshold_scale.connect_value_changed(clone!(@weak self as this => move |_| {
        glib::spawn_future_local(clone!(@weak this => async move {
                this.render_to_screen().await;}));
            }));
        self.imp().monochrome_color.connect_rgba_notify(clone!(@weak self as this => move |_| {
        glib::spawn_future_local(clone!(@weak this => async move {
                this.render_to_screen().await;
                this.imp().reset_color.set_visible(true);}));
            }));
    }

    fn reset_colors(&self){
        let imp = self.imp();
        imp.monochrome_color.set_rgba(&gdk::RGBA::new(67.0,141.0,230.0,0.0));
        self.check_icon_update();
        imp.reset_color.set_visible(false);
    }

    async fn check_chache_icon(&self, file_name: &str) -> PathBuf{
        let imp = self.imp();
        let icon_path = PathBuf::from(&imp.settings.string("folder-svg-path"));
        let cache_path = env::var("XDG_CACHE_HOME").unwrap();
        println!("{}",cache_path);
        let folder_icon_cache_path = PathBuf::from(format!("{}/{}",cache_path,file_name));
        if folder_icon_cache_path.exists() {
            println!("File found in cache at: {:?}",folder_icon_cache_path);
            return folder_icon_cache_path;
        }
        else if icon_path.exists() {
            println!("File not found in cache, copying to: {:?}",folder_icon_cache_path);
            return self.copy_folder_image(icon_path).0;
        }
        else {
            println!("File not found AT ALL");
            let new_path = match self.open_file_chooser_gtk().await{
                Some(x) => x.path().unwrap().into_os_string().into_string().unwrap(),
                None => {adw::prelude::ActionGroupExt::activate_action(self, "app.quit", None);
                        String::from("")}
            };
            imp.settings.set_string("folder-svg-path", &new_path).unwrap();
            let cached_file_name = self.copy_folder_image(PathBuf::from(new_path)).1;
            imp.settings.set_string("folder-cache-name", &cached_file_name).unwrap();
            let cache_file_name = &imp.settings.string("folder-cache-name");
            let cache_path = env::var("XDG_CACHE_HOME").unwrap();
            let folder_icon_cache_path = PathBuf::from(format!("{}/{}",cache_path,cache_file_name));
            return PathBuf::from(folder_icon_cache_path);
        }
    }

    fn copy_folder_image(&self, original_path: PathBuf) -> (PathBuf, String) {
        let cache_dir = env::var("XDG_CACHE_HOME").expect("$HOME is not set");
        let file_name = format!("folder.{}",original_path.extension().unwrap().to_str().unwrap());
        self.imp().settings.set("folder-cache-name",file_name.clone()).unwrap();
        let mut cache_path = PathBuf::from(cache_dir);
        cache_path.push(file_name.clone());
        println!("{:?}",cache_path);
        fs::copy(original_path,cache_path.clone()).unwrap();
        (cache_path,file_name)
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


    async fn render_to_screen(&self) {
        let imp = self.imp();
        let base = imp.folder_image_file.lock().unwrap().as_ref().unwrap().thumbnail.clone();
        let top_image =match self.imp().monochrome_switch.state(){
            false => {imp.top_image_file.lock().unwrap().as_ref().unwrap().thumbnail.clone()},
            true => {self.to_monochrome(imp.top_image_file.lock().unwrap().as_ref().unwrap().thumbnail.clone(), imp.threshold_scale.value() as u8, imp.monochrome_color.rgba())}
        };
        let texture = self.dynamic_image_to_texture(&self.generate_image(base, top_image,imageops::FilterType::Nearest).await);
        imp.image_view.set_paintable(Some(&texture));
    }

    pub async fn save_file(&self) -> bool{
        let imp = self.imp();
        if !imp.save_button.is_sensitive() {
            imp.toast_overlay.add_toast(adw::Toast::new("Can't save anything"));
            return false;
        };
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
                let top_image = match self.imp().monochrome_switch.state(){
                    false => {imp.top_image_file.lock().unwrap().as_ref().unwrap().dynamic_image.clone()},
                    true => {self.to_monochrome(imp.top_image_file.lock().unwrap().as_ref().unwrap().dynamic_image.clone(), imp.threshold_scale.value() as u8, imp.monochrome_color.rgba())}
                };
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
        button.stack.set_visible_child_name("stack_main_page");
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

    async fn load_top_icon (&self){
        let imp = self.imp();
		match self.open_file_chooser_gtk().await {
            Some(x) => {self.get_file_name(x,&imp.top_image_file).await;}
            None => {imp.toast_overlay.add_toast(adw::Toast::new("Nothing selected"));}
        };
        self.check_icon_update();
    }

    fn load_folder_icon (&self, path: &str){
        let path1 = "/usr/share/icons/Adwaita/scalable/places/folder.svg";
        let size: i32 = self.imp().settings.get("thumbnail-size");
        self.imp().folder_image_file.lock().unwrap().replace(File::from_path(path,self.imp().settings.get("svg-render-size"),size));
        self.check_icon_update();
    }

    fn check_icon_update(&self){
        let imp = self.imp();
        if imp.top_image_file.lock().unwrap().as_ref() != None && imp.folder_image_file.lock().unwrap().as_ref() != None {
            self.setup_update();
            glib::spawn_future_local(glib::clone!(@weak self as window => async move {
                window.render_to_screen().await;
            }));
        }
        else if imp.folder_image_file.lock().unwrap().as_ref() != None {
            imp.image_view.set_paintable(Some(&self.dynamic_image_to_texture(&imp.folder_image_file.lock().unwrap().as_ref().unwrap().thumbnail)));
        }
    }

    pub async fn open_file_chooser_gtk(&self) -> Option<gio::File> {
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
                        Some(x)},
            Err(y) => {println!("{:#?}",y);
                        None},
        }

    }

    fn to_monochrome(&self, image: DynamicImage,threshold: u8, color: gdk::RGBA) -> DynamicImage {
        // Convert the image to RGBA8
        let rgba_img = image.to_rgba8();
        // Define a threshold value
        let threshold = threshold; // Adjust the threshold value as needed

        // Create a new image buffer for the monochrome image
        let mut mono_img: RgbaImage = ImageBuffer::new(rgba_img.width(), rgba_img.height());

        // Apply the threshold to create a black and white image, keeping the alpha channel
        for (x, y, pixel) in rgba_img.enumerate_pixels() {
            let rgba = pixel.0;
            let luma = 0.299 * rgba[0] as f32 + 0.587 * rgba[1] as f32 + 0.114 * rgba[2] as f32;
            let mono_pixel = if luma > threshold as f32 {
                Rgba([(color.red()*255.0) as u8, (color.green()*255.0) as u8, (color.blue()*255.0) as u8, rgba[3]]) // White with original alpha
            } else {
                Rgba([0u8, 0u8, 0u8, 0u8])       // Black with original alpha
            };
            mono_img.put_pixel(x, y, mono_pixel);
        }

        // Convert the monochrome RgbaImage to DynamicImage
        DynamicImage::ImageRgba8(mono_img)
    }

    async fn get_file_name(&self, filename: gio::File, file: &Arc<Mutex<Option<File>>>) -> String{
        let imp = self.imp();
        imp.image_loading_spinner.set_spinning(true);
        if imp.stack.visible_child_name() == Some("stack_welcome_page".into()) {
            imp.stack.set_visible_child_name("stack_loading_page");
        }
        let svg_render_size = imp.settings.get("svg-render-size");
        let size: i32 = imp.settings.get("thumbnail-size");
        let _ = gio::spawn_blocking(clone!(@weak file => move ||{
            file.lock().expect("Could not get file").replace(File::new(filename,svg_render_size,size));
        })).await;
        let file = file.lock().unwrap().clone().unwrap();
        imp.image_loading_spinner.set_spinning(false);
        format!("{}{}",file.name,file.extension)
    }

    fn dynamic_image_to_texture(&self, dynamic_image: &DynamicImage) -> gdk::Texture {
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

    fn enable_monochrome_expand(&self){
        let switch_state = self.imp().monochrome_switch.state();
        match switch_state{
            false => {self.imp().monochrome_action_row.set_property("enable_expansion",true);},
            true => {self.imp().monochrome_action_row.set_property("enable_expansion",false);}
        };
        self.check_icon_update();
    }
}


