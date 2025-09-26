use crate::objects::errors::{IntoResult, show_error_popup};
use crate::objects::file::File;
use crate::objects::properties::{BottomImageType, FileProperties};
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gio::*;
use gtk::{gdk, glib};
use image::*;
use log::*;
use xmp_toolkit::{xmp_ns, OpenFileOptions, XmpFile, XmpMeta, XmpValue};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{GenResult, GtkTestWindow};

impl GtkTestWindow {
    // Load the correct folder based on settings
    // TODO This function is quite confusingly written
    pub fn load_folder_path_from_settings(&self) {
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = win)]
            self,
            async move {
                let imp = win.imp();
                let path;
                imp.temp_bottom_image_loaded.replace(false);
                let mut file_properties = imp.file_properties.borrow_mut();
                if imp.settings.boolean("manual-bottom-image-selection") {
                    file_properties.bottom_image_type = BottomImageType::Custom;
                    let cache_file_name: &str = &win.imp().settings.string("folder-cache-name");
                    path = win.check_chache_icon(cache_file_name).await;
                } else if imp.settings.string("selected-accent-color") == "Custom" {
                    let custom_primary_color: String =
                        imp.settings.string("primary-folder-color").into();
                    let custom_secondary_color: String =
                        imp.settings.string("secondary-folder-color").into();
                    path = win
                        .create_custom_folder_color(&custom_primary_color, &custom_secondary_color)
                        .await;
                    file_properties.bottom_image_type =
                        BottomImageType::FolderCustom(custom_primary_color, custom_secondary_color);
                } else {
                    let set_folder_color: String =
                        imp.settings.string("selected-accent-color").into();
                    path = win.load_built_in_bottom_icon(&set_folder_color).await;
                    file_properties.bottom_image_type = match set_folder_color.as_str() {
                        "None" => BottomImageType::FolderSystem,
                        _ => BottomImageType::Folder(set_folder_color),
                    }
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

    // Replace the
    async fn create_custom_folder_color(
        &self,
        primary_color: &str,
        secondary_color: &str,
    ) -> PathBuf {
        info!("Creating custom folder colors");
        let folder_svg_file =
            std::fs::read_to_string("/app/share/folder_icon/folders/folder_Custom.svg").unwrap();
        let folder_svg_lines = folder_svg_file.lines();
        let new_custom_folder: String = folder_svg_lines
            .map(|row| {
                let row_clone = row.to_string();
                let row_clone = row_clone.replace("a4caee", &primary_color);
                let mut row_clone = row_clone.replace("438de6", &secondary_color);
                row_clone.push_str("\n");
                row_clone
            })
            .collect();
        let new_custom_folder_bytes = new_custom_folder.as_bytes().to_owned();
        let mut cache_location = Self::get_cache_path();
        cache_location.push("custom_folder.svg");
        let cache_location_clone = cache_location.clone();

        gio::spawn_blocking(move || {
            let _ = std::fs::write(&cache_location_clone, new_custom_folder_bytes);
        })
        .await
        .unwrap();
        cache_location
    }

    pub async fn load_built_in_bottom_icon(&self, accent_color_setting: &str) -> PathBuf {
        // let imp = self.imp();
        let folder_color_name = match accent_color_setting {
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
        let thumbnail_size: u32 = imp.settings.get("thumbnail-size");
        let svg_size: u32 = imp.settings.get("svg-render-size");

        match clipboard
            .read_future(&["image/svg+xml"], glib::Priority::DEFAULT)
            .await
        {
            Ok((stream, _mime)) => self.clipboard_load_svg(Some(stream)).await,
            Err(_) => match clipboard.read_texture_future().await {
                Ok(Some(texture)) => {
                    let top_file_selected = self.top_or_bottom_popup().await;
                    imp.image_loading_spinner.set_visible(true);

                    let png_texture = texture.save_to_tiff_bytes();
                    let image =
                        image::load_from_memory_with_format(&png_texture, image::ImageFormat::Tiff)
                            .unwrap();
                    match top_file_selected {
                        Some(true) => {
                            // Pasting from a clipboard does not get a file path, which the new_iconic_file creation function needs
                            let iconic_file = gio::spawn_blocking(move || {
                                File::from_image(image, thumbnail_size, Some(svg_size), "pasted")
                            })
                            .await
                            .unwrap();
                            imp.top_image_file.lock().unwrap().replace(iconic_file);
                        }
                        _ => {
                            imp.temp_bottom_image_loaded.replace(true);
                            imp.bottom_image_file.lock().unwrap().replace(
                                gio::spawn_blocking(move || {
                                    File::from_image(
                                        image.clone(),
                                        thumbnail_size,
                                        Some(svg_size),
                                        "pasted",
                                    )
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
        let thumbnail_size: u32 = imp.settings.get("thumbnail-size");
        let svg_render_size: u32 = imp.settings.get("svg-render-size");
        let none_file: Option<&gio::File> = None;
        let none_cancelable: Option<&Cancellable> = None;
        let loader = rsvg::Loader::new()
            .read_stream(&stream.unwrap(), none_file, none_cancelable)
            .unwrap();
        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            svg_render_size as i32,
            svg_render_size as i32,
        )
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
        let thumbnail_size: u32 = imp.settings.get("thumbnail-size");
        let svg_render_size: u32 = imp.settings.get("svg-render-size");

        debug!("{:#?}", file.path().value_type());
        let file_info =
            match file.query_info("standard::", FileQueryInfoFlags::NONE, Cancellable::NONE) {
                Ok(x) => x,
                Err(e) => {
                    show_error_popup(&self, &e.to_string(), true, Some(Box::new(e)));
                    FileInfo::default()
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
                imp.image_loading_spinner.set_visible(true);
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
                        imp.temp_bottom_image_loaded.replace(true);
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
                imp.image_loading_spinner.set_visible(false);
            }
            _ => {
                show_error_popup(&self, &gettext("Unsupported file type"), true, None);
            }
        }
    }

    pub async fn open_dragged_texture(&self, file: gdk::Texture) {
        let imp = self.imp();
        let thumbnail_size: u32 = imp.settings.get("thumbnail-size");
        let svg_size: u32 = imp.settings.get("svg-render-size");
        if let Ok(image) =
            image::load_from_memory_with_format(&file.save_to_png_bytes(), image::ImageFormat::Png)
        {
            let file = File::from_image(image, thumbnail_size, Some(svg_size), "dragged.png");
            let top_file_selected = self.top_or_bottom_popup().await;
            match top_file_selected {
                Some(true) => {
                    imp.top_image_file.lock().unwrap().replace(file);
                    self.check_icon_update();
                }
                Some(false) => {
                    // This value must be true if a temporary bottom image is loaded
                    // That is dumb
                    imp.temp_bottom_image_loaded.replace(true);
                    imp.bottom_image_file.lock().unwrap().replace(file);
                    self.check_icon_update();
                }
                None => (),
            }
        } else {
            show_error_popup(&self, "Failed to load image", true, None);
        }
    }

    pub async fn open_save_file_dialog(&self) -> Result<bool, Box<dyn Error + '_>> {
        let imp = self.imp();
        if !imp.save_button.is_sensitive() {
            imp.toast_overlay
                .add_toast(adw::Toast::new(&gettext("Nothing to save")));
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
                    .save_file(file, imp.monochrome_switch.is_active(), None, None)
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
                            .add_toast(adw::Toast::new(&gettext("File not saved")));
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
        Ok((cache_path, file_name))
    }

    pub async fn save_file(
        &self,
        file: gio::File,
        use_monochrome: bool,
        manual_monochrome_values: Option<(u8, gtk::gdk::RGBA)>,
        top_image_hash: Option<u64>
    ) -> Result<bool, Box<dyn Error + '_>> {
        let imp = self.imp();
        let _busy_lock = Arc::clone(&imp.app_busy);
        self.image_save_sensitive(false);
        imp.saved_file.lock()?.replace(file.clone());
        let base_image = imp
            .bottom_image_file
            .lock()?
            .as_ref()
            .into_reason_result("No bottom image found")?
            .dynamic_image
            .clone();
        
        let mut top_image_dynamicimage = {
            let _top_image_lock = imp.top_image_file.lock()?;
            let top_image = _top_image_lock
                .as_ref()
                .into_reason_result("No top image found")?;
            top_image.dynamic_image.clone()};
        if use_monochrome {
            let (monochrome_threshold, monochrome_color) = match manual_monochrome_values {
                Some((threshold, color)) => (threshold, color),
                None => (
                    imp.threshold_scale.value() as u8,
                    imp.monochrome_color.rgba(),
                ),
            };
            top_image_dynamicimage = self.to_monochrome(
                top_image_dynamicimage,
                monochrome_threshold,
                monochrome_color,
                None,
            );
        }
        let generated_image = self
            .generate_image(
                base_image,
                top_image_dynamicimage,
                imageops::FilterType::Gaussian,
                imp.x_scale.value(),
                imp.y_scale.value(),
                imp.size.value(),
            )
            .await;
        let path = file.path().unwrap();
        let path_clone = path.clone();
        let _ = gio::spawn_blocking(move || {
            generated_image.save_with_format(path, ImageFormat::Png)
        })
        .await
        .unwrap()?;
        self.add_image_metadata(path_clone, top_image_hash).unwrap();
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
        let thumbnail_size: u32 = imp.settings.get("thumbnail-size");
        let size: u32 = imp.settings.get("svg-render-size");
        match self.open_file_chooser().await {
            Some(x) => {
                imp.temp_bottom_image_loaded.replace(true);
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
        let size: u32 = self.imp().settings.get("thumbnail-size");
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
            imp.stack.set_visible_child_name("stack_main_page");
            imp.image_loading_spinner.set_visible(true);
        }
        let svg_render_size: u32 = imp.settings.get("svg-render-size");
        let thumbnail_size: u32 = imp.settings.get("thumbnail-size");
        self.new_iconic_file_creation(Some(filename), None, svg_render_size, thumbnail_size, true)
            .await;
    }

    // Creates a new folder_icon::File from a gio::file or path.
    // Will show an error if none are provided
    pub async fn new_iconic_file_creation(
        &self,
        file: Option<gio::File>,
        path: Option<PathBuf>,
        svg_render_size: u32,
        thumbnail_render_size: u32,
        change_top_icon: bool,
    ) -> Option<File> {
        if path.is_none() && file.is_none() {
            show_error_popup(
                &self,
                &gettext("No file or path found, this is probably not your fault."),
                true,
                None,
            );
            return None;
        }
        let imp = self.imp();
        let file_temp = if let Some(path_temp) = path {
            gio::File::for_path(path_temp)
        } else {
            file.unwrap()
        };
        let new_file = match gio::spawn_blocking(move || {
            File::new(file_temp, svg_render_size, thumbnail_render_size)
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

        match change_top_icon {
            true => imp.top_image_file.lock().unwrap().replace(new_file.clone()),
            false => imp
                .bottom_image_file
                .lock()
                .unwrap()
                .replace(new_file.clone()),
        };

        self.check_icon_update();
        Some(new_file)
    }

    fn add_image_metadata(&self, path: PathBuf, top_image_hash: Option<u64>) -> GenResult<()> {
        let properties = FileProperties::new(&self, top_image_hash, self.get_default_color());
        let mut file = XmpFile::new()?;
        file.open_file(path, OpenFileOptions::default().for_update())?;
        let mut metadata = XmpMeta::new()?;
        metadata.set_property(xmp_ns::XMP, "x_val", &XmpValue::new(properties.x_val.to_string()))?;
        metadata.set_property(xmp_ns::XMP, "y_val", &XmpValue::new(properties.y_val.to_string()))?;
        metadata.set_property(xmp_ns::XMP, "zoom_val", &XmpValue::new(properties.zoom_val.to_string()))?;
        metadata.set_property(xmp_ns::XMP, "monochrome_toggle", &XmpValue::new(properties.monochrome_toggle.to_string()))?;
        if let Some(colors) = properties.monochrome_color {
            metadata.set_property(xmp_ns::XMP, "monochrome_red", &XmpValue::new(colors.0.to_string()))?;
            metadata.set_property(xmp_ns::XMP, "monochrome_blue", &XmpValue::new(colors.1.to_string()))?;
            metadata.set_property(xmp_ns::XMP, "monochrome_green", &XmpValue::new(colors.2.to_string()))?;
        }
        metadata.set_property(xmp_ns::XMP, "monochrome_default", &XmpValue::new(properties.monochrome_default.to_string()))?;
        metadata.set_property(xmp_ns::XMP, "monochrome_invert", &XmpValue::new(properties.monochrome_invert.to_string()))?;
        metadata.set_property(xmp_ns::XMP, "monochrome_threshold", &XmpValue::new(properties.monochrome_threshold_val.to_string()))?;
        if let Some(hash) = properties.top_image_hash {
            metadata.set_property(xmp_ns::XMP, "top_image_hash", &XmpValue::new(hash.to_string()))?;
        }
        metadata.set_property(xmp_ns::XMP, "bottom_image_type", &XmpValue::new(serde_json::to_string(&properties.bottom_image_type)?))?;
        file.put_xmp(&metadata)?;
        file.close();
        Ok(())
    }
}
