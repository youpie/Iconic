use crate::IconicWindow;
use crate::glib;
use adw::subclass::prelude::*;
use gio::glib::object::Cast;
use gio::glib::object::ObjectExt;
use gtk::GestureClick;
use gtk::GestureLongPress;
use gtk::gdk;
use gtk::prelude::PopoverExt;
use log::debug;

#[gtk::template_callbacks]
impl IconicWindow {
    #[template_callback]
    pub async fn render_callback(&self, _hoi: glib::Object) {
        if self.imp().stack.visible_child_name() == Some("stack_main_page".into()) {
            self.render_to_screen().await;
        } else {
            debug!("not main stack");
        }
    }

    #[template_callback]
    pub async fn open_popover(&self, #[rest] values: &[glib::Value]) {
        let imp = self.imp();
        if let Some(gesture) = values.last().and_then(|x| x.get::<glib::Object>().ok()) {
            let (x, y) = if gesture.downcast_ref::<GestureLongPress>().is_some() {
                debug!("Longpress");
                (
                    values[0].get::<f64>().unwrap_or_default(),
                    values[1].get::<f64>().unwrap_or_default(),
                )
            } else if gesture.downcast_ref::<GestureClick>().is_some() {
                debug!("Click");
                (
                    values[1].get::<f64>().unwrap_or_default(),
                    values[2].get::<f64>().unwrap_or_default(),
                )
            } else {
                debug!("Neither, {}", gesture.type_().name());
                (0.0, 0.0)
            };
            debug!("X: {x}, Y:{y}");
            let position = gdk::Rectangle::new(x as i32, y as i32, 0, 0);
            imp.popover_menu.set_pointing_to(Some(&position));
            imp.popover_menu.popup();
        }
    }
}
