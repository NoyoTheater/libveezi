//! Types representing screenable films with the Veezi API.
//!
//! The primary type is [`Film`], which represents a film and its metadata.

use std::fmt::{self, Debug, Display, Formatter};

use chrono::NaiveDateTime;
use serde::Deserialize;

#[allow(unused_imports)] // for docs
use crate::session::{SalesVia, Session, SessionStatus, ShowType};
use crate::{client::Client, error::ApiResult, session::SessionList};

/// The status of a particular [`Film`]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum FilmStatus {
    /// Film is active and can be scheduled
    Active,
    /// Film is inactive and cannot be scheduled
    Inactive,
    /// Film has been deleted
    Deleted,
}

/// The format of a particular [`Film`]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum FilmFormat {
    /// A 2D film
    #[serde(rename = "2D Film")]
    Film2D,
    /// A 2D digital film
    #[serde(rename = "2D Digital")]
    Digital2D,
    /// A 3D digital film
    #[serde(rename = "3D Digital")]
    Digital3D,
    /// A 3D HFR (High Frame Rate) digital film
    #[serde(rename = "3D HFR")]
    Digital3DHFR,
    /// Not a film (e.g., live event)
    #[serde(rename = "Not a Film")]
    NotAFilm,
}

/// The unique ID of a [`Person`]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(transparent)]
pub struct PersonId(String);
impl PersonId {
    /// Get the string representation of this [`PersonId`]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl Display for PersonId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A particular person associated with a [`Film`]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct Person {
    /// The unique ID of the person
    pub id: PersonId,
    /// The first name of the person
    pub first_name: String,
    /// The last name of the person
    pub last_name: String,
    /// The role of the person in the film (e.g., Actor, Director)
    pub role: String,
}

/// The unique ID of a [`Film`]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(transparent)]
pub struct FilmId(String);
impl FilmId {
    /// Get the string representation of this [`FilmId`]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Fetch the full [`Film`] associated with this [`FilmId`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn fetch(&self, client: &Client) -> ApiResult<Film> {
        client.get_film(self).await
    }
}
impl Display for FilmId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A particular film in the Veezi system
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Film {
    /// The unique ID of the film
    pub id: FilmId,
    /// The title of the film
    pub title: String,
    /// The short name of the film (len<=10)
    pub short_name: String,
    /// The synopsis of the film
    pub synopsis: Option<String>,
    /// The genre of the film
    pub genre: String,
    /// The signage display name for the film
    pub signage_text: String,
    /// The distributor of the film
    pub distributor: String,
    /// The opening date of the film
    pub opening_date: NaiveDateTime,
    /// The rating of the film (e.g., "PG-13")
    ///
    /// If `None`, the film is not rated ("NR")
    pub rating: Option<String>,
    /// The current status of the film
    pub status: FilmStatus,
    /// The rating's content description ("R for language and violence", etc)
    pub content: Option<String>,
    /// The duration of the film in minutes
    pub duration: u32,
    /// The display sequence of the film
    pub display_sequence: u32,
    /// The film's national code (if any)
    pub national_code: Option<String>,
    /// The format of the film
    pub format: FilmFormat,
    /// Whether the film is restricted to certain audiences
    pub is_restricted: bool,
    /// The list of people associated with the film
    pub people: Vec<Person>,
    /// The primary audio language of the film
    pub audio_language: Option<String>,
    /// The federal title of the film for box office reporting, if any
    pub government_film_title: Option<String>,
    /// The film's poster URL, if any
    pub film_poster_url: Option<String>,
    /// The film's poster thumbnail URL
    pub film_poster_thumbnail_url: String,
    /// The film's backdrop image URL, if any
    pub backdrop_image_url: Option<String>,
    /// The film's trailer URL, if any
    pub film_trailer_url: Option<String>,
}
impl Film {
    /// Get a list of all future [`Session`]s for this [`Film`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn sessions(&self, client: &Client) -> ApiResult<SessionList> {
        Ok(client.list_sessions().await?.filter_by_film(&self.id))
    }

    /// Get a list of all future [Session]s for this [`Film`] that should be
    /// available for online sales.
    ///
    /// This asserts the following for each [Session]:
    /// - [`Session::sales_cut_off_time`] is in the future
    /// - [`Session::status`] is [`SessionStatus::Open`]
    /// - [`Session::show_type`] is [`ShowType::Public`]
    /// - [`Session::sales_via`] allows [`SalesVia::www`] sales
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn web_sessions(&self, client: &Client) -> ApiResult<SessionList> {
        Ok(client.list_web_sessions().await?.filter_by_film(&self.id))
    }

    /// Format the duration of the film as "Xh Ym" or "Xh"
    #[must_use]
    pub fn formatted_duration(&self) -> String {
        let hours = self.duration / 60;
        let minutes = self.duration % 60;
        if minutes == 0 {
            format!("{hours}h")
        } else {
            format!("{hours}h {minutes}m")
        }
    }

    /// Check if the film is currently active
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.status, FilmStatus::Active)
    }

    /// Check if the film is a 3D film (any 3D format)
    #[must_use]
    pub const fn is_3d(&self) -> bool {
        matches!(
            self.format,
            FilmFormat::Digital3D | FilmFormat::Digital3DHFR
        )
    }

    /// Check if the film is a 2D film (any 2D format)
    #[must_use]
    pub const fn is_2d(&self) -> bool {
        matches!(self.format, FilmFormat::Film2D | FilmFormat::Digital2D)
    }

    /// Get the list of actors associated with this film
    #[must_use]
    pub fn actors(&self) -> Vec<&Person> {
        self.people.iter().filter(|p| p.role == "Actor").collect()
    }

    /// Get the list of directors associated with this film
    #[must_use]
    pub fn directors(&self) -> Vec<&Person> {
        self.people
            .iter()
            .filter(|p| p.role == "Director")
            .collect()
    }

    /// Get a formatted string of actor names, separated by commas
    #[must_use]
    pub fn actors_formatted(&self) -> String {
        self.actors()
            .iter()
            .map(|p| format!("{} {}", p.first_name, p.last_name))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Get a formatted string of director names, separated by commas
    #[must_use]
    pub fn directors_formatted(&self) -> String {
        self.directors()
            .iter()
            .map(|p| format!("{} {}", p.first_name, p.last_name))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Get a display-friendly rating string, returning "NR" if no rating is set
    #[must_use]
    pub fn rating_display(&self) -> String {
        self.rating.clone().unwrap_or_else(|| "NR".to_string())
    }
}
