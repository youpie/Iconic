use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use crate::glib::clone;
use gtk::*;
use gettextrs::*;
use crate::config::{APP_ID, PROFILE};
use std::path::PathBuf;
use std::{env,fs,path};
use adw::prelude::AlertDialogExt;
use adw::prelude::AdwDialogExt;
use crate::Results;

mod imp {
    use super::*;

    use adw::subclass::{prelude::PreferencesWindowImpl, window::AdwWindowImpl};

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/nl/emphisia/icon/settings/settings.ui")]
    pub struct PreferencesWindow {
        #[template_child()]
        pub custom: TemplateChild<adw::ActionRow>,
        #[template_child()]
        pub select_folder: TemplateChild<gtk::Button>,
        #[template_child()]
        pub custom1: TemplateChild<adw::ActionRow>,
        #[template_child()]
        pub svg_image_size: TemplateChild<adw::SpinRow>,
        #[template_child()]
        pub advanced_settings: TemplateChild<adw::PreferencesGroup>,
        #[template_child()]
        pub thumbnail_image_size: TemplateChild<adw::SpinRow>,
        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "PreferencesWindow";
        type Type = super::PreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn new() -> Self {
            Self {
                custom: TemplateChild::default(),
                select_folder: TemplateChild::default(),
                custom1: TemplateChild::default(),
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
                glib::spawn_future_local(clone!(@weak win => async move {
                    win.select_path_filechooser();
                }));
            });
            klass.install_action("app.reset_location", None, move |win, _, _| {
                glib::spawn_future_local(clone!(@weak win => async move {
                    win.reset_location_fn();
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesWindow {
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

    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl AdwWindowImpl for PreferencesWindow {}
    impl PreferencesWindowImpl for PreferencesWindow {}
}

glib::wrapper! {
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
    @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;
}

#[gtk::template_callbacks]
impl PreferencesWindow {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let win = glib::Object::new::<Self>();
        win.setup_settings();
        if PROFILE != "Devel" {
            win.imp().advanced_settings.set_visible(false);
        }
        win.set_path_title();
        win
    }

    fn setup_settings(&self){
        let current_value: i32 = self.imp().settings.get("svg-render-size");
        self.imp().svg_image_size.set_value(current_value as f64);
        self.imp().svg_image_size.connect_changed(clone!(@weak self as this => move |_| {
            let value = this.imp().svg_image_size.value() as i32;
            println!("{}",value);
            let _ = this.imp().settings.set("svg-render-size",value);
        }));
        let current_value: i32 = self.imp().settings.get("thumbnail-size");
        self.imp().thumbnail_image_size.set_value(current_value as f64);
        self.imp().thumbnail_image_size.connect_changed(clone!(@weak self as this => move |_| {
            let value = this.imp().thumbnail_image_size.value() as i32;
            println!("{}",value);
            let _ = this.imp().settings.set("thumbnail-size",value);
        }));
    }

    fn reset_location_fn(&self){
        let mut default_value = self.imp().settings.default_value("folder-svg-path").unwrap().to_string();
        default_value.pop();
        default_value.remove(0);
        self.can_error(self.set_path(&default_value));
    }

    fn set_path_title (&self){
        let current_path = &self.imp().settings.string("folder-svg-path");
        self.imp().custom1.set_property("title",current_path);
    }

    fn select_path_filechooser (&self) {
        glib::spawn_future_local(glib::clone!(@weak self as window => async move {
            let filters = gio::ListStore::new::<gtk::FileFilter>();
            let filter = gtk::FileFilter::new();
            filter.add_mime_type("image/*");
            filters.append(&filter);
            let dialog = gtk::FileDialog::builder()
                    .title(gettext("Open Document"))
                    .modal(true)
                    .filters(&filters)
                    .build();
            let file = dialog.open_future(Some(&window)).await;

            match file {
                Ok(x) => {  println!("{:#?}",&x.path().unwrap());
                            let path: &str = &x.path().unwrap().into_os_string().into_string().unwrap();
                            window.can_error(window.set_path(path));},
                Err(y) => {println!("{:#?}",y);},
            }
        }));
    }

    fn set_path(&self,path: &str) -> Results<()>{
        self.copy_folder_image_to_cache(path::PathBuf::from(path))?;
        self.imp().settings.set("folder-svg-path", path)?;
        self.set_path_title();
        Ok(())
    }

    fn copy_folder_image_to_cache(&self, original_path: path::PathBuf) -> Results<()>{
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
        let file_name = format!("folder.{}",original_path.extension().unwrap().to_str().unwrap());
        self.imp().settings.set("folder-cache-name",file_name.clone())?;
        let cache_path = cache_dir.join(file_name);
        fs::copy(original_path,cache_path)?;
        Ok(())
    }

    fn can_error <T>(&self,result:Results<T>){
        let _ = result.map_err(|e|
        {const RESPONSE_OK: &str = "ok";
        let dialog = adw::AlertDialog::builder()
                .heading(gettext("Error"))
                .body(&e.to_string())
                .default_response(RESPONSE_OK)
                .build();
        dialog.add_response(RESPONSE_OK, "ok");
        dialog.present(self)});
    }
}

