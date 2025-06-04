use crate::GtkTestWindow;
use crate::glib;
use adw::subclass::prelude::*;
use log::debug;

#[gtk::template_callbacks]
impl GtkTestWindow {
    #[template_callback]
    pub async fn render_callback(&self, _hoi: glib::Object) {
        if self.imp().stack.visible_child_name() == Some("stack_main_page".into()) {
            self.render_to_screen().await;
        } else {
            debug!("not main stack");
        }
    }
}
