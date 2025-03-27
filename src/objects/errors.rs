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
    let error_message = match message {
        "" => {
            if let Some(error_value) = &error {
                error_value.to_string()
            } else {
                "unknown error".to_string()
            }
        }
        _ => message.to_string(),
    };
    let dialog = adw::AlertDialog::builder()
        .heading(format!(
            "<span foreground=\"red\"><b>⚠ {error_text}</b></span>"
        ))
        .heading_use_markup(true)
        .body(error_message)
        .default_response(RESPONSE_OK)
        .close_response(RESPONSE_OK)
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

pub trait IntoResult<T> {
    fn into_result(self) -> Result<T, Box<dyn Error>>;
    fn into_reason_result(self, reason: &'static str) -> Result<T, Box<dyn Error>>;
}

impl<T> IntoResult<T> for Option<T> {
    fn into_result(self) -> Result<T, Box<dyn Error>> {
        match self {
            Some(value) => Ok(value),
            None => Err(Box::new(OptionError::NoneUnwrap(None))),
        }
    }
    fn into_reason_result(self, reason: &'static str) -> Result<T, Box<dyn Error>> {
        match self {
            Some(value) => Ok(value),
            None => Err(Box::new(OptionError::NoneUnwrap(Some(reason)))),
        }
    }
}
