//! Types representing individual sessions with the Veezi API.
//!
//! The primary type is [`Session`], which represents a single screening session
//! of a film at a specific time.

use std::{
    fmt::{self, Debug, Display, Formatter},
    vec::IntoIter,
};

use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

use crate::{
    attr::{Attribute, AttributeId},
    client::Client,
    error::ApiResult,
    film::{Film, FilmFormat, FilmId},
    package::{FilmPackage, FilmPackageId},
    screen::{Screen, ScreenId},
};

/// The seating type for a particular [Session]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum Seating {
    /// Allocated (reserved) seating
    Allocated,
    /// Reserved seating with some open (general admission) seats
    Select,
    /// Unallocated (general admission) seating
    Open,
}

/// The show type for a particular [Session]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum ShowType {
    /// Private show not available to the general public
    Private,
    /// Public show
    Public,
}

/// The status of a particular [Session]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum SessionStatus {
    /// Open, tickets can be sold
    Open,
    /// Closed, tickets cannot be sold
    Closed,
    /// Planned, session is planned but not yet open for sales
    Planned,
}

/// The sales channels via which tickets for a particular [Session] can be sold
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)] // this is not a state machine like clippy assumes
pub struct SalesVia {
    /// Whether tickets can be sold via KIOSK
    pub kiosk: bool,
    /// Whether tickets can be sold via POS
    pub pos: bool,
    /// Whether tickets can be sold via WWW (online)
    pub www: bool,
    /// Whether tickets can be sold via MX
    pub mx: bool,
    /// Whether tickets can be sold via RSP
    pub rsp: bool,
}
impl<'de> Deserialize<'de> for SalesVia {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec: Vec<String> = Deserialize::deserialize(deserializer)?;
        let mut sales_via = Self {
            kiosk: false,
            pos: false,
            www: false,
            mx: false,
            rsp: false,
        };
        for entry in vec {
            match entry.as_str() {
                "KIOSK" => sales_via.kiosk = true,
                "POS" => sales_via.pos = true,
                "WWW" => sales_via.www = true,
                "MX" => sales_via.mx = true,
                "RSP" => sales_via.rsp = true,
                _ => {}
            }
        }
        Ok(sales_via)
    }
}

/// A list of [Session]s with some useful helper methods
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SessionList(Vec<Session>);
impl SessionList {
    /// Obtain the [`Vec<Session>`] contained within this [`SessionList`]
    #[must_use]
    pub fn into_vec(self) -> Vec<Session> {
        self.0
    }

    /// Obtain a reference to the internal [`Vec<Session>`] contained within
    /// this [`SessionList`]
    #[must_use]
    pub const fn as_vec(&self) -> &Vec<Session> {
        &self.0
    }

    /// Filter the sessions by a given screen ID, returning a new
    /// [`SessionList`]
    #[must_use]
    pub fn filter_by_screen(self, screen_id: ScreenId) -> Self {
        let filtered: Vec<Session> = self
            .0
            .into_iter()
            .filter(|session| session.screen_id == screen_id)
            .collect();
        Self(filtered)
    }

    /// Filter the sessions by a given film ID, returning a new [`SessionList`]
    #[must_use]
    pub fn filter_by_film(self, film_id: &FilmId) -> Self {
        let filtered: Vec<Session> = self
            .0
            .into_iter()
            .filter(|session| session.film_id == *film_id)
            .collect();
        Self(filtered)
    }

    /// Filter the sessions to only those containing a given attribute ID,
    /// returning a new [`SessionList`]
    #[must_use]
    pub fn filter_containing_attribute(self, attribute_id: &AttributeId) -> Self {
        let filtered: Vec<Session> = self
            .0
            .into_iter()
            .filter(|session| session.attributes.contains(attribute_id))
            .collect();
        Self(filtered)
    }

    /// Filter sessions whose `pre_show_start_time` is within the given time
    /// range, returning a new [`SessionList`]
    #[must_use]
    pub fn filter_by_time_range(self, start: NaiveDateTime, end: NaiveDateTime) -> Self {
        let filtered: Vec<Session> = self
            .0
            .into_iter()
            .filter(|session| {
                let date = session.pre_show_start_time;
                date >= start && date <= end
            })
            .collect();
        Self(filtered)
    }

