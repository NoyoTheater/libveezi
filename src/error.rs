//! Useful error types
//!
//! The primary error type is [`LibVeeziError`], which encapsulates errors that can occur
//! when using the libveezi library.

use std::fmt::{self, Debug};

/// The list of errors that can occur when using the libveezi library
#[derive(Debug)]
pub enum LibVeeziError {
    /// An error occurred while making an HTTP request
    Http(reqwest::Error),
    /// An error occurred while parsing a URL
    UrlParse(url::ParseError),
}
impl fmt::Display for LibVeeziError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibVeeziError::Http(err) => write!(f, "HTTP error: {}", err),
            LibVeeziError::UrlParse(err) => write!(f, "URL parse error: {}", err),
        }
    }
}
impl std::error::Error for LibVeeziError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LibVeeziError::Http(err) => Some(err),
            LibVeeziError::UrlParse(err) => Some(err),
        }
    }
}
impl From<reqwest::Error> for LibVeeziError {
    fn from(err: reqwest::Error) -> Self {
        LibVeeziError::Http(err)
    }
}
impl From<url::ParseError> for LibVeeziError {
    fn from(err: url::ParseError) -> Self {
        LibVeeziError::UrlParse(err)
    }
}

/// A result type for the libveezi library
pub type ApiResult<T> = Result<T, LibVeeziError>;
