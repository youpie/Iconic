mod imp;

use adw::subclass::prelude::ObjectSubclassIsExt;
use gdk4::Texture;
use gtk::glib;

glib::wrapper! {
    pub struct PreviewWindow(ObjectSubclass<imp::PreviewWindow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl PreviewWindow {
    pub fn set_paintable(&self, paintable: &Texture) {
        self.imp().image_preview.set_paintable(Some(paintable));
    }
}
