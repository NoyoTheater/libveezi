#![doc = include_str!("../README.md")]
#![allow(unknown_lints)] // in case you use non-nightly clippy
#![warn(
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic,
    clippy::missing_docs_in_private_items,
    missing_docs,
    clippy::absolute_paths,
    clippy::as_conversions,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::deref_by_slicing,
    clippy::disallowed_script_idents,
    clippy::else_if_without_else,
    clippy::empty_structs_with_brackets,
    clippy::format_push_string,
    clippy::if_then_some_else_none,
    clippy::let_underscore_must_use,
    clippy::min_ident_chars,
    clippy::mixed_read_write_in_expression,
    clippy::multiple_inherent_impl,
    clippy::multiple_unsafe_ops_per_block,
    clippy::non_ascii_literal,
    clippy::redundant_type_annotations,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::semicolon_inside_block,
    clippy::unseparated_literal_suffix,
    clippy::implicit_clone,
    clippy::todo,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unneeded_field_pattern,
    clippy::wildcard_enum_match_arm,
    unused_qualifications,
    clippy::unwrap_used,
    clippy::print_stderr,
    clippy::print_stdout
)]
#![allow(
    clippy::multiple_crate_versions,
    clippy::cargo_common_metadata,
    clippy::module_name_repetitions,
    clippy::doc_comment_double_space_linebreaks
)]

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
