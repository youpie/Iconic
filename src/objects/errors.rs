use crate::window::GtkTestWindow;
use adw::prelude::*;
use gettextrs::gettext;
use log::*;
use std::error::Error;

pub fn show_error_popup(
    window: &GtkTestWindow,
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
            dialog.present(Some(window));
            None
        }
        false => Some(dialog),
    }
}

#[derive(Debug, thiserror::Error)]
enum OptionError {
    #[error("Unwrapped on a None value. (optional)reason: {0:?}")]
    NoneUnwrap(Option<&'static str>),
}

trait IntoResult<T> {
    fn into_result(self) -> Result<T, OptionError>;
}

impl<T> IntoResult<T> for Option<T> {
    fn into_result(self) -> Result<T, OptionError> {
        match self {
            Some(value) => Ok(value),
            None => Err(OptionError::NoneUnwrap(None)),
        }
    }
}
