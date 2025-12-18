use gio::glib::subclass::InitializingObject;
use gtk::glib;
use gtk::prelude::WidgetExt;
use gtk::subclass::prelude::*;

#[derive(Default, gtk::CompositeTemplate)]
#[template(resource = "/nl/emphisia/icon/windows/preview_window/window.ui")]
pub struct PreviewWindow {
    #[template_child]
    pub image_preview: TemplateChild<gtk::Picture>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for PreviewWindow {
    const NAME: &'static str = "PreviewWindow";
    type Type = super::PreviewWindow;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for PreviewWindow {
    fn constructed(&self) {
        self.parent_constructed();
        self.image_preview.set_cursor_from_name(Some("pointer"));
    }
}

// Trait shared by all widgets
impl WidgetImpl for PreviewWindow {}

impl BoxImpl for PreviewWindow {}
