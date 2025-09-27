use crate::GenResult;
use crate::config::{APP_ID, PROFILE};
use crate::glib::clone;
use adw::prelude::AlertDialogExt;
use adw::prelude::AlertDialogExtManual;
use adw::prelude::ComboRowExt;
use adw::prelude::{ActionRowExt, AdwDialogExt};
use adw::subclass::prelude::AdwDialogImpl;
use fs_extra;
use gettextrs::*;
use gio::AppInfo;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::*;
use log::*;
use std::path::PathBuf;
use std::{env, fs, path};

use crate::GtkTestWindow;

mod imp {
    use super::*;

    use adw::subclass::prelude::PreferencesDialogImpl;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/nl/emphisia/icon/settings/settings.ui")]
    pub struct PreferencesDialog {
        #[template_child]
        pub current_botton: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub svg_image_size: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub advanced_settings: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub default_dnd: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub dnd_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub radio_button_top: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub radio_button_bottom: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub thumbnail_image_size: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub select_bottom_color: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub use_builtin_icons_button: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub use_external_icon_button: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub use_external_icon_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub use_builtin_icons_expander: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub use_system_color: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub store_top_images: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub ignore_custom: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub strict_regeneration: TemplateChild<gtk::Switch>,
        #[template_child]
        pub automatic_regeneration: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub cache_size: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub reset_top_cache: TemplateChild<adw::ButtonRow>,
        #[template_child]
        pub primary_color_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub primary_folder_color: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub secondary_color_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub secondary_folder_color: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub meta_drop_switch: TemplateChild<adw::SwitchRow>,
        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesDialog {
        const NAME: &'static str = "PreferencesDialog";
        type Type = super::PreferencesDialog;
        type ParentType = adw::PreferencesDialog;

        fn new() -> Self {
            Self {
                default_dnd: TemplateChild::default(),
                dnd_switch: TemplateChild::default(),
                radio_button_top: TemplateChild::default(),
                radio_button_bottom: TemplateChild::default(),
                current_botton: TemplateChild::default(),
                svg_image_size: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                advanced_settings: TemplateChild::default(),
                thumbnail_image_size: TemplateChild::default(),
                select_bottom_color: TemplateChild::default(),
                use_builtin_icons_button: TemplateChild::default(),
                use_external_icon_button: TemplateChild::default(),
                use_external_icon_expander: TemplateChild::default(),
                use_builtin_icons_expander: TemplateChild::default(),
                use_system_color: TemplateChild::default(),
                store_top_images: TemplateChild::default(),
                automatic_regeneration: TemplateChild::default(),
                cache_size: TemplateChild::default(),
                reset_top_cache: TemplateChild::default(),
                primary_color_row: TemplateChild::default(),
                primary_folder_color: TemplateChild::default(),
                secondary_color_row: TemplateChild::default(),
                secondary_folder_color: TemplateChild::default(),
                meta_drop_switch: TemplateChild::default(),
                strict_regeneration: TemplateChild::default(),
                ignore_custom: TemplateChild::default(),
                // reveal_custom_colors: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("app.select_folder", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.select_path_filechooser();
                    }
                ));
            });
            klass.install_action("app.reset_color_primary", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.imp()
                            .primary_folder_color
                            .set_rgba(&GtkTestWindow::to_rgba(164, 202, 238));
                    }
                ));
            });
            klass.install_action("app.reset_color_secondary", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.imp()
                            .secondary_folder_color
                            .set_rgba(&GtkTestWindow::to_rgba(67, 141, 230));
                    }
                ));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesDialog {
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

    impl WidgetImpl for PreferencesDialog {}
    // impl WindowImpl for PreferencesDialog {}
    impl AdwDialogImpl for PreferencesDialog {}
    impl PreferencesDialogImpl for PreferencesDialog {}
}

