//! Useful error types
//!
//! The primary error type is [`LibVeeziError`], which encapsulates errors that
//! can occur when using the libveezi library.

use std::{
    error::Error,
    fmt::{self, Debug, Display},
};

/// The list of errors that can occur when using the libveezi library
#[derive(Debug)]
pub enum LibVeeziError {
    /// An error occurred while making an HTTP request
    Http(reqwest::Error),
    /// An error occurred while parsing a URL
    UrlParse(url::ParseError),
}
impl Display for LibVeeziError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(err) => write!(f, "HTTP error: {err}"),
            Self::UrlParse(err) => write!(f, "URL parse error: {err}"),
        }
    }
}
impl Error for LibVeeziError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Http(err) => Some(err),
            Self::UrlParse(err) => Some(err),
        }
    }
}
impl From<reqwest::Error> for LibVeeziError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err)
    }
}
impl From<url::ParseError> for LibVeeziError {
    fn from(err: url::ParseError) -> Self {
        Self::UrlParse(err)
    }
}

/// A result type for the libveezi library
pub type ApiResult<T> = Result<T, LibVeeziError>;
