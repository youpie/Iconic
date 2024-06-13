use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use crate::glib::clone;
use crate::GtkTestWindow;
use gtk::*;
use gio::*;
use glib::*;
use gettextrs::*;
use crate::config::{APP_ID};

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
        pub svg_image_size_row: TemplateChild<adw::SpinRow>,
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
                svg_image_size_row: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("app.select_folder", None, move |win, _, _| {
                glib::spawn_future_local(clone!(@weak win => async move {
                    win.select_path();
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
            // self.settings
            //     .bind("svg-render-size", &*self.svg_image_size_row, "value")
            //     .build();

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
        win.imp().svg_image_size_row.connect_changed(clone!(@weak win as this => move |_| {
            let value = this.imp().svg_image_size_row.value() as i32;
            let _ = this.imp().settings.set("svg-render-size",value);
        }));

        win.set_path_title();
        win
    }

    fn set_path_title (&self){
        let current_path = &self.imp().settings.string("folder-svg-path");
        self.imp().custom1.set_property("title",current_path);
    }

    fn select_path (&self) {
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
                Ok(x) => {println!("{:#?}",&x.path().unwrap());
                            let path: &str = &x.path().unwrap().into_os_string().into_string().unwrap();
                            window.imp().settings.set("folder-svg-path", path).unwrap();
                            window.set_path_title();},
                Err(y) => {println!("{:#?}",y);
                            },
            }
        }));
    }
}