glib::wrapper! {
    pub struct PreferencesDialog(ObjectSubclass<imp::PreferencesDialog>)
    @extends gtk::Widget, gtk::Window, adw::Dialog, adw::PreferencesDialog,
    @implements
        gtk::Accessible,
        gtk::Buildable,
        gtk::ConstraintTarget,
        gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl PreferencesDialog {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let win = glib::Object::new::<Self>();
        let imp = win.imp();
        if PROFILE != "Devel" {
            win.imp().advanced_settings.set_visible(false);
        }
        win.imp()
            .dnd_switch
            .set_active(win.imp().settings.boolean("default-dnd-activated"));
        if imp.settings.string("selected-accent-color") == "None" {
            imp.use_system_color.set_active(true);
        }
        if imp.settings.boolean("manual-bottom-image-selection") {
            imp.use_external_icon_button.set_active(true);
        }
        if imp.settings.string("default-dnd-action") == "bottom" {
            imp.radio_button_bottom.set_active(true);
        }
        imp.select_bottom_color
            .set_selected(imp.settings.uint("selected-accent-color-index"));
        win.load_set_colors();
        win.set_path_title();
        win.disable_color_dropdown(true);
        win.setup_settings();
        win.get_file_size();
        win.show_color_options();
        win
    }

    fn load_set_colors(&self) {
        let imp = self.imp();
        let current_primary = imp.settings.string("primary-folder-color");
        let current_secondary = imp.settings.string("secondary-folder-color");
        imp.primary_folder_color
            .set_rgba(&PreferencesDialog::hex_to_rgba(current_primary.to_string()));
        imp.secondary_folder_color
            .set_rgba(&PreferencesDialog::hex_to_rgba(
                current_secondary.to_string(),
            ));
    }

    fn setup_settings(&self) {
        let imp = self.imp();
        let current_value: u32 = imp.settings.get("svg-render-size");
        imp.settings
            .bind("store-top-in-cache", &*imp.store_top_images, "active")
            .build();
        imp.settings
            .bind(
                "automatic-regeneration",
                &*imp.automatic_regeneration,
                "active",
            )
            .build();
        imp.settings
            .bind("allow-meta-drop", &*imp.meta_drop_switch, "active")
            .build();
        imp.settings
            .bind("default-dnd-activated", &*imp.dnd_switch, "active")
            .build();
        imp.settings
            .bind("strict-regeneration", &*imp.strict_regeneration, "active")
            .build();
        imp.settings
            .bind("ignore-custom", &*imp.ignore_custom, "active")
            .build();
        imp.settings
            .bind(
                "manual-bottom-image-selection",
                &*imp.use_external_icon_button,
                "active",
            )
            .build();
        imp.svg_image_size.set_value(current_value as f64);
        imp.svg_image_size.connect_changed(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                let value = win.imp().svg_image_size.value() as u32;
                debug!("{}", value);
                let _ = win.imp().settings.set("svg-render-size", value);
            }
        ));
        let current_value: u32 = imp.settings.get("thumbnail-size");
        imp.thumbnail_image_size.set_value(current_value as f64);
        imp.thumbnail_image_size.connect_changed(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                let value = win.imp().thumbnail_image_size.value() as u32;
                debug!("{}", value);
                let _ = win.imp().settings.set("thumbnail-size", value);
            }
        ));
        imp.select_bottom_color.connect_selected_item_notify(clone!(
            #[weak (rename_to = this)]
            self,
            move |_| {
                this.get_selected_accent_color(false);
                this.show_color_options();
            }
        ));

        imp.radio_button_top.connect_toggled(clone!(
            #[weak (rename_to = this)]
            self,
            move |_| {
                this.dnd_radio_state();
            }
        ));
        imp.use_system_color.connect_active_notify(clone!(
            #[weak (rename_to = this)]
            self,
            move |_| {
                this.disable_color_dropdown(false);
                this.show_color_options();
            }
        ));
        imp.reset_top_cache.connect_activated(clone!(
            #[weak (rename_to = this)]
            self,
            move |_| {
                glib::spawn_future_local(clone!(
                    #[weak (rename_to = win)]
                    this,
                    async move {
                        win.on_buttonrow_activated().await;
                    }
                ));
            }
        ));
        imp.primary_folder_color.connect_rgba_notify(clone!(
            #[weak (rename_to = this)]
            self,
            move |_| {
                let color = this.imp().primary_folder_color.rgba();
                let _ = this
                    .imp()
                    .settings
                    .set_string("primary-folder-color", &this.rgba_to_hex(color));
            }
        ));
        imp.secondary_folder_color.connect_rgba_notify(clone!(
            #[weak (rename_to = this)]
            self,
            move |_| {
                let color = this.imp().secondary_folder_color.rgba();
                let _ = this
                    .imp()
                    .settings
                    .set_string("secondary-folder-color", &this.rgba_to_hex(color));
            }
        ));
    }

    fn disable_color_dropdown(&self, init: bool) {
        let imp = self.imp();
        let switch_state = imp.use_system_color.is_active();
        match switch_state {
            true => {
                if !init {
                    let _ = imp.settings.set("selected-accent-color", "None");
                }
            }
            false => {
                self.get_selected_accent_color(init);
            }
        };
    }

    fn get_file_size(&self) {
        let imp = self.imp();
        let mut path = self.get_cache_path();
        path.push("top_images");
        let file_size = fs_extra::dir::get_size(path).unwrap_or(0);
        let file_size_float: f64 = file_size as f64 / 1000000.0;
        imp.cache_size
            .set_subtitle(&format!("{:.2} MB", file_size_float));
    }

    fn get_selected_accent_color(&self, init: bool) {
        let color_vec = vec![
            "Blue", "Teal", "Green", "Yellow", "Orange", "Red", "Pink", "Purple", "Slate", "Custom",
        ];
        let imp = self.imp();
        let selected_index = imp.select_bottom_color.selected() as usize;
        let selected_color = color_vec[selected_index];
        debug!("Selected accent color: {selected_color}");
        if !init {
            let _ = imp.settings.set("selected-accent-color", selected_color);
            let _ = imp
                .settings
                .set("selected-accent-color-index", selected_index as u32);
        }
    }

    fn set_path_title(&self) {
        let current_path = &self.imp().settings.string("folder-svg-path");
        self.imp()
            .current_botton
            .set_property("title", current_path);
    }

    fn select_path_filechooser(&self) {
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = win)]
            self,
            async move {
                let filters = gio::ListStore::new::<gtk::FileFilter>();
                let filter = gtk::FileFilter::new();
                filter.add_mime_type("image/*");
                filters.append(&filter);
                let dialog = gtk::FileDialog::builder()
                    .title(gettext("Open Document"))
                    .modal(true)
                    .filters(&filters)
                    .build();
                let file = dialog.open_future(Some(&win)).await;

                match file {
                    Ok(x) => {
                        info!("{:#?}", &x.path().unwrap());
                        let path: &str = &x.path().unwrap().into_os_string().into_string().unwrap();
                        win.can_error(win.set_path(path));
                    }
                    Err(y) => {
                        warn!("{:#?}", y);
                    }
                }
            }
        ));
    }

    fn set_path(&self, path: &str) -> GenResult<()> {
        self.copy_folder_image_to_cache(path::PathBuf::from(path))?;
        self.imp().settings.set("folder-svg-path", path)?;
        self.set_path_title();
        Ok(())
    }

    fn copy_folder_image_to_cache(&self, original_path: path::PathBuf) -> GenResult<()> {
        let cache_dir = self.get_cache_path();
        let file_name = format!(
            "folder.{}",
            original_path.extension().unwrap().to_str().unwrap()
        );
        self.imp()
            .settings
            .set("folder-cache-name", file_name.clone())?;
        let cache_path = cache_dir.join(file_name);
        fs::copy(original_path, cache_path)?;
        Ok(())
    }

    fn can_error<T>(&self, result: GenResult<T>) {
        let _ = result.map_err(|e| {
            const RESPONSE_OK: &str = "OK";
            let dialog = adw::AlertDialog::builder()
                .heading(gettext("Error"))
                .body(&e.to_string())
                .default_response(RESPONSE_OK)
                .build();
            dialog.add_response(RESPONSE_OK, "ok");
            dialog.present(Some(self))
        });
    }

    async fn on_buttonrow_activated(&self) {
        const RESPONSE_REMOVE: &str = "remove";
        const RESPONSE_CANCEL: &str = "cancel";
        let dialog = adw::AlertDialog::builder()
        .heading(format!(
            "<span foreground=\"red\"><b>Confirm Cache Removal</b></span>"
        ))
        .heading_use_markup(true)
            .body(&gettext("Are you sure you want to clear the cache? \n Clearing the cache means you probably won't be able to regenerate a lot of images."))
            .default_response(RESPONSE_CANCEL)
            .build();
        dialog.add_response(RESPONSE_CANCEL, &gettext("Cancel"));
        dialog.set_response_appearance(RESPONSE_CANCEL, adw::ResponseAppearance::Default);
        dialog.add_response(RESPONSE_REMOVE, &gettext("Remove"));
        dialog.set_response_appearance(RESPONSE_REMOVE, adw::ResponseAppearance::Destructive);

        match &*dialog.clone().choose_future(self).await {
            RESPONSE_CANCEL => {
                dialog.close();
            }
            RESPONSE_REMOVE => {
                self.can_error(self.remove_cache_folder());
                self.get_file_size();
            }
            _ => unreachable!(),
        }
    }

    // This might turn out bad, but iconic does not have file persmission so it is probably fine :D
    fn remove_cache_folder(&self) -> GenResult<()> {
        let mut path = self.get_cache_path();
        path.push("top_images");
        fs::remove_dir_all(&path)?;
        fs::create_dir(&path)?;
        Ok(())
    }

    pub fn dnd_radio_state(&self) {
        let imp = self.imp();
        let radio_button = imp.radio_button_top.is_active();
        debug!("Radio button changed: button 1 is {}", radio_button);
        match radio_button {
            true => {
                let _ = imp.settings.set("default-dnd-action", "top");
            }
            false => {
                let _ = imp.settings.set("default-dnd-action", "bottom");
            }
        }
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

    #[template_callback]
    pub async fn open_image_cache(&self, _button: adw::ButtonRow) {
        let file = gio::File::for_path(format!(
            "{}/top_images/",
            GtkTestWindow::get_cache_path().to_str().unwrap()
        ))
        .uri();
        if let Err(e) = AppInfo::launch_default_for_uri(&file, None::<&gio::AppLaunchContext>) {
            self.can_error::<()>(Err(std::boxed::Box::new(e)));
        };
    }
}
