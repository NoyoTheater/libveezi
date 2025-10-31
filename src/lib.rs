#![doc = include_str!("../README.md")]
use chrono::NaiveDateTime;
use log::debug;
use reqwest::Url;
use serde::{Deserialize, de::DeserializeOwned};
use std::fmt::Debug;

/// The list of errors that can occur when using the libveezi library
pub enum LibVeeziError {
    /// An error occurred while making an HTTP request
    Http(reqwest::Error),
    /// An error occurred while parsing a URL
    UrlParse(url::ParseError),
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

/// The main client for interacting with the Veezi API
pub struct Client {
    http: reqwest::Client,
    base: Url,
    token: String,
}
impl Client {
    /// Create a new Veezi API client from the given base URL and access token
    pub fn new(base_url: &str, token: String) -> Result<Self, url::ParseError> {
        debug!("Spawning new libveezi Client for API base: {base_url}");
        let base = Url::parse(base_url)?;
        let http = reqwest::Client::new();
        Ok(Client { http, base, token })
    }

    async fn get_json<T>(&self, endpoint: &str) -> ApiResult<T>
    where
        T: DeserializeOwned + Debug,
    {
        let url = self.base.join(endpoint)?;

        debug!(target: "libveezi-http", "GET {url}");

        let resp = self
            .http
            .get(url)
            .header("VeeziAccessToken", &self.token)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await?;

        debug!(target: "libveezi-http", "OK: {:?}", resp);

        Ok(resp)
    }

    /// Get a list of all future [Session]s.
    pub async fn list_sessions(&self) -> ApiResult<Vec<Session>> {
        self.get_json("v1/session").await
    }

    /// Get a list of all future [Session]s that should be available for online sales.
    ///
    /// This asserts the following for each [Session]:
    /// - [`Session::sales_cut_off_time`] is in the future
    /// - [`Session::status`] is `SessionStatus::Open`
    /// - [`Session::show_type`] is `ShowType::Public`
    /// - [`Session::sales_via`] allows [`SalesVia::www`] sales
    pub async fn list_web_sessions(&self) -> ApiResult<Vec<Session>> {
        self.get_json("v1/websession").await
    }

    /// Get a specific [Session] by its ID.
    pub async fn get_session(&self, id: u32) -> ApiResult<Session> {
        self.get_json(&format!("v1/session/{}", id)).await
    }

    /// Get a list of all [Film]s in the Veezi system.
    pub async fn list_films(&self) -> ApiResult<Vec<Film>> {
        self.get_json("v4/film").await
    }

    /// Get a specific [Film] by its ID.
    pub async fn get_film(&self, id: &str) -> ApiResult<Film> {
        self.get_json(&format!("v4/film/{}", id)).await
    }

    /// Get a list of all [FilmPackage]s in the Veezi system.
    pub async fn list_film_packages(&self) -> ApiResult<Vec<FilmPackage>> {
        self.get_json("v1/filmpackage").await
    }

    /// Get a specific [FilmPackage] by its ID.
    pub async fn get_film_package(&self, id: u32) -> ApiResult<FilmPackage> {
        self.get_json(&format!("v1/filmpackage/{}", id)).await
    }

    /// Get a list of all [Screen]s in the current site.
    pub async fn list_screens(&self) -> ApiResult<Vec<Screen>> {
        self.get_json("v1/screen").await
    }

    /// Get a specific [Screen] by its ID.
    pub async fn get_screen(&self, id: u32) -> ApiResult<Screen> {
        self.get_json(&format!("v1/screen/{}", id)).await
    }

    /// Get the [Site] information for the current Veezi site.
    pub async fn get_site(&self) -> ApiResult<Site> {
        self.get_json("v1/site").await
    }

    /// Get a list of all [Attribute]s set in the site.
    pub async fn list_attributes(&self) -> ApiResult<Vec<Attribute>> {
        self.get_json("v1/attribute").await
    }

    /// Get a specific [Attribute] by its ID.
    pub async fn get_attribute(&self, id: &str) -> ApiResult<Attribute> {
        self.get_json(&format!("v1/attribute/{}", id)).await
    }
}

/// Helper function used to deserialize `[{Id:1},{Id:2}]` into `vec![1, 2]`
fn deserialize_id_array<'de, D>(deserializer: D) -> Result<Vec<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct IdHelper {
        id: u32,
    }

    let helper_vec: Vec<IdHelper> = Deserialize::deserialize(deserializer)?;
    Ok(helper_vec.into_iter().map(|attr| attr.id).collect())
}

/// The seating type for a particular [Session]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum Seating {
    /// Allocated (reserved) seating
    Allocated,
    Select,
    /// Unallocated (general admission) seating
    Open,
}

