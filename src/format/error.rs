use std::io;
use rust_i18n::t;

#[derive(Debug)]
pub enum FormatError {
    Io(io::Error),
    InvalidData(String),
    UnexpectedEof(String),
    InvalidSliceLength,
    InvalidUtf8(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::Io(err) => write!(f, "{}", t!("error.io_error", err = err.to_string())),
            FormatError::InvalidData(msg) => write!(f, "{}", t!("error.invalid_data", msg = msg)),
            FormatError::UnexpectedEof(msg) => write!(f, "{}", t!("error.unexpected_eof", msg = msg)),
            FormatError::InvalidSliceLength => write!(f, "{}", t!("error.invalid_slice_length")),
            FormatError::InvalidUtf8(msg) => write!(f, "{}", t!("error.invalid_utf8", msg = msg)),
        }
    }
}

impl std::error::Error for FormatError {}

impl From<io::Error> for FormatError {
    fn from(err: io::Error) -> Self { FormatError::Io(err) }
}

pub type Result<T> = std::result::Result<T, FormatError>;