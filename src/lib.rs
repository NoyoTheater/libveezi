#![doc = include_str!("../README.md")]
mod utils;

pub mod error;
#[deprecated(since = "0.3.0", note = "Use `libveezi::error::<item>` instead")]
pub use error::*;

pub mod client;
#[deprecated(since = "0.3.0", note = "Use `libveezi::client::<item>` instead")]
pub use client::*;

pub mod session;
#[deprecated(since = "0.3.0", note = "Use `libveezi::session::<item>` instead")]
pub use session::*;

pub mod film;
#[deprecated(since = "0.3.0", note = "Use `libveezi::film::<item>` instead")]
pub use film::*;

pub mod package;
#[deprecated(since = "0.3.0", note = "Use `libveezi::package::<item>` instead")]
pub use package::*;

pub mod screen;
#[deprecated(since = "0.3.0", note = "Use `libveezi::screen::<item>` instead")]
pub use screen::*;

pub mod site;
#[deprecated(since = "0.3.0", note = "Use `libveezi::site::<item>` instead")]
pub use site::*;

pub mod attr;
#[deprecated(since = "0.3.0", note = "Use `libveezi::attr::<item>` instead")]
pub use attr::*;
