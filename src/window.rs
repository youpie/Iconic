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

use crate::config::{APP_ICON, APP_ID, PROFILE};
use crate::glib::clone;
use crate::objects::errors::show_error_popup;
use crate::objects::file::File;
use crate::settings::settings::PreferencesDialog;
use adw::prelude::AlertDialogExtManual;
use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gio::Cancellable;
use gtk::gdk::RGBA;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{gdk, glib};
use image::*;
use log::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::hash::RandomState;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

mod imp {
    use std::collections::HashMap;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/nl/emphisia/icon/window.ui")]
    pub struct GtkTestWindow {
        // Template widgets
        // Every item in this list, is something defined in
        // a blueprint file, if you want to control it from the code
        // it needs to be added here
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
        #[template_child]
        pub popover_menu: TemplateChild<gtk::PopoverMenu>,
        #[template_child]
        pub gesture_click: TemplateChild<gtk::GestureClick>,
        #[template_child]
        pub regeneration_osd: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub regeneration_revealer: TemplateChild<gtk::Revealer>,

        pub bottom_image_file: Arc<Mutex<Option<File>>>,
        pub default_color: RefCell<HashMap<String, gdk::RGBA, RandomState>>,
        pub top_image_file: Arc<Mutex<Option<File>>>,
        pub saved_file: Arc<Mutex<Option<gio::File>>>,
        pub file_created: RefCell<bool>,
        pub image_saved: RefCell<bool>,
        pub last_dnd_generated_name: RefCell<Option<gio::File>>,
        pub generated_image: RefCell<Option<DynamicImage>>,
        pub temp_image_loaded: RefCell<bool>,
        pub signals: RefCell<Vec<glib::SignalHandlerId>>,
        pub settings: gio::Settings,
        pub count: RefCell<i32>,
        pub regeneration_lock: Arc<RefCell<usize>>,
        pub app_busy: Arc<()>,
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
                regeneration_osd: TemplateChild::default(),
                x_scale: TemplateChild::default(),
                y_scale: TemplateChild::default(),
                size: TemplateChild::default(),
                stack: TemplateChild::default(),
                main_status_page: TemplateChild::default(),
                monochrome_invert: TemplateChild::default(),
                image_loading_spinner: TemplateChild::default(),
                popover_menu: TemplateChild::default(),
                gesture_click: TemplateChild::default(),
                regeneration_revealer: TemplateChild::default(),
                bottom_image_file: Arc::new(Mutex::new(None)),
                top_image_file: Arc::new(Mutex::new(None)),
                saved_file: Arc::new(Mutex::new(None)),
                image_saved: RefCell::new(true),
                generated_image: RefCell::new(None),
                file_created: RefCell::new(false),
                signals: RefCell::new(vec![]),
                settings: gio::Settings::new(APP_ID),
                count: RefCell::new(0),
                temp_image_loaded: RefCell::new(false),
                default_color: RefCell::new(HashMap::new()),
                last_dnd_generated_name: RefCell::new(None),
                regeneration_lock: Arc::new(RefCell::new(0)),
                app_busy: Arc::new(()),
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
            klass.install_action("app.open_top_icon", None, move |win, _, _| {
                // glib::spawn_future_local(clone!(
                //     #[weak]
                //     win,
                //     async move {
                //         win.load_top_icon().await;
                //     }
                // ));
                error!("References: {}", Arc::strong_count(&win.imp().app_busy));
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
            klass.install_action("app.open_bottom_icon", None, move |win, _, _| {
                win.check_icon_update();
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
            klass.install_action("app.regenerate", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        let imp = win.imp();
                        let id = *imp.regeneration_lock.borrow();
                        imp.regeneration_lock.replace(id + 1);
                        match win.regenerate_icons().await {
                            Ok(_) => (),
                            Err(x) => {
                                show_error_popup(&win, "", true, Some(x));
                            }
                        };

                        imp.toast_overlay.add_toast(adw::Toast::new(&gettext(
                            "Regeneration sucessful, restart nautilus",
                        )));
                        //imp.stack.set_visible_child_name(&previous_stack);
                        debug!("Done generating");
                    }
                ));
            });
            klass.install_action("app.save_button", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.drag_and_drop_information_dialog();
                        match win.open_save_file_dialog().await {
                            Ok(_) => (),
                            Err(error) => {
                                show_error_popup(&win, &error.to_string(), true, Some(error));
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

            drag_source.connect_drag_end(clone!(
                #[weak (rename_to = win)]
                obj,
                move |_, _, _| win.drag_connect_end()
            ));

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
            error!("close request");
            let window = self.obj();
            if window.visible_dialog().is_some() {
                return glib::Propagation::Proceed;
            }
            if Arc::strong_count(&self.app_busy) >= 2 {
                match glib::MainContext::default().block_on(glib::clone!(
                    #[weak]
                    window,
                    #[upgrade_or]
                    false,
                    async move { window.force_quit_dialog().await }
                )) {
                    true => (),
                    false => return glib::Propagation::Stop,
                }
            }
            if self.image_saved.borrow().clone() {
                return self.parent_close_request();
            }

            return match glib::MainContext::default()
                .block_on(async move { window.confirm_save_changes().await })
            {
                Ok(p) => p,
                _ => glib::Propagation::Stop,
            };
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
        imp.default_color.replace(HashMap::from([
            ("Blue".to_string(), GtkTestWindow::to_rgba(67, 141, 230)),
            ("Teal".to_string(), GtkTestWindow::to_rgba(18, 158, 176)),
            ("Green".to_string(), GtkTestWindow::to_rgba(61, 158, 79)),
            ("Yellow".to_string(), GtkTestWindow::to_rgba(203, 147, 26)),
            ("Orange".to_string(), GtkTestWindow::to_rgba(241, 119, 56)),
            ("Red".to_string(), GtkTestWindow::to_rgba(232, 64, 83)),
            ("Pink".to_string(), GtkTestWindow::to_rgba(230, 67, 146)),
            ("Purple".to_string(), GtkTestWindow::to_rgba(149, 74, 181)),
            ("Slate".to_string(), GtkTestWindow::to_rgba(99, 118, 146)),
        ]));
        win.setup_defaults();
        win.create_popover_image();
        win
    }

    fn create_popover_image(&self) {
        self.imp().gesture_click.connect_pressed(glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_gesture, _n_press, x, y| {
                let imp = win.imp();
                let position = gdk::Rectangle::new(x as i32, y as i32, 0, 0);
                debug!("popover");
                imp.popover_menu.set_pointing_to(Some(&position));
                imp.popover_menu.popup();
            }
        ));
    }

    pub fn default_sliders(&self) {
        let imp = self.imp();
        imp.x_scale.add_mark(0.0, gtk::PositionType::Top, None);
        imp.y_scale.add_mark(0.0, gtk::PositionType::Bottom, None);
        imp.y_scale.set_value(9.447);
        imp.size.set_value(24.0);
        imp.size.add_mark(24.0, gtk::PositionType::Top, None);
        imp.y_scale.add_mark(9.447, gtk::PositionType::Bottom, None);
    }

    pub fn setup_defaults(&self) {
        let imp = self.imp();
        imp.save_button.set_sensitive(false);
        self.default_sliders();

        imp.reset_color.set_visible(false);
        self.check_regeneration_needed();
        let _ = imp.settings.set_string(
            "previous-system-accent-color",
            &self.get_accent_color_and_show_dialog(),
        );
        imp.stack.set_visible_child_name("stack_welcome_page");
        self.setup_settings();
        self.setup_update();
        self.load_folder_path_from_settings();
        self.slider_control_sensitivity(false);
    }

    pub fn to_rgba(r: u8, g: u8, b: u8) -> gdk::RGBA {
        let r_float = (1.0 / 255.0 * r as f64) as f32;
        let g_float = (1.0 / 255.0 * g as f64) as f32;
        let b_float = (1.0 / 255.0 * b as f64) as f32;
        gdk::RGBA::new(r_float, g_float, b_float, 1.0)
    }
    pub fn drag_connect_prepare(&self, source: &gtk::DragSource) -> Option<gdk::ContentProvider> {
        let imp = self.imp();
        //imp.main_status_page.remove_controller(&imp.drop_target_item.borrow().clone().unwrap());
        let generated_image = imp.generated_image.borrow().clone().unwrap();
        let file_hash = imp.top_image_file.lock().unwrap().clone().unwrap().hash;
        let icon = self.dynamic_image_to_texture(&generated_image.resize(
            64,
            64,
            imageops::FilterType::Nearest,
        ));
        debug!("temp image loaded {}", *imp.temp_image_loaded.borrow());
        source.set_icon(Some(&icon), 0 as i32, 0 as i32);
        let gio_file = self.create_drag_file(file_hash);
        imp.last_dnd_generated_name.replace(Some(gio_file.clone()));
        let gio_file_clone = gio_file.clone();
        glib::spawn_future_local(clone!(
            #[weak (rename_to = win)]
            self,
            async move {
                win.save_file(
                    gio_file_clone,
                    win.imp().monochrome_switch.is_active(),
                    None,
                )
                .await
                .unwrap();
            }
        ));
        Some(gdk::ContentProvider::for_value(&glib::Value::from(
            &gio_file,
        )))
    }

    pub fn create_drag_file(&self, file_hash: u64) -> gio::File {
        // let imp = self.imp();
        let data_path = self.get_data_path();
        debug!("data path: {:?}", data_path);
        // let random_number = random::<u64>();
        let properties_string = self.create_image_properties_string();
        let generated_file_name = format!("folder_new-{}-{}.png", properties_string, file_hash);
        debug!("generated_file_name: {}", generated_file_name);
        let mut file_path = data_path.clone();
        file_path.push(generated_file_name.clone());
        debug!("generated file path: {:?}", file_path);
        let gio_file = gio::File::for_path(file_path);
        gio_file
    }

    /* This function is used to create a string with all properties applied to the current image.
    This makes it possible to completely recreate the image if the top image is still available
    */
    fn create_image_properties_string(&self) -> String {
        let imp = self.imp();
        let is_default = (!imp.settings.boolean("manual-bottom-image-selection")
            && imp.settings.string("selected-accent-color").as_str() == "None"
            && !*imp.temp_image_loaded.borrow()) as usize;
        let x_scale_val = imp.x_scale.value();
        let y_scale_val = imp.y_scale.value();
        let zoom_val = imp.size.value();
        let is_monochrome = imp.monochrome_switch.is_active() as u8;
        let monochrome_slider = imp.threshold_scale.value();
        let monochrome_red_val = imp.monochrome_color.rgba().red().to_string();
        let monochrome_green_val = imp.monochrome_color.rgba().green().to_string();
        let monochrome_blue_val = imp.monochrome_color.rgba().blue().to_string();
        let monochrome_inverted = imp.monochrome_invert.is_active() as u8;
        let is_default_monochrome = imp.monochrome_color.rgba() == self.get_default_color();
        debug!("is default? {}", is_default_monochrome);
        let combined_string = format!(
            "{}-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}",
            is_default,
            x_scale_val,
            y_scale_val,
            zoom_val,
            is_monochrome,
            monochrome_slider,
            monochrome_red_val,
            monochrome_green_val,
            monochrome_blue_val,
            monochrome_inverted,
            is_default_monochrome
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

    fn drag_connect_end(&self) {
        debug!("drag end");
        self.drag_and_drop_regeneration_popup();
    }

    pub fn setup_settings(&self) {
        let imp = self.imp();
        let update_folder = glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_: &gio::Settings, _: &str| {
                win.load_folder_path_from_settings();
            }
        );

        let resize_folder = glib::clone!(
            #[weak(rename_to = this)]
            self,
            move |_: &gio::Settings, _: &str| {
                glib::spawn_future_local(glib::clone!(
                    #[weak(rename_to = win)]
                    this,
                    async move {
                        let path: &str = &win.imp().settings.string("folder-svg-path");
                        win.load_folder_icon(path).await;
                    }
                ));
            }
        );

        let reload_thumbnails = glib::clone!(
            #[weak(rename_to = this)]
            self,
            move |_: &gio::Settings, _: &str| {
                glib::spawn_future_local(glib::clone!(
                    #[weak(rename_to = win)]
                    this,
                    async move {
                        let path: &str = &win.imp().settings.string("folder-svg-path");
                        win.load_folder_icon(path).await;
                        win.imp().reset_color.set_visible(true);
                    }
                ));
            }
        );

        adw::StyleManager::default().connect_accent_color_notify(glib::clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                if win.imp().stack.visible_child_name() != Some("regenerating_page".into()) {
                    // error!("Reloading folder image");
                    win.check_regeneration_needed();
                    win.load_folder_path_from_settings();
                }
                let _ = win.imp().settings.set_string(
                    "previous-system-accent-color",
                    &win.get_accent_color_and_show_dialog(),
                );
            }
        ));

        imp.settings
            .connect_changed(Some("folder-svg-path"), update_folder.clone());
        imp.settings
            .connect_changed(Some("primary-folder-color"), update_folder.clone());
        imp.settings
            .connect_changed(Some("secondary-folder-color"), update_folder.clone());
        imp.settings
            .connect_changed(Some("selected-accent-color"), update_folder.clone());
        imp.settings
            .connect_changed(Some("manual-bottom-image-selection"), update_folder.clone());
        imp.settings
            .connect_changed(Some("svg-render-size"), resize_folder.clone());
        imp.settings
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
                        if imp.monochrome_color.rgba() != win.get_default_color() {
                            imp.reset_color.set_visible(true);
                        }

                        // TODO: I do not like this approach, but it works
                        if imp.stack.visible_child_name() == Some("stack_main_page".into()) {
                            win.render_to_screen().await;
                        }
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
                    }
                ));
            }
        ));
    }

    pub fn image_save_sensitive(&self, sensitive: bool) {
        let imp = self.imp();
        imp.image_saved.replace(!sensitive);
        imp.save_button.set_sensitive(sensitive);
    }

    fn check_regeneration_needed(&self) -> bool {
        let imp = self.imp();
        let previous_accent: String = imp.settings.string("previous-system-accent-color").into();
        let current_accent = self.get_accent_color_and_show_dialog();
        // error!("previous {previous_accent} current {current_accent}");
        let id = *imp.regeneration_lock.borrow();
        imp.regeneration_lock.replace(id + 1);
        if previous_accent != current_accent && imp.settings.boolean("automatic-regeneration") {
            glib::spawn_future_local(glib::clone!(
                #[weak(rename_to = win)]
                self,
                async move {
                    match win.regenerate_icons().await {
                        Ok(_) => info!("Regeneration succesfull!"),
                        Err(x) => {
                            error!("{}", x.to_string());
                        }
                    };
                }
            ));
            return true;
        }
        false
    }

    pub fn reset_colors(&self) {
        let imp = self.imp();
        imp.reset_color.set_visible(false);
        imp.monochrome_color.set_rgba(&self.get_default_color());
        //self.check_icon_update();
        imp.reset_color.set_visible(false);
    }

    // TODO: This approach is dumb. I am purposely failing a dictionary lookup and using unwrap_or to get my way
    pub fn get_default_color(&self) -> gdk::RGBA {
        let imp = self.imp();
        let accent_color;
        debug!("Resetting top color");
        let selected_accent_color = imp.settings.string("selected-accent-color");
        let mut custom_rgb = RGBA::new(0.0, 0.0, 0.0, 0.0);
        if selected_accent_color == "None" {
            accent_color = self.get_accent_color_and_show_dialog();
        } else if imp.settings.boolean("manual-bottom-image-selection") {
            accent_color = "Blue".to_string();
        } else if selected_accent_color == "Custom" {
            accent_color = "Custom".to_string();
            custom_rgb = PreferencesDialog::hex_to_rgba(
                imp.settings.string("secondary-folder-color").into(),
            );
        } else {
            accent_color = selected_accent_color.into();
        }
        let color = imp
            .default_color
            .borrow()
            .get(&accent_color)
            .unwrap_or(&custom_rgb)
            .clone();
        debug!("Found color: {:?}", &color);
        color
    }

    pub async fn check_chache_icon(&self, file_name: &str) -> PathBuf {
        let imp = self.imp();
        let icon_path = PathBuf::from(&imp.settings.string("folder-svg-path"));
        let cache_path = self.get_cache_path();
        let folder_icon_cache_path = cache_path.join(file_name);
        if folder_icon_cache_path.exists() {
            info!("File found in cache at: {:?}", folder_icon_cache_path);
            return folder_icon_cache_path;
        } else if icon_path.exists() {
            info!(
                "File not found in cache, copying to: {:?}",
                folder_icon_cache_path
            );
            return self
                .copy_folder_image_to_cache(&icon_path, &cache_path)
                .await
                .unwrap()
                .0;
        }
        info!("File not found AT ALL");
        let dialog = show_error_popup(
            &self,
            &gettext("The set bottom icon could not be found, press ok to select a new one"),
            false,
            None,
        )
        .unwrap();
        match &*dialog.clone().choose_future(self).await {
            "OK" => {
                let new_path = match self.open_file_chooser().await {
                    Some(x) => x.path().unwrap().into_os_string().into_string().unwrap(),
                    None => {
                        self.application().unwrap().activate_action("quit", None);
                        return PathBuf::new();
                    }
                };
                imp.settings
                    .set_string("folder-svg-path", &new_path)
                    .unwrap();
                let cached_file_name = self
                    .copy_folder_image_to_cache(&PathBuf::from(new_path), &cache_path)
                    .await
                    .unwrap()
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

    pub fn get_cache_path(&self) -> PathBuf {
        let cache_path = match env::var("XDG_CACHE_HOME") {
            Ok(value) => PathBuf::from(value),
            Err(_) => {
                let config_dir = PathBuf::from(env::var("HOME").unwrap())
                    .join(".cache")
                    .join(format!("nl.emphisia.icon"));
                if !config_dir.exists() {
                    fs::create_dir(&config_dir).unwrap();
                }
                config_dir
            }
        };
        debug!("cache path {:?}", cache_path);
        cache_path
    }

    pub fn get_data_path(&self) -> PathBuf {
        let data_path = match env::var("XDG_DATA_HOME") {
            Ok(value) => PathBuf::from(value),
            Err(_) => {
                let config_dir = PathBuf::from(env::var("HOME").unwrap())
                    .join(".data")
                    .join("nl.emphisia.icon");
                if !config_dir.exists() {
                    fs::create_dir(&config_dir).unwrap();
                }
                config_dir
            }
        };
        data_path
    }

    // This checks if the main page, or welcome screen needs to be shown. And adds ability to loads just a bottom file
    // TODO: This function is REALLY confusing and needs to be rewritten
    pub fn check_icon_update(&self) {
        let imp = self.imp();
        let mut top_image = imp.top_image_file.lock().unwrap();
        let bottom_image = imp.bottom_image_file.lock().unwrap();
        if (*top_image).is_some() && (*bottom_image).is_some() {
            let top_image_width = top_image.as_ref().unwrap().dynamic_image.width();
            if top_image_width > 1 {
                // If the top image is empty, these controlls are disabled
                // This is to check if it's needed to turn them on again
                self.slider_control_sensitivity(true);
            }
            glib::spawn_future_local(glib::clone!(
                #[weak(rename_to = win)]
                self,
                async move {
                    win.render_to_screen().await;
                }
            ));
            imp.stack.set_visible_child_name("stack_main_page");
        } else if (*bottom_image).is_some() {
            let folder_bottom_name = bottom_image.as_ref().unwrap().filename.clone();
            debug!("Loaded temporary image for render");
            // Create image of nothing
            let empty_image = DynamicImage::new(1, 1, ColorType::Rgba8);
            (*top_image).replace(File::from_image(empty_image, 1, &folder_bottom_name));
            self.slider_control_sensitivity(false);

            if imp.stack.visible_child_name() != Some("stack_main_page".into()) {
                imp.stack.set_visible_child_name("stack_welcome_page");
            }
        }
    }

    pub fn slider_control_sensitivity(&self, sensitive: bool) {
        let imp = self.imp();
        imp.x_scale.set_sensitive(sensitive);
        imp.y_scale.set_sensitive(sensitive);
        imp.scale_row.set_sensitive(sensitive);
        imp.threshold_scale.set_sensitive(sensitive);
        imp.monochrome_color.set_sensitive(sensitive);
        imp.monochrome_invert.set_sensitive(sensitive);
        imp.monochrome_switch.set_sensitive(sensitive);
        imp.monochrome_action_row.set_sensitive(sensitive);
    }

    pub async fn open_file_chooser(&self) -> Option<gio::File> {
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
            Ok(file) => {
                debug!("{:?}", &file.path().unwrap());
                Some(file)
            }
            Err(error) => {
                error!("{:?}", error);
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
            show_error_popup(&self, "", true, Some(Box::new(e)));
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

    // TODO decouple UI components from these functions
    fn enable_monochrome_expand(&self) {
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
        if self.imp().stack.visible_child_name() == Some("stack_main_page".into()) {
            self.check_icon_update();
        }
    }
}
