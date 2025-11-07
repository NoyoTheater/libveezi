//! Types representing individual sessions with the Veezi API.
//!
//! The primary type is [`Session`], which represents a single screening session
//! of a film at a specific time.

use crate::attr::Attribute;
use crate::client::Client;
use crate::error::ApiResult;
use crate::film::{Film, FilmFormat};
use crate::package::FilmPackage;
use crate::screen::Screen;
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use std::fmt::Debug;

/// The seating type for a particular [Session]
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum Seating {
    /// Allocated (reserved) seating
    Allocated,
    Select,
    /// Unallocated (general admission) seating
    Open,
}

/// The show type for a particular [Session]
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ShowType {
    /// Private show not available to the general public
    Private,
    /// Public show
    Public,
}

/// The status of a particular [Session]
#[derive(Deserialize, Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
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
        let mut sales_via = SalesVia {
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
#[derive(Debug, PartialEq)]
pub struct SessionList(Vec<Session>);
impl SessionList {
    /// Obtain the [`Vec<Session>`] contained within this [SessionList]
    pub fn into_vec(self) -> Vec<Session> {
        self.0
    }

    /// Obtain a reference to the internal [`Vec<Session>`] contained within this [SessionList]
    pub fn as_vec(&self) -> &Vec<Session> {
        &self.0
    }

    /// Filter the sessions by a given screen ID, returning a new [SessionList]
    pub fn filter_by_screen(self, screen_id: u32) -> SessionList {
        let filtered: Vec<Session> = self
            .0
            .into_iter()
            .filter(|session| session.screen_id == screen_id)
            .collect();
        SessionList(filtered)
    }

    /// Filter the sessions by a given film ID, returning a new [SessionList]
    pub fn filter_by_film(self, film_id: &str) -> SessionList {
        let filtered: Vec<Session> = self
            .0
            .into_iter()
            .filter(|session| session.film_id == film_id)
            .collect();
        SessionList(filtered)
    }

    /// Filter the sessions to only those containing a given attribute ID, returning a new [SessionList]
    pub fn filter_containing_attribute(self, attribute_id: &str) -> SessionList {
        let filtered: Vec<Session> = self
            .0
            .into_iter()
            .filter(|session| session.attributes.contains(&attribute_id.to_string()))
            .collect();
        SessionList(filtered)
    }

    /// Filter sessions whose `pre_show_start_time` is within the given date range, returning a new [SessionList]
    pub fn filter_by_date_range(self, start: NaiveDate, end: NaiveDate) -> SessionList {
        let filtered: Vec<Session> = self
            .0
            .into_iter()
            .filter(|session| {
                let date = session.pre_show_start_time.date();
                date >= start && date <= end
            })
            .collect();
        SessionList(filtered)
    }
}
impl From<Vec<Session>> for SessionList {
    fn from(sessions: Vec<Session>) -> Self {
        SessionList(sessions)
    }
}
impl From<SessionList> for Vec<Session> {
    fn from(val: SessionList) -> Self {
        val.0
    }
}
impl IntoIterator for SessionList {
    type Item = Session;
    type IntoIter = std::vec::IntoIter<Session>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A particular screening session of a [Film]
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Session {
    /// The unique ID of the session
    pub id: u32,
    /// The ID of the film being shown in this session
    pub film_id: String,
    /// The ID of the film package (if any) associated with this session
    pub film_package_id: Option<u32>,
    /// The title of the film being shown in this session
    pub title: String,
    /// The screen ID where this session is being shown
    pub screen_id: u32,
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
    pub attributes: Vec<String>,
    /// The audio language of the film being shown in this session
    pub audio_language: Option<String>,
}
impl Session {
    /// Get the [Film] associated with this [Session]
    pub async fn film(&self, client: &Client) -> ApiResult<Film> {
        client.get_film(&self.film_id).await
    }

    /// Get the [FilmPackage] associated with this [Session], if any
    pub async fn film_package(&self, client: &Client) -> ApiResult<Option<FilmPackage>> {
        match &self.film_package_id {
            Some(id) => {
                let package = client.get_film_package(*id).await?;
                Ok(Some(package))
            }
            None => Ok(None),
        }
    }

    /// Get the [Screen] associated with this [Session]
    pub async fn screen(&self, client: &Client) -> ApiResult<Screen> {
        client.get_screen(self.screen_id).await
    }

    /// Get the list of [Attribute]s associated with this [Session]
    pub async fn attributes(&self, client: &Client) -> ApiResult<Vec<Attribute>> {
        let mut attrs = Vec::new();
        for attr_id in &self.attributes {
            let attr = client.get_attribute(attr_id).await?;
            attrs.push(attr);
        }
        Ok(attrs)
    }

    /// Returns whether tickets can still be sold for this session
    pub fn is_open_for_sales(&self) -> bool {
        let now = chrono::Utc::now().naive_utc();
        self.status == SessionStatus::Open
            && now < self.sales_cut_off_time
            && self.seats_available > 0
    }
}