    /// Filter sessions whose `pre_show_start_time` is within the given date
    /// range, returning a new [`SessionList`]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn filter_by_date_range(self, start: NaiveDate, end: NaiveDate) -> Self {
        self.filter_by_time_range(
            start.and_hms_opt(0, 0, 0).expect("midnight should exist"),
            end.and_hms_opt(0, 0, 0).expect("midnight should exist"),
        )
    }

    /// Group a list of sessions by date, returning a vector of tuples where the
    /// first element is the date and the second element is a vector of
    /// references to the sessions on that date
    #[must_use]
    pub fn group_by_date(&self) -> Vec<(NaiveDate, Vec<&Session>)> {
        let mut grouped: Vec<(NaiveDate, Vec<&Session>)> = Vec::new();
        for session in &self.0 {
            let date = session.pre_show_start_time.date();
            if let Some((_, sessions)) = grouped.iter_mut().find(|(d, _)| *d == date) {
                sessions.push(session);
            } else {
                grouped.push((date, vec![session]));
            }
        }
        grouped.sort_by(|(a, _), (b, _)| a.cmp(b));
        grouped
    }

    /// Get all of the films represented in this [`SessionList`]
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the API requests fail.
    pub async fn films(&self, client: &Client) -> ApiResult<Vec<Film>> {
        let mut films = Vec::new();
        let mut seen_ids = Vec::new();
        for session in &self.0 {
            if !seen_ids.contains(&session.film_id) {
                let film = client.get_film(&session.film_id).await?;
                films.push(film);
                seen_ids.push(session.film_id.clone());
            }
        }
        Ok(films)
    }

    /// Get all of the screens represented in this [`SessionList`]
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the API requests fail.
    pub async fn screens(&self, client: &Client) -> ApiResult<Vec<Screen>> {
        let mut screens = Vec::new();
        let mut seen_ids = Vec::new();
        for session in &self.0 {
            if !seen_ids.contains(&session.screen_id) {
                let screen = client.get_screen(session.screen_id).await?;
                screens.push(screen);
                seen_ids.push(session.screen_id);
            }
        }
        Ok(screens)
    }

    /// Get an iterator over the sessions in this [`SessionList`]
    pub fn iter(&self) -> impl Iterator<Item = &Session> {
        self.0.iter()
    }
}
impl From<Vec<Session>> for SessionList {
    fn from(sessions: Vec<Session>) -> Self {
        Self(sessions)
    }
}
impl From<SessionList> for Vec<Session> {
    fn from(val: SessionList) -> Self {
        val.0
    }
}
impl IntoIterator for SessionList {
    type Item = Session;
    type IntoIter = IntoIter<Session>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// The unique ID of a [`Session`]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[serde(transparent)]
pub struct SessionId(u32);
impl SessionId {
    /// Get the numeric ID of this [`SessionId`]
    #[must_use]
    pub const fn into_u32(self) -> u32 {
        self.0
    }

    /// Fetch the full [`Session`] associated with this [`SessionId`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn fetch(self, client: &Client) -> ApiResult<Session> {
        client.get_session(self).await
    }
}
impl Display for SessionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A particular screening session of a [Film]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Session {
    /// The unique ID of the session
    pub id: SessionId,
    /// The ID of the film being shown in this session
    pub film_id: FilmId,
    /// The ID of the film package (if any) associated with this session
    pub film_package_id: Option<FilmPackageId>,
    /// The title of the film being shown in this session
    pub title: String,
    /// The screen ID where this session is being shown
    pub screen_id: ScreenId,
    /// The seating type for this session
    pub seating: Seating,
    /// Whether complimentary tickets are allowed for this session
    pub are_complimentaries_allowed: bool,
    /// The show type for this session
    pub show_type: ShowType,
    /// The sales channels via which tickets for this session can be sold
    pub sales_via: SalesVia,
    /// The status of this session
    pub status: SessionStatus,
    /// The time this session starts
    pub pre_show_start_time: NaiveDateTime,
    /// The time this session ends sales
    pub sales_cut_off_time: NaiveDateTime,
    /// The time this session's feature starts
    pub feature_start_time: NaiveDateTime,
    /// The time this session's feature ends
    pub feature_end_time: NaiveDateTime,
    /// The time this session's cleanup ends
    pub cleanup_end_time: NaiveDateTime,
    /// Whether tickets for this session are sold out
    pub tickets_sold_out: bool,
    /// Whether there are few tickets left for this session
    pub few_tickets_left: bool,
    /// The number of seats available for this session
    pub seats_available: u32,
    /// The number of seats held for this session
    pub seats_held: u32,
    /// The number of house seats for this session
    pub seats_house: u32,
    /// The number of seats sold for this session
    pub seats_sold: u32,
    /// The format of the film being shown in this session
    pub film_format: FilmFormat,
    /// The price card name associated with this session
    pub price_card_name: String,
    /// The list of attribute IDs associated with this session
    pub attributes: Vec<AttributeId>,
    /// The audio language of the film being shown in this session
    pub audio_language: Option<String>,
}
impl Session {
    /// Get the [`Film`] associated with this [`Session`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn film(&self, client: &Client) -> ApiResult<Film> {
        client.get_film(&self.film_id).await
    }

    /// Get the [`FilmPackage`] associated with this [`Session`], if any
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn film_package(&self, client: &Client) -> ApiResult<Option<FilmPackage>> {
        match &self.film_package_id {
            Some(id) => {
                let package = client.get_film_package(*id).await?;
                Ok(Some(package))
            }
            None => Ok(None),
        }
    }

    /// Get the [`Screen`] associated with this [`Session`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn screen(&self, client: &Client) -> ApiResult<Screen> {
        client.get_screen(self.screen_id).await
    }

    /// Get the list of [Attribute]s associated with this [`Session`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn attributes(&self, client: &Client) -> ApiResult<Vec<Attribute>> {
        let mut attrs = Vec::new();
        for attr_id in &self.attributes {
            let attr = client.get_attribute(attr_id).await?;
            attrs.push(attr);
        }
        Ok(attrs)
    }

    /// Returns whether tickets can still be sold for this session
    #[must_use]
    pub fn is_open_for_sales(&self) -> bool {
        let now = chrono::Utc::now().naive_utc();
        self.status == SessionStatus::Open
            && now < self.sales_cut_off_time
            && self.seats_available > 0
    }
}