/// The show type for a particular [Session]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum ShowType {
    /// Private show not available to the general public
    Private,
    /// Public show
    Public,
}

/// The status of a particular [Session]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum SessionStatus {
    /// Open, tickets can be sold
    Open,
    /// Closed, tickets cannot be sold
    Closed,
    /// Planned, session is planned but not yet open for sales
    Planned,
}

/// The status of a particular [Film]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum FilmStatus {
    /// Film is active and can be scheduled
    Active,
    /// Film is inactive and cannot be scheduled
    Inactive,
    /// Film has been deleted
    Deleted,
}

/// The format of a particular [Film]
#[derive(Deserialize, Debug)]
pub enum FilmFormat {
    #[serde(rename = "2D Film")]
    Film2D,
    #[serde(rename = "2D Digital")]
    Digital2D,
    #[serde(rename = "3D Digital")]
    Digital3D,
    #[serde(rename = "3D HFR")]
    Digital3DHFR,
    #[serde(rename = "Not a Film")]
    NotAFilm,
}

/// The sales channels via which tickets for a particular [Session] can be sold
#[derive(Debug)]
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

/// A particular person associated with a [Film]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Person {
    /// The unique ID of the person
    pub id: String,
    /// The first name of the person
    pub first_name: String,
    /// The last name of the person
    pub last_name: String,
    /// The role of the person in the film (e.g., Actor, Director)
    pub role: String,
}

/// A particular screening session of a [Film]
#[derive(Deserialize, Debug)]
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
}

/// A particular film in the Veezi system
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Film {
    /// The unique ID of the film
    pub id: String,
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
    /// Get a list of all future [Session]s for this [Film]
    pub async fn sessions(&self, client: &Client) -> ApiResult<Vec<Session>> {
        let all_sessions = client.list_sessions().await?;
        let film_sessions: Vec<Session> = all_sessions
            .into_iter()
            .filter(|session| session.film_id == self.id)
            .collect();
        Ok(film_sessions)
    }
}

/// A particular film within a [FilmPackage]
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
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

/// A particular screen (auditorium) in the Veezi system
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Screen {
    /// The unique ID of the screen
    pub id: u32,
    /// The name of the screen
    pub name: String,
    /// The screen number as a string
    pub screen_number: String,
    /// Whether the screen has a custom layout
    pub has_custom_layout: bool,
    /// The total number of seats in the screen
    pub total_seats: u32,
    /// The number of house seats in the screen
    pub house_seats: u32,
}
impl Screen {
    /// Get a list of all future [Session]s for this [Screen]
    pub async fn sessions(&self, client: &Client) -> ApiResult<Vec<Session>> {
        let all_sessions = client.list_sessions().await?;
        let screen_sessions: Vec<Session> = all_sessions
            .into_iter()
            .filter(|session| session.screen_id == self.id)
            .collect();
        Ok(screen_sessions)
    }
}

/// Information about the current Veezi site
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Site {
    pub name: String,
    pub short_name: String,
    pub legal_name: String,
    pub national_code: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub address_3: Option<String>,
    pub post_code: Option<String>,
    pub phone_1: Option<String>,
    pub phone_2: Option<String>,
    pub fax: Option<String>,
    pub sales_tax_registration: Option<String>,
    pub ticket_message_1: Option<String>,
    pub ticket_message_2: Option<String>,
    pub receipt_message_1: Option<String>,
    pub receipt_message_2: Option<String>,
    pub receipt_message_3: Option<String>,
    pub receipt_message_4: Option<String>,
    pub receipt_message_5: Option<String>,
    pub receipt_message_6: Option<String>,
    pub time_zone_identifier: String,
    pub country: String,
    #[serde(deserialize_with = "deserialize_id_array")]
    pub screens: Vec<u32>,
}

/// An attribute that can be associated with [Session]s
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Attribute {
    /// The unique ID of the attribute
    pub id: String,
    /// The name of the attribute
    pub description: String,
    /// The short name of the attribute
    pub short_name: String,
    /// The font color associated with the attribute (hex code)
    pub font_color: String,
    /// The background color associated with the attribute (hex code)
    pub background_color: String,
    /// Whether to show this attribute on sessions that have no complimentary tickets
    pub show_on_sessions_with_no_comps: bool,
}
impl Attribute {
    pub async fn sessions(&self, client: &Client) -> ApiResult<Vec<Session>> {
        let all_sessions = client.list_sessions().await?;
        let attribute_sessions: Vec<Session> = all_sessions
            .into_iter()
            .filter(|session| session.attributes.contains(&self.id))
            .collect();
        Ok(attribute_sessions)
    }
}
