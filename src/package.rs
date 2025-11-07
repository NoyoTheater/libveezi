//! Types representing screenable film packages (e.g. double features) with the
//! Veezi API.
//!
//! The primary type is [`FilmPackage`], which represents a film package and its
//! metadata.

use std::fmt::{self, Debug, Display, Formatter};

use serde::Deserialize;

use crate::{
    client::Client,
    error::ApiResult,
    film::{Film, FilmId, FilmStatus},
};

/// A particular film within a [`FilmPackage`]
#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PackageFilm {
    /// The unique ID of the film
    pub film_id: FilmId,
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
    /// Get the full raw [Film] associated with this [`PackageFilm`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn film(&self, client: &Client) -> ApiResult<Film> {
        client.get_film(&self.film_id).await
    }
}

/// The unique ID of a [`FilmPackage`]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[serde(transparent)]
pub struct FilmPackageId(u32);
impl FilmPackageId {
    /// Get the numeric ID of this [`FilmPackageId`]
    #[must_use]
    pub const fn into_u32(self) -> u32 {
        self.0
    }

    /// Fetch the full [`FilmPackage`] associated with this [`FilmPackageId`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn fetch(self, client: &Client) -> ApiResult<FilmPackage> {
        client.get_film_package(self).await
    }
}
impl Display for FilmPackageId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A package of [`PackageFilm`]s in the Veezi system ("double feature")
#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FilmPackage {
    /// The unique ID of the film package
    pub id: FilmPackageId,
    /// The title of the film package
    pub title: String,
    /// The current status of the film package
    pub status: FilmStatus,
    /// The list of films within this package
    pub films: Vec<PackageFilm>,
}
