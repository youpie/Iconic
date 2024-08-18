use crate::objects::file::File;
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gio::*;
use gtk::glib;
use image::*;
use log::*;
use std::env;
use std::path::PathBuf;
use std::error::Error;

use crate::GtkTestWindow;

impl GtkTestWindow {
    pub fn load_folder_path_from_settings(&self) {
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = win)]
            self,
            async move {
                let cache_file_name: &str = &win.imp().settings.string("folder-cache-name");
                let path = win.check_chache_icon(cache_file_name).await;
                win.load_folder_icon(&path.into_os_string().into_string().unwrap());
            }
        ));
    }

    pub async fn paste_from_clipboard(&self) {
        let clipboard = self.clipboard();
        let imp = self.imp();
        let thumbnail_size: i32 = imp.settings.get("thumbnail-size");

        match clipboard
            .read_future(&["image/svg+xml"], glib::Priority::DEFAULT)
            .await
        {
            Ok((stream, _mime)) => self.clipboard_load_svg(Some(stream)),
            Err(_) => match clipboard.read_texture_future().await {
                Ok(Some(texture)) => {
                    imp.stack.set_visible_child_name("stack_loading_page");

                    let png_texture = texture.save_to_tiff_bytes();
                    let image =
                        image::load_from_memory_with_format(&png_texture, image::ImageFormat::Tiff)
                            .unwrap();
                    imp.top_image_file
                        .lock()
                        .unwrap()
                        .replace(File::from_image(image, thumbnail_size));
                    self.check_icon_update();
                }
                Ok(None) => {
                    warn!("No texture found");
                }
                Err(err) => {
                    error!("Failed to paste texture {err}");
                }
            }, // no svg in clipboard
        };
    }

    pub fn clipboard_load_svg(&self, stream: Option<gio::InputStream>) {
        let imp = self.imp();

        let thumbnail_size: i32 = imp.settings.get("thumbnail-size");
        let svg_render_size: i32 = imp.settings.get("svg-render-size");
        let none_file: Option<&gio::File> = None;
        let none_cancelable: Option<&Cancellable> = None;
        let loader = rsvg::Loader::new()
            .read_stream(&stream.unwrap(), none_file, none_cancelable)
            .unwrap();
        let surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, svg_render_size, svg_render_size)
                .unwrap();
        let cr = cairo::Context::new(&surface).expect("Failed to create a cairo context");
        let renderer = rsvg::CairoRenderer::new(&loader);
        renderer
            .render_document(
                &cr,
                &cairo::Rectangle::new(
                    0.0,
                    0.0,
                    f64::from(svg_render_size),
                    f64::from(svg_render_size),
                ),
            )
            .unwrap();
        let cache_path = env::var("XDG_CACHE_HOME").unwrap();
        let clipboard_path = PathBuf::from(format!("{}/clipboard.png", cache_path));
        let mut stream = std::fs::File::create(&clipboard_path).unwrap();
        surface.write_to_png(&mut stream).unwrap();
        self.new_iconic_file_creation(None, Some(clipboard_path), svg_render_size, thumbnail_size, true);
        self.check_icon_update();
    }

    pub async fn open_dragged_file(&self, file: gio::File) {
        let imp = self.imp();
        let thumbnail_size: i32 = imp.settings.get("thumbnail-size");
        let svg_render_size: i32 = imp.settings.get("svg-render-size");
        debug!("{:#?}", file.path().value_type());

        let file_info =
            match file.query_info("standard::", FileQueryInfoFlags::NONE, Cancellable::NONE) {
                Ok(x) => x,
                Err(e) => {
                    self.show_error_popup(&e.to_string(), true, Some(Box::new(e)));
                    return;
                }
            };
        imp.stack.set_visible_child_name("stack_loading_page");
        debug!("file name: {:?}", file_info.name());
        let mime_type: Option<String> = match file_info.content_type() {
            Some(x) => {
                let sub_string: Vec<&str> = x.split("/").collect();
                Some(sub_string.first().unwrap().to_string())
            }
            None => None,
        };
        debug!("file type: {:?}", mime_type);
        match mime_type {
            Some(x) if x == String::from("image") => {
                let top_file_selected = self.top_or_bottom_popup().await;
                match top_file_selected {
                    Some(true) => self.new_iconic_file_creation(Some(file), None, svg_render_size, thumbnail_size, true),
                    Some(false) => self.new_iconic_file_creation(Some(file), None, svg_render_size, thumbnail_size, false),
                    _ => self.new_iconic_file_creation(Some(file), None, svg_render_size, thumbnail_size, true),
                };
            }
            _ => {
                self.show_error_popup(&gettext("Unsupported file type"), true, None);
            }
        }

        self.check_icon_update();
    }

    pub async fn save_file(&self) -> Result<bool, Box<dyn Error + '_>> {
        let imp = self.imp();
        if !imp.save_button.is_sensitive() {
            imp.toast_overlay
                .add_toast(adw::Toast::new("Can't save anything"));
            return Ok(false);
        };
        let file_name = format!(
            "folder-{}.png",
            imp.top_image_file.lock()?.as_ref().unwrap().name
        );
        let file_chooser = gtk::FileDialog::builder()
            .initial_name(file_name)
            .modal(true)
            .build();
        let file = file_chooser.save_future(Some(self)).await;
        match file {
            Ok(file) => {
                self.imp().stack.set_visible_child_name("stack_saving_page");
                imp.saved_file
                    .lock()
                    .expect("Could not get file")
                    .replace(file.clone());
                imp.image_saved.replace(true);
                imp.save_button.set_sensitive(false);
                let base_image = imp
                    .folder_image_file
                    .lock()?
                    .as_ref()
                    .unwrap()
                    .dynamic_image
                    .clone();
                let top_image = match self.imp().monochrome_switch.state() {
                    false => imp
                        .top_image_file
                        .lock()?
                        .as_ref()
                        .unwrap()
                        .dynamic_image
                        .clone(),
                    true => self.to_monochrome(
                        imp.top_image_file
                            .lock()?
                            .as_ref()
                            .unwrap()
                            .dynamic_image
                            .clone(),
                        imp.threshold_scale.value() as u8,
                        imp.monochrome_color.rgba(),
                    ),
                };
                let generated_image = self
                    .generate_image(base_image, top_image, imageops::FilterType::Gaussian)
                    .await;
                //let _ = gio::spawn_blocking(move || {
                    generated_image.save_with_format(file.path().unwrap(), ImageFormat::Png)?;
                //})
                //.await;
                self.imp().stack.set_visible_child_name("stack_main_page");
                imp.toast_overlay.add_toast(
                    adw::Toast::builder()
                        .button_label(gettext("Open Folder"))
                        .action_name("app.open_file_location")
                        .title(gettext("File Saved"))
                        .build(),
                );
            }
            Err(e) => {
                match e.message() {
                    "Dismissed by user" => {
                        imp.toast_overlay
                            .add_toast(adw::Toast::new("File not saved"));
                       },
                    _ => {
                        imp.image_saved.replace(false);
                        imp.save_button.set_sensitive(true);
                        return Err(Box::new(e));
                    }
                };
            }
        };
        Ok(true)
    }

    pub fn reset_bottom_icon(&self){
        self.imp().toast_overlay.add_toast(adw::Toast::new(&gettext("Icon reset")));
        self.load_folder_path_from_settings();
    }

    pub async fn load_top_icon(&self) {
        let imp = self.imp();
        match self.open_file_chooser_gtk().await {
            Some(x) => {
                self.load_top_file(x).await;
            }
            None => {
                imp.toast_overlay
                    .add_toast(adw::Toast::new(&gettext("Nothing selected")));
            }
        };
        self.check_icon_update();
    }

    pub async fn load_temp_folder_icon(&self) {
        let imp = self.imp();
        let thumbnail_size: i32 = imp.settings.get("thumbnail-size");
        let size: i32 = imp.settings.get("svg-render-size");

        match self.open_file_chooser_gtk().await {
            Some(x) => {
                self.new_iconic_file_creation(Some(x), None, size, thumbnail_size, false)
            }
            None => {
                imp.toast_overlay.add_toast(adw::Toast::new(&gettext("Nothing selected")));
            }
        };
    }

    pub fn load_folder_icon(&self, path: &str) {
        //let path1 = "/usr/share/icons/Adwaita/scalable/places/folder.svg";
        let size: i32 = self.imp().settings.get("thumbnail-size");
        self.new_iconic_file_creation(None, Some(PathBuf::from(path)), self.imp().settings.get("svg-render-size"), size, false);
    }

    pub async fn load_top_file(&self, filename: gio::File) {
        let imp = self.imp();
        imp.image_loading_spinner.set_spinning(true);
        if imp.stack.visible_child_name() == Some("stack_welcome_page".into()) {
            imp.stack.set_visible_child_name("stack_loading_page");
        }
        let svg_render_size: i32 = imp.settings.get("svg-render-size");
        let size: i32 = imp.settings.get("thumbnail-size");
        self.new_iconic_file_creation(Some(filename),None, svg_render_size, size, true);
        imp.image_loading_spinner.set_spinning(false);
    }

    pub fn new_iconic_file_creation(
        &self,
        file: Option<gio::File>,
        path: Option<PathBuf>,
        svg_render_size: i32,
        thumbnail_render_size: i32,
        change_top_icon: bool,
    ) {
        let imp = self.imp();
        let new_file = if let Some(file_temp) = file {
            match File::new(file_temp, svg_render_size, thumbnail_render_size) {
                Ok(x) => Some(x),
                Err(e) => {
                    self.show_error_popup(&e.to_string(), true, Some(e));
                    None
                }
            }
        }
        else if let Some(path_temp) = path{
            match File::from_path(path_temp, svg_render_size, thumbnail_render_size) {
                Ok(x) => Some(x),
                Err(e) => {
                    self.show_error_popup(&e.to_string(), true, Some(e));
                    None
                }
            }
        }
        else {
            self.show_error_popup(&gettext("No file or path found, this is probably not your fault."), true, None);
            None
        };
        match new_file {
            Some(file) => {
                match change_top_icon{
                    true => imp.top_image_file.lock().unwrap().replace(file),
                    false => imp.folder_image_file.lock().unwrap().replace(file),
                };
                self.check_icon_update();
            }
            None => {
                self.check_icon_update();
            }
        }
    }
}
