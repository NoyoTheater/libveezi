//! Types representing screenable film packages (e.g. double features) with the Veezi API.
//!
//! The primary type is [`FilmPackage`], which represents a film package and its metadata.

use crate::client::Client;
use crate::error::ApiResult;
use crate::film::{Film, FilmStatus};
use serde::Deserialize;
use std::fmt::Debug;

/// A particular film within a [FilmPackage]
#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PackageFilm {
    /// The unique ID of the film
    pub film_id: String,
    /// The title of the film
    pub title: String,
    /// What percent of the box office this film receives within the package
    pub split_percent: f32,
    /// The duration of the trailers for this film in minutes
    pub trailer_duration: u32,
    /// The duration of the cleaning up after this film in minutes
    pub clean_up_duration: u32,
    /// The order of this film within the package
    pub order: u32,
}
impl PackageFilm {
    /// Get the full raw [Film] associated with this [PackageFilm]
    pub async fn film(&self, client: &Client) -> ApiResult<Film> {
        client.get_film(&self.film_id).await
    }
}

/// A package of [PackageFilm]s in the Veezi system ("double feature")
#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct FilmPackage {
    /// The unique ID of the film package
    pub id: u32,
    /// The title of the film package
    pub title: String,
    /// The current status of the film package
    pub status: FilmStatus,
    /// The list of films within this package
    pub films: Vec<PackageFilm>,
}
