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
use crate::objects::file::File;
use adw::prelude::{AlertDialogExt, AlertDialogExtManual};
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gio::Cancellable;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{gdk, glib};
use image::*;
use log::*;
use rand::prelude::*;
use std::cell::RefCell;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::config::{APP_ICON, APP_ID, PROFILE};

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
        pub image_loading_spinner: TemplateChild<adw::Spinner>,
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
        #[template_child]
        pub monochrome_invert: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub main_status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub image_preferences: TemplateChild<adw::Clamp>,

        pub folder_image_file: Arc<Mutex<Option<File>>>,
        pub default_color: gdk::RGBA,
        pub top_image_file: Arc<Mutex<Option<File>>>,
        pub saved_file: Arc<Mutex<Option<gio::File>>>,
        pub file_created: RefCell<bool>,
        pub image_saved: RefCell<bool>,
        pub last_dnd_generated_name: RefCell<Option<gio::File>>,
        pub generated_image: RefCell<Option<DynamicImage>>,
        pub signals: RefCell<Vec<glib::SignalHandlerId>>,
        //pub drop_target_item: RefCell<Option<gtk::DropTarget>>,
        pub settings: gio::Settings,
        pub count: RefCell<i32>,
    }

    impl Default for GtkTestWindow {
        fn default() -> Self {
            Self {
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
                image_preferences: TemplateChild::default(),
                folder_image_file: Arc::new(Mutex::new(None)),
                top_image_file: Arc::new(Mutex::new(None)),
                saved_file: Arc::new(Mutex::new(None)),
                image_saved: RefCell::new(true),
                generated_image: RefCell::new(None),
                file_created: RefCell::new(false),
                signals: RefCell::new(vec![]),
                x_scale: TemplateChild::default(),
                y_scale: TemplateChild::default(),
                size: TemplateChild::default(),
                stack: TemplateChild::default(),
                main_status_page: TemplateChild::default(),
                monochrome_invert: TemplateChild::default(),
                image_loading_spinner: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                count: RefCell::new(0),
                default_color: gdk::RGBA::new(0.262745098, 0.552941176, 0.901960784, 1.0),
                last_dnd_generated_name: RefCell::new(None),
                //drop_target_item: RefCell::new(None),
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
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.render_to_screen().await;
                    }
                ));
            });
            klass.install_action("app.open_top_icon", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.load_top_icon().await;
                    }
                ));
            });
            klass.install_action("app.open_file_location", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.open_directory().await;
                    }
                ));
            });
            klass.install_action("app.select_folder", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.load_temp_folder_icon().await;
                    }
                ));
            });
            klass.install_action("app.reset_bottom", None, move |win, _, _| {
                win.reset_bottom_icon();
            });
            klass.install_action("app.paste", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.paste_from_clipboard().await;
                    }
                ));
            });
            klass.install_action("app.save_button", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        match win.open_save_file_dialog().await {
                            Ok(_) => (),
                            Err(error) => {
                                win.show_error_popup(&error.to_string(), true, Some(error));
                            }
                        };
                    }
                ));
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
            let drop_target = gtk::DropTarget::new(gio::File::static_type(), gdk::DragAction::COPY);
            drop_target.connect_drop(clone!(
                #[strong]
                obj,
                move |_, value, _, _| {
                    if let Ok(file) = value.get::<gio::File>() {
                        glib::spawn_future_local(glib::clone!(
                            #[weak(rename_to = win)]
                            obj,
                            async move {
                                win.open_dragged_file(file).await;
                            }
                        ));
                        true
                    } else {
                        false
                    }
                }
            ));

            let drop_target_2 =
                gtk::DropTarget::new(gio::File::static_type(), gdk::DragAction::COPY);
            drop_target_2.connect_drop(clone!(
                #[strong]
                obj,
                move |_, value, _, _| {
                    if let Ok(file) = value.get::<gio::File>() {
                        glib::spawn_future_local(glib::clone!(
                            #[weak(rename_to = win)]
                            obj,
                            async move {
                                win.open_dragged_file(file).await;
                            }
                        ));
                        true
                    } else {
                        false
                    }
                }
            ));

            //self.drop_target_item.replace(Some(drop_target));
            let drag_source = gtk::DragSource::builder()
                .actions(gdk::DragAction::COPY)
                .build();

            drag_source.connect_prepare(clone!(
                #[weak (rename_to = win)]
                obj,
                #[upgrade_or]
                None,
                move |drag, _, _| win.drag_connect_prepare(drag)
            ));

            // drag_source.connect_drag_end(clone!(
            //     #[weak (rename_to = win)]
            //     self,
            //     move|_,_,_| {
            //     debug!("drag end");
            //     let drop_target = win.drop_target_item.borrow().clone().unwrap();
            //     win.main_status_page.add_controller(drop_target);}
            // ));

            drag_source.connect_drag_cancel(clone!(
                #[weak (rename_to = win)]
                obj,
                #[upgrade_or]
                false,
                move |_, _, drag_cancel_reason| win.drag_connect_cancel(drag_cancel_reason)
            ));
            self.main_status_page.add_controller(drop_target);
            self.image_preferences.add_controller(drop_target_2);

            self.image_view.add_controller(drag_source);
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }
    impl WidgetImpl for GtkTestWindow {}
    impl WindowImpl for GtkTestWindow {
        fn close_request(&self) -> glib::Propagation {
            if !self.image_saved.borrow().clone() {
                let window = self.obj();
                return match glib::MainContext::default()
                    .block_on(async move { window.confirm_save_changes().await })
                {
                    Ok(p) => p,
                    _ => glib::Propagation::Stop,
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
        let imp = win.imp();
        if PROFILE == "Devel" {
            imp.main_status_page.set_icon_name(Some(APP_ICON));
        }
        imp.save_button.set_sensitive(false);
        imp.x_scale.add_mark(0.0, gtk::PositionType::Top, None);
        imp.y_scale.add_mark(0.0, gtk::PositionType::Bottom, None);
        imp.y_scale.set_value(9.447);
        imp.size.set_value(24.0);
        imp.size.add_mark(24.0, gtk::PositionType::Top, None);
        imp.y_scale.add_mark(9.447, gtk::PositionType::Bottom, None);
        imp.stack.set_visible_child_name("stack_welcome_page");
        imp.reset_color.set_visible(false);
        win.load_folder_path_from_settings();
        win.setup_settings();
        win.setup_update();
        win
    }

    pub fn drag_connect_prepare(&self, source: &gtk::DragSource) -> Option<gdk::ContentProvider> {
        let imp = self.imp();
        //imp.main_status_page.remove_controller(&imp.drop_target_item.borrow().clone().unwrap());
        let generated_image = imp.generated_image.borrow().clone().unwrap();
        let file_name = imp.top_image_file.lock().unwrap().clone().unwrap().name;
        let icon = self.dynamic_image_to_texture(&generated_image.resize(
            64,
            64,
            imageops::FilterType::Nearest,
        ));
        source.set_icon(Some(&icon), 0 as i32, 0 as i32);
        let gio_file = self.create_drag_file(&file_name);
        imp.last_dnd_generated_name.replace(Some(gio_file.clone()));
        let gio_file_clone = gio_file.clone();
        glib::spawn_future_local(clone!(
            #[weak (rename_to = win)]
            self,
            async move {
                win.save_file(gio_file_clone).await.unwrap();
            }
        ));
        Some(gdk::ContentProvider::for_value(&glib::Value::from(
            &gio_file,
        )))
    }

    pub fn create_drag_file(&self, file_name: &str) -> gio::File {
        let imp = self.imp();
        let data_path = match env::var("XDG_DATA_HOME") {
            Ok(value) => PathBuf::from(value),
            Err(_) => {
                let config_dir = PathBuf::from(env::var("HOME").unwrap())
                    .join(".data")
                    .join("Iconic");
                if !config_dir.exists() {
                    fs::create_dir(&config_dir).unwrap();
                }
                config_dir
            }
        };
        debug!("data path: {:?}", data_path);
        let random_number = random::<u64>();
        let properties_string = self.create_image_properties_string();
        let generated_file_name = format!("folder-{}-{}.png", properties_string, file_name);
        debug!("generated_file_name: {}", generated_file_name);
        let mut file_path = data_path.clone();
        file_path.push(generated_file_name.clone());
        debug!("generated file path: {:?}", file_path);
        let gio_file = gio::File::for_path(file_path);
        // if gio_file.query_exists(None::<&Cancellable>) {
        //     warn!("File with name {} already exists, creating new file name", generated_file_name);
        //     self.create_drag_file(file_name)
        // }
        // else{
        //     info!("File with name {} does not yet exist, using file name", generated_file_name);
        //     gio_file
        // }
        gio_file
    }

    /* This function is used to create a string with all properties applied to the current image.
    This makes it possible to completely recreate the image if the top image is still available
    */
    fn create_image_properties_string(&self) -> String {
        let imp = self.imp();
        let x_scale_val = imp.x_scale.value();
        let y_scale_val = imp.y_scale.value();
        let zoom_val = imp.size.value();
        let is_monochrome = imp.monochrome_switch.is_active() as u8;
        let monochrome_slider = imp.threshold_scale.value();
        let monochrome_color_val = imp.monochrome_color.rgba().to_string();
        let monochrome_inverted = imp.monochrome_invert.is_active() as u8;
        let combined_string = format!(
            "{}-{}-{}-{}-{}-{}-{}",
            x_scale_val,
            y_scale_val,
            zoom_val,
            is_monochrome,
            monochrome_slider,
            monochrome_color_val,
            monochrome_inverted
        );
        debug!("{}", &combined_string);
        combined_string
    }

    fn drag_connect_cancel(&self, reason: gdk::DragCancelReason) -> bool {
        let imp = self.imp();
        let gio_file = imp.last_dnd_generated_name.borrow().clone().unwrap();
        info!(
            "Drag operation cancelled, removing file. Reason: {:?}",
            reason
        );
        match gio_file.delete(None::<&Cancellable>) {
            Ok(_) => {
                debug!("Deletion succesfull!");
            }
            Err(e) => {
                warn!("Could not delete drag file, error: {:?}", e);
            }
        };
        false
    }

    pub fn setup_settings(&self) {
        let update_folder = glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_: &gio::Settings, setting: &str| {
                let path: &str = &win.imp().settings.string(setting);
                win.load_folder_icon(path);
            }
        );

        let resize_folder = glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_: &gio::Settings, _: &str| {
                let path: &str = &win.imp().settings.string("folder-svg-path");
                win.load_folder_icon(path);
            }
        );

        let reload_thumbnails = glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_: &gio::Settings, _: &str| {
                let path: &str = &win.imp().settings.string("folder-svg-path");
                win.load_folder_icon(path);
                win.imp().reset_color.set_visible(true);
            }
        );

        self.imp()
            .settings
            .connect_changed(Some("folder-svg-path"), update_folder.clone());
        self.imp()
            .settings
            .connect_changed(Some("svg-render-size"), resize_folder.clone());
        self.imp()
            .settings
            .connect_changed(Some("thumbnail-size"), reload_thumbnails.clone());
    }

    pub fn setup_update(&self) {
        self.imp().x_scale.connect_value_changed(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.imp().image_saved.replace(false);
                        win.imp().save_button.set_sensitive(true);
                        win.render_to_screen().await;
                    }
                ));
            }
        ));
        self.imp().y_scale.connect_value_changed(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.render_to_screen().await;
                        win.imp().image_saved.replace(false);
                        win.imp().save_button.set_sensitive(true);
                    }
                ));
            }
        ));
        self.imp().size.connect_value_changed(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.render_to_screen().await;
                        win.imp().image_saved.replace(false);
                        win.imp().save_button.set_sensitive(true);
                    }
                ));
            }
        ));
        self.imp().threshold_scale.connect_value_changed(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.render_to_screen().await;
                        win.imp().image_saved.replace(false);
                        win.imp().save_button.set_sensitive(true);
                    }
                ));
            }
        ));
        self.imp().monochrome_color.connect_rgba_notify(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        let imp = win.imp();
                        if imp.monochrome_color.rgba() != imp.default_color.clone() {
                            imp.reset_color.set_visible(true);
                        }
                        imp.image_saved.replace(false);
                        imp.save_button.set_sensitive(true);
                        win.render_to_screen().await;
                    }
                ));
            }
        ));
        self.imp().monochrome_invert.connect_active_notify(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.render_to_screen().await;
                        win.imp().image_saved.replace(false);
                        win.imp().save_button.set_sensitive(true);
                    }
                ));
            }
        ));
    }

    pub fn reset_colors(&self) {
        let imp = self.imp();
        imp.reset_color.set_visible(false);

        imp.monochrome_color.set_rgba(&imp.default_color.clone());
        self.check_icon_update();
        imp.reset_color.set_visible(false);
    }

    pub async fn check_chache_icon(&self, file_name: &str) -> PathBuf {
        let imp = self.imp();
        let icon_path = PathBuf::from(&imp.settings.string("folder-svg-path"));
        let cache_path = match env::var("XDG_CACHE_HOME") {
            Ok(value) => PathBuf::from(value),
            Err(_) => {
                let config_dir = PathBuf::from(env::var("HOME").unwrap())
                    .join(".cache")
                    .join("Iconic");
                if !config_dir.exists() {
                    fs::create_dir(&config_dir).unwrap();
                }
                config_dir
            }
        };
        let folder_icon_cache_path = cache_path.join(file_name);
        if folder_icon_cache_path.exists() {
            info!("File found in cache at: {:?}", folder_icon_cache_path);
            return folder_icon_cache_path;
        }
        if icon_path.exists() {
            info!(
                "File not found in cache, copying to: {:?}",
                folder_icon_cache_path
            );
            return self.copy_folder_image_to_cache(&icon_path, &cache_path).0;
        }
        info!("File not found AT ALL");
        let dialog = self
            .show_error_popup(
                &gettext("The set folder icon could not be found, press ok to select a new one"),
                false,
                None,
            )
            .unwrap();
        match &*dialog.clone().choose_future(self).await {
            "OK" => {
                let new_path = match self.open_file_chooser_gtk().await {
                    Some(x) => x.path().unwrap().into_os_string().into_string().unwrap(),
                    None => {
                        //adw::subclass::prelude::ActionGroupImpl::activate_action(&self, "app.quit", None);
                        String::from("")
                    }
                };
                imp.settings
                    .set_string("folder-svg-path", &new_path)
                    .unwrap();
                let cached_file_name = self
                    .copy_folder_image_to_cache(&PathBuf::from(new_path), &cache_path)
                    .1;
                imp.settings
                    .set_string("folder-cache-name", &cached_file_name)
                    .unwrap();
                let cache_file_name = &imp.settings.string("folder-cache-name");
                let folder_icon_cache_path = cache_path.join(cache_file_name);
                return PathBuf::from(folder_icon_cache_path);
            }
            _ => unreachable!(),
        };
    }

    pub fn copy_folder_image_to_cache(
        &self,
        original_path: &PathBuf,
        cache_dir: &PathBuf,
    ) -> (PathBuf, String) {
        let file_name = format!(
            "folder.{}",
            original_path.extension().unwrap().to_str().unwrap()
        );
        self.imp()
            .settings
            .set("folder-cache-name", file_name.clone())
            .unwrap();
        let cache_path = cache_dir.join(file_name.clone());
        fs::copy(original_path, cache_path.clone()).unwrap();
        (cache_path, file_name)
    }

    pub fn show_error_popup(
        &self,
        message: &str,
        show: bool,
        error: Option<Box<dyn Error + '_>>,
    ) -> Option<adw::AlertDialog> {
        const RESPONSE_OK: &str = "OK";
        let error_text: &str = &gettext("Error");
        let dialog = adw::AlertDialog::builder()
            .heading(format!(
                "<span foreground=\"red\"><b>⚠ {error_text}</b></span>"
            ))
            .heading_use_markup(true)
            .body(message)
            .default_response(RESPONSE_OK)
            .build();
        dialog.add_response(RESPONSE_OK, &gettext("OK"));
        match error {
            Some(ref x) => error!("An error has occured: \"{:?}\"", x),
            None => error!("An error has occured: \"{}\"", message),
        };
        match show {
            true => {
                dialog.present(Some(self));
                None
            }
            false => Some(dialog),
        }
    }

    pub async fn top_or_bottom_popup(&self) -> Option<bool> {
        let dnd_switch_state = self.imp().settings.boolean("default-dnd-activated");
        let dnd_radio_state = self.imp().settings.string("default-dnd-action");
        if dnd_switch_state {
            return match dnd_radio_state.as_str() {
                "top" => Some(true),
                "bottom" => Some(false),
                _ => None,
            };
        }
        const RESPONSE_TOP: &str = "TOP";
        const RESPONSE_BOTTOM: &str = "BOTTOM";
        let load_question: &str =
            &gettext("Do you want to load this image to the top or bottom layer?");
        let disable_hint: &str = &gettext("Hint: You can disable this pop-up in the settings");
        let dialog = adw::AlertDialog::builder()
            .heading(gettext("Select layer"))
            .body(format!("{load_question} \n <sub> <span foreground=\"#9A9996\"> {disable_hint}</span> </sub>"))
            .body_use_markup(true)
            .default_response(RESPONSE_TOP)
            .build();
        dialog.add_response(RESPONSE_TOP, &gettext("Top"));
        dialog.add_response(RESPONSE_BOTTOM, &gettext("Bottom"));
        dialog.set_response_appearance(RESPONSE_TOP, adw::ResponseAppearance::Suggested);

        match &*dialog.clone().choose_future(self).await {
            RESPONSE_TOP => Some(true),
            RESPONSE_BOTTOM => Some(false),
            _ => None,
        }
    }

    pub async fn confirm_save_changes(&self) -> Result<glib::Propagation, ()> {
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
                dialog.close();
                Ok(glib::Propagation::Stop)
            }
            RESPONSE_DISCARD => Ok(glib::Propagation::Proceed),
            RESPONSE_SAVE => match self.open_save_file_dialog().await {
                Ok(saved) => match saved {
                    true => Ok(glib::Propagation::Proceed),
                    false => Ok(glib::Propagation::Stop),
                },
                Err(error) => {
                    self.show_error_popup(&error.to_string(), true, Some(error));
                    Ok(glib::Propagation::Stop)
                }
            },
            _ => unreachable!(),
        }
    }

    pub fn check_icon_update(&self) {
        let imp = self.imp();
        if imp.top_image_file.lock().unwrap().as_ref() != None
            && imp.folder_image_file.lock().unwrap().as_ref() != None
        {
            self.imp().save_button.set_sensitive(true);
            self.imp().image_saved.replace(false);
            glib::spawn_future_local(glib::clone!(
                #[weak(rename_to = win)]
                self,
                async move {
                    win.render_to_screen().await;
                }
            ));
            imp.stack.set_visible_child_name("stack_main_page");
        } else if imp.folder_image_file.lock().unwrap().as_ref() != None {
            imp.image_view.set_paintable(Some(
                &self.dynamic_image_to_texture(
                    &imp.folder_image_file
                        .lock()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .thumbnail,
                ),
            ));
            if imp.stack.visible_child_name() != Some("stack_main_page".into()) {
                imp.stack.set_visible_child_name("stack_welcome_page");
            }
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
            Ok(x) => {
                debug!("{:?}", &x.path().unwrap());
                Some(x)
            }
            Err(y) => {
                error!("{:?}", y);
                None
            }
        }
    }

    pub async fn open_directory(&self) {
        let imp = self.imp();
        let launcher =
            gtk::FileLauncher::new(Some(&imp.saved_file.lock().unwrap().clone().unwrap()));
        let win = self.native().and_downcast::<gtk::Window>();
        if let Err(e) = launcher.open_containing_folder_future(win.as_ref()).await {
            error!("Could not open directory {}", e);
        };
    }

    pub fn dynamic_image_to_texture(&self, dynamic_image: &DynamicImage) -> gdk::Texture {
        let rgba_image = dynamic_image.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        let pixels = rgba_image.into_raw(); // Get the raw pixel data
                                            // Create Pixbuf from raw pixel data
        let pixbuf = Pixbuf::from_bytes(
            &glib::Bytes::from(&pixels),
            gtk::gdk_pixbuf::Colorspace::Rgb,
            true, // has_alpha
            8,    // bits_per_sample
            width as i32,
            height as i32,
            width as i32 * 4, // rowstride
        );
        gdk::Texture::for_pixbuf(&pixbuf)
    }

    pub fn enable_monochrome_expand(&self) {
        let switch_state = self.imp().monochrome_switch.state();
        match switch_state {
            false => {
                self.imp()
                    .monochrome_action_row
                    .set_property("enable_expansion", true);
            }
            true => {
                self.imp()
                    .monochrome_action_row
                    .set_property("enable_expansion", false);
            }
        };
        self.check_icon_update();
    }
}
