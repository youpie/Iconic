use crate::objects::errors::{IntoResult, show_error_popup};
use crate::objects::file::File;
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gio::*;
use gtk::glib;
use image::*;
use log::*;
use std::env;
use std::error::Error;
use std::path::PathBuf;

use crate::{GenResult, GtkTestWindow, RUNTIME};

impl GtkTestWindow {
    pub fn load_folder_path_from_settings(&self) {
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = win)]
            self,
            async move {
                let imp = win.imp();
                let path;
                imp.temp_image_loaded.replace(false);
                if imp.settings.boolean("manual-bottom-image-selection") {
                    let cache_file_name: &str = &win.imp().settings.string("folder-cache-name");
                    path = win.check_chache_icon(cache_file_name).await;
                } else if imp.settings.string("selected-accent-color") == "Custom" {
                    path = win.create_custom_folder_color().await;
                } else {
                    path = win.load_built_in_bottom_icon().await;
                }
                if !imp.reset_color.is_visible() {
                    win.reset_colors();
                }
                info!("Loading path: {:?}", &path);
                win.load_folder_icon(&path.into_os_string().into_string().unwrap())
                    .await;
            }
        ));
    }

    async fn create_custom_folder_color(&self) -> PathBuf {
        let imp = self.imp();
        info!("Creating custom folder colors");
        let custom_primary_color = imp.settings.string("primary-folder-color");
        let custom_secondary_color = imp.settings.string("secondary-folder-color");
        let folder_svg_file =
            std::fs::read_to_string("/app/share/folder_icon/folders/folder_Custom.svg").unwrap();
        let folder_svg_lines = folder_svg_file.lines();
        let new_custom_folder: String = folder_svg_lines
            .map(|row| {
                let row_clone = row.to_string();
                let row_clone = row_clone.replace("a4caee", &custom_primary_color);
                let mut row_clone = row_clone.replace("438de6", &custom_secondary_color);
                row_clone.push_str("\n");
                row_clone
            })
            .collect();
        let new_custom_folder_bytes = new_custom_folder.as_bytes().to_owned();
        let mut cache_location = self.get_cache_path();
        cache_location.push("custom_folder.svg");
        let cache_location_clone = cache_location.clone();
        RUNTIME
            .spawn_blocking(move || {
                let _ = std::fs::write(&cache_location_clone, new_custom_folder_bytes);
            })
            .await
            .unwrap();
        cache_location
    }

    pub async fn load_built_in_bottom_icon(&self) -> PathBuf {
        let imp = self.imp();
        let current_set_accent_color = imp.settings.string("selected-accent-color");
        let folder_color_name = match current_set_accent_color.as_str() {
            "None" => self.get_accent_color_and_show_dialog(),
            x => x.to_string(),
        };
        let folder_path = PathBuf::from(format!(
            "/app/share/folder_icon/folders/folder_{}.svg",
            folder_color_name
        ));
        folder_path
    }

    pub async fn paste_from_clipboard(&self) {
        let clipboard = self.clipboard();
        let imp = self.imp();
        let thumbnail_size: i32 = imp.settings.get("thumbnail-size");

        match clipboard
            .read_future(&["image/svg+xml"], glib::Priority::DEFAULT)
            .await
        {
            Ok((stream, _mime)) => self.clipboard_load_svg(Some(stream)).await,
            Err(_) => match clipboard.read_texture_future().await {
                Ok(Some(texture)) => {
                    let top_file_selected = self.top_or_bottom_popup().await;
                    imp.stack.set_visible_child_name("stack_loading_page");

                    let png_texture = texture.save_to_tiff_bytes();
                    let image =
                        image::load_from_memory_with_format(&png_texture, image::ImageFormat::Tiff)
                            .unwrap();
                    match top_file_selected {
                        Some(true) => {
                            let iconic_file = RUNTIME
                                .spawn_blocking(move || {
                                    File::from_image(image, thumbnail_size, "pasted")
                                })
                                .await
                                .unwrap();
                            match self.store_top_image_in_cache(&iconic_file, None) {
                                Err(x) => {
                                    show_error_popup(&self, &x.to_string(), true, None);
                                }
                                _ => (),
                            };
                            imp.top_image_file.lock().unwrap().replace(iconic_file);
                        }
                        _ => {
                            imp.temp_image_loaded.replace(true);
                            imp.bottom_image_file.lock().unwrap().replace(
                                RUNTIME
                                    .spawn_blocking(move || {
                                        File::from_image(image.clone(), thumbnail_size, "pasted")
                                    })
                                    .await
                                    .unwrap(),
                            );
                        }
                    }
                    self.check_icon_update();
                }
                Ok(None) => {
                    warn!("No texture found");
                    imp.toast_overlay
                        .add_toast(adw::Toast::new(&gettext("No texture found")));
                }
                Err(err) => {
                    error!("Failed to paste texture {err}");
                    imp.toast_overlay
                        .add_toast(adw::Toast::new(&gettext("No texture found")));
                }
            }, // no svg in clipboard
        };
    }

    pub async fn clipboard_load_svg(&self, stream: Option<gio::InputStream>) {
        let imp = self.imp();
        let top_file = match self.top_or_bottom_popup().await {
            Some(true) => true,
            _ => false,
        };
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
        self.new_iconic_file_creation(
            None,
            Some(clipboard_path),
            svg_render_size,
            thumbnail_size,
            top_file,
        )
        .await;
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
                    show_error_popup(&self, &e.to_string(), true, Some(Box::new(e)));
                    return;
                }
            };

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
                imp.stack.set_visible_child_name("stack_loading_page");
                match top_file_selected {
                    Some(true) => {
                        self.new_iconic_file_creation(
                            Some(file),
                            None,
                            svg_render_size,
                            thumbnail_size,
                            true,
                        )
                        .await;
                    }
                    Some(false) => {
                        imp.temp_image_loaded.replace(true);
                        imp.stack.set_visible_child_name("stack_main_page");
                        self.new_iconic_file_creation(
                            Some(file),
                            None,
                            svg_render_size,
                            thumbnail_size,
                            false,
                        )
                        .await;
                    }
                    _ => (),
                };
            }
            _ => {
                show_error_popup(&self, &gettext("Unsupported file type"), true, None);
            }
        }
    }

    pub async fn open_save_file_dialog(&self) -> Result<bool, Box<dyn Error + '_>> {
        let imp = self.imp();
        if !imp.save_button.is_sensitive() {
            imp.toast_overlay
                .add_toast(adw::Toast::new("Can't save anything"));
            return Ok(false);
        };
        let file_name = format!(
            "folder-{}.png",
            imp.top_image_file
                .lock()?
                .as_ref()
                .into_reason_result("No top image found")?
                .filename
        );
        let file_chooser = gtk::FileDialog::builder()
            .initial_name(file_name)
            .modal(true)
            .build();
        self.imp().stack.set_visible_child_name("stack_saving_page");
        match file_chooser.save_future(Some(self)).await {
            Ok(file) => {
                let saved_file = self
                    .save_file(file, imp.monochrome_switch.is_active(), None)
                    .await?;
                self.imp().stack.set_visible_child_name("stack_main_page");
                imp.toast_overlay.add_toast(
                    adw::Toast::builder()
                        .button_label(gettext("Open Folder"))
                        .action_name("app.open_file_location")
                        .title(gettext("File Saved"))
                        .build(),
                );
                saved_file
            }
            Err(file_chooser_error) => {
                self.imp().stack.set_visible_child_name("stack_main_page");
                match file_chooser_error
                    .kind()
                    .into_reason_result("Unknown file picker error")?
                {
                    gtk::DialogError::Dismissed => {
                        error!("{:?}", file_chooser_error);
                        imp.toast_overlay
                            .add_toast(adw::Toast::new("File not saved"));
                        return Ok(false);
                    }
                    _ => {
                        return Err(Box::new(file_chooser_error));
                    }
                };
            }
        };
        Ok(true)
    }

    pub async fn copy_folder_image_to_cache(
        &self,
        original_path: &PathBuf,
        cache_dir: &PathBuf,
    ) -> GenResult<(PathBuf, String)> {
        let file_name = format!(
            "folder.{}",
            original_path
                .extension()
                .into_reason_result("Failed to get folder file extension")?
                .to_str()
                .into_result()?
        );
        self.imp()
            .settings
            .set("folder-cache-name", file_name.clone())
            .unwrap();
        let cache_path = cache_dir.join(file_name.clone());
        std::fs::copy(original_path, cache_path.clone())?;
        //let test = RUNTIME.spawn_blocking(move || true).await;
        Ok((cache_path, file_name))
    }

    pub async fn save_file(
        &self,
        file: gio::File,
        use_monochrome: bool,
        manual_monochrome_values: Option<(u8, gtk::gdk::RGBA)>,
    ) -> Result<bool, Box<dyn Error + '_>> {
        let imp = self.imp();
        self.image_save_sensitive(false);
        imp.saved_file.lock()?.replace(file.clone());
        let base_image = imp
            .bottom_image_file
            .lock()?
            .as_ref()
            .into_reason_result("No bottom image found")?
            .dynamic_image
            .clone();
        let mut top_image = imp
            .top_image_file
            .lock()?
            .as_ref()
            .into_reason_result("No top image found")?
            .thumbnail
            .clone();
        if use_monochrome {
            let (monochrome_threshold, monochrome_color) = match manual_monochrome_values {
                Some((threshold, color)) => (threshold, color),
                None => (
                    imp.threshold_scale.value() as u8,
                    imp.monochrome_color.rgba(),
                ),
            };
            top_image = self.to_monochrome(top_image, monochrome_threshold, monochrome_color);
        }
        let generated_image = self
            .generate_image(
                base_image,
                top_image,
                imageops::FilterType::Gaussian,
                imp.x_scale.value(),
                imp.y_scale.value(),
                imp.size.value(),
            )
            .await;
        let _ = RUNTIME
            .spawn_blocking(move || {
                generated_image.save_with_format(file.path().unwrap(), ImageFormat::Png)
            })
            .await?;
        Ok(true)
    }

    pub fn reset_bottom_icon(&self) {
        self.imp()
            .toast_overlay
            .add_toast(adw::Toast::new(&gettext("Icon reset")));
        self.load_folder_path_from_settings();
    }

    pub async fn load_top_icon(&self) {
        let imp = self.imp();
        imp.image_loading_spinner.set_visible(true);
        match self.open_file_chooser().await {
            Some(x) => {
                self.load_top_file(x).await;
            }
            None => {
                imp.toast_overlay
                    .add_toast(adw::Toast::new(&gettext("Nothing selected")));
            }
        };
        imp.image_loading_spinner.set_visible(false);
    }

    pub async fn load_temp_folder_icon(&self) {
        let imp = self.imp();
        let thumbnail_size: i32 = imp.settings.get("thumbnail-size");
        let size: i32 = imp.settings.get("svg-render-size");
        match self.open_file_chooser().await {
            Some(x) => {
                imp.temp_image_loaded.replace(true);
                imp.stack.set_visible_child_name("stack_main_page");
                self.new_iconic_file_creation(Some(x), None, size, thumbnail_size, false)
                    .await;
            }
            None => {
                imp.toast_overlay
                    .add_toast(adw::Toast::new(&gettext("Nothing selected")));
            }
        };
    }

    pub async fn load_folder_icon(&self, path: &str) {
        //let path1 = "/usr/share/icons/Adwaita/scalable/places/folder.svg";
        let size: i32 = self.imp().settings.get("thumbnail-size");
        self.new_iconic_file_creation(
            None,
            Some(PathBuf::from(path)),
            self.imp().settings.get("svg-render-size"),
            size,
            false,
        )
        .await;
    }

    pub async fn load_top_file(&self, filename: gio::File) {
        let imp = self.imp();
        if imp.stack.visible_child_name() == Some("stack_welcome_page".into()) {
            imp.stack.set_visible_child_name("stack_loading_page");
        }
        let svg_render_size: i32 = imp.settings.get("svg-render-size");
        let size: i32 = imp.settings.get("thumbnail-size");
        self.new_iconic_file_creation(Some(filename), None, svg_render_size, size, true)
            .await;
    }

    // Creates a new folder_icon::File from a gio::file, path or dynamicimage.
    // Will show an error if none are provided
    // TODO rewrite this, it is REALLY confusing
    pub async fn new_iconic_file_creation(
        &self,
        file: Option<gio::File>,
        path: Option<PathBuf>,
        svg_render_size: i32,
        thumbnail_render_size: i32,
        change_top_icon: bool,
    ) -> Option<File> {
        let imp = self.imp();
        let new_file = if let Some(file_temp) = file {
            let file_temp_clone = file_temp.clone();
            let iconic_file = match RUNTIME
                .spawn_blocking(move || {
                    File::new(file_temp_clone, svg_render_size, thumbnail_render_size)
                        .map_err(|err| err.to_string())
                })
                .await
                .unwrap()
            {
                Ok(x) => x,
                Err(e) => {
                    show_error_popup(&self, &e.to_string(), true, None);
                    return None;
                }
            };
            if change_top_icon {
                match self.store_top_image_in_cache(&iconic_file, Some(&file_temp)) {
                    Err(x) => {
                        show_error_popup(&self, "", true, Some(x));
                    }
                    _ => (),
                };
            }
            Some(iconic_file)
        } else if let Some(path_temp) = path {
            let file_temp = gio::File::for_path(path_temp);
            let file_temp_clone = file_temp.clone();
            let iconic_file = match RUNTIME
                .spawn_blocking(move || {
                    File::new(file_temp_clone, svg_render_size, thumbnail_render_size)
                        .map_err(|err| err.to_string())
                })
                .await
                .unwrap()
            {
                Ok(x) => x,
                Err(e) => {
                    show_error_popup(&self, &e.to_string(), true, None);
                    return None;
                }
            };
            if change_top_icon {
                match self.store_top_image_in_cache(&iconic_file, Some(&file_temp)) {
                    Err(x) => {
                        show_error_popup(&self, "", true, Some(x));
                    }
                    _ => (),
                };
            }
            Some(iconic_file)
        } else {
            show_error_popup(
                &self,
                &gettext("No file or path found, this is probably not your fault."),
                true,
                None,
            );
            None
        };
        match new_file.clone() {
            Some(file) => {
                match change_top_icon {
                    true => imp.top_image_file.lock().unwrap().replace(file),
                    false => imp.bottom_image_file.lock().unwrap().replace(file),
                };
                self.check_icon_update();
            }
            None => {
                self.check_icon_update();
            }
        }
        new_file
    }
}
