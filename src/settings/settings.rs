use crate::config::{APP_ID, PROFILE};
use crate::glib::clone;
use crate::Results;
use adw::prelude::AdwDialogExt;
use adw::prelude::AlertDialogExt;
use adw::subclass::prelude::AdwDialogImpl;
use gettextrs::*;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::*;
use log::*;
use std::path::PathBuf;
use std::{env, fs, path};

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
        pub radio_button_1: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub thumbnail_image_size: TemplateChild<adw::SpinRow>,
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
                radio_button_1: TemplateChild::default(),
                current_botton: TemplateChild::default(),
                svg_image_size: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                advanced_settings: TemplateChild::default(),
                thumbnail_image_size: TemplateChild::default(),
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
            klass.install_action("app.reset_location", None, move |win, _, _| {
                glib::spawn_future_local(clone!(
                    #[weak]
                    win,
                    async move {
                        win.reset_location_fn();
                    }
                ));
            });
            klass.install_action("app.dnd_switch", None, move |win, _, _| {
                win.dnd_row_expand();
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
    impl WindowImpl for PreferencesDialog {}
    impl AdwDialogImpl for PreferencesDialog {}
    impl PreferencesDialogImpl for PreferencesDialog {}
}

glib::wrapper! {
    pub struct PreferencesDialog(ObjectSubclass<imp::PreferencesDialog>)
    @extends gtk::Widget, gtk::Window, adw::Dialog, adw::PreferencesDialog;
}

#[gtk::template_callbacks]
impl PreferencesDialog {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let win = glib::Object::new::<Self>();
        win.setup_settings();
        if PROFILE != "Devel" {
            win.imp().advanced_settings.set_visible(false);
        }
        win.imp()
            .dnd_switch
            .set_active(win.imp().settings.boolean("default-dnd-activated"));
        win.dnd_row_expand();
        win.set_path_title();

        win.imp().radio_button_1.connect_toggled(clone!(
            #[weak (rename_to = this)]
            win,
            move |_| {
                this.dnd_radio_state();
            }
        ));
        win
    }

    fn setup_settings(&self) {
        let current_value: i32 = self.imp().settings.get("svg-render-size");
        self.imp().svg_image_size.set_value(current_value as f64);
        self.imp().svg_image_size.connect_changed(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                let value = win.imp().svg_image_size.value() as i32;
                debug!("{}", value);
                let _ = win.imp().settings.set("svg-render-size", value);
            }
        ));
        let current_value: i32 = self.imp().settings.get("thumbnail-size");
        self.imp()
            .thumbnail_image_size
            .set_value(current_value as f64);
        self.imp().thumbnail_image_size.connect_changed(clone!(
            #[weak(rename_to = win)]
            self,
            move |_| {
                let value = win.imp().thumbnail_image_size.value() as i32;
                debug!("{}", value);
                let _ = win.imp().settings.set("thumbnail-size", value);
            }
        ));
    }

    fn reset_location_fn(&self) {
        let mut default_value = self
            .imp()
            .settings
            .default_value("folder-svg-path")
            .unwrap()
            .to_string();
        default_value.pop();
        default_value.remove(0);
        self.can_error(self.set_path(&default_value));
    }

    fn set_path_title(&self) {
        let current_path = &self.imp().settings.string("folder-svg-path");
        self.imp().current_botton.set_property("title", current_path);
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

    fn set_path(&self, path: &str) -> Results<()> {
        self.copy_folder_image_to_cache(path::PathBuf::from(path))?;
        self.imp().settings.set("folder-svg-path", path)?;
        self.set_path_title();
        Ok(())
    }

    fn copy_folder_image_to_cache(&self, original_path: path::PathBuf) -> Results<()> {
        let cache_dir = match env::var("XDG_CACHE_HOME") {
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

    fn can_error<T>(&self, result: Results<T>) {
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

    pub fn dnd_row_expand(&self) {
        let switch_state = self.imp().dnd_switch.is_active();
        let _ = self
            .imp()
            .settings
            .set("default-dnd-activated", switch_state);
        debug!("Current switch state: {}", switch_state);
        match switch_state {
            true => {
                self.imp()
                    .default_dnd
                    .set_property("enable_expansion", false);
                self.imp()
                    .default_dnd
                    .set_property("enable_expansion", true);
            }
            false => {
                self.imp()
                    .default_dnd
                    .set_property("enable_expansion", false);
            }
        };
    }

    pub fn dnd_radio_state(&self) {
        let imp = self.imp();
        let radio_button = imp.radio_button_1.is_active();
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
}
