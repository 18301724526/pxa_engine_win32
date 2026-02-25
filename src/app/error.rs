use std::io;
use crate::format::error::FormatError;
use rust_i18n::t;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Image(image::ImageError),
    Format(FormatError),
    VersionTooHigh,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "{}", t!("error.io_error", err = err.to_string())),
            AppError::Image(err) => write!(f, "{}", t!("error.image_error", err = err.to_string())),
            AppError::Format(err) => write!(f, "{}", t!("error.format_error", err = err.to_string())),
            AppError::VersionTooHigh => write!(f, "{}", t!("error.version_too_high")),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError { fn from(err: io::Error) -> Self { AppError::Io(err) } }
impl From<image::ImageError> for AppError { fn from(err: image::ImageError) -> Self { AppError::Image(err) } }
impl From<FormatError> for AppError { fn from(err: FormatError) -> Self { AppError::Format(err) } }

pub type Result<T> = std::result::Result<T, AppError>;