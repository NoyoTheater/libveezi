use chrono::NaiveDateTime;
use log::debug;
use reqwest::Url;
use serde::{Deserialize, de::DeserializeOwned};
use std::{error::Error, fmt::Debug};

pub type ApiResult<T> = Result<T, Box<dyn Error>>;
pub struct Client {
    http: reqwest::Client,
    base: Url,
    token: String,
}
impl Client {
    pub fn new(base_url: &str, token: String) -> Result<Self, Box<dyn Error>> {
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

    pub async fn list_sessions(&self) -> ApiResult<Vec<Session>> {
        self.get_json("v1/session").await
    }
    pub async fn list_web_sessions(&self) -> ApiResult<Vec<Session>> {
        self.get_json("v1/websession").await
    }
    pub async fn get_session(&self, id: u32) -> ApiResult<Session> {
        self.get_json(&format!("v1/session/{}", id)).await
    }

    pub async fn list_films(&self) -> ApiResult<Vec<Film>> {
        self.get_json("v4/film").await
    }
    pub async fn get_film(&self, id: &str) -> ApiResult<Film> {
        self.get_json(&format!("v4/film/{}", id)).await
    }

    pub async fn list_film_packages(&self) -> ApiResult<Vec<FilmPackage>> {
        self.get_json("v1/filmpackage").await
    }
    pub async fn get_film_package(&self, id: u32) -> ApiResult<FilmPackage> {
        self.get_json(&format!("v1/filmpackage/{}", id)).await
    }

    pub async fn list_screens(&self) -> ApiResult<Vec<Screen>> {
        self.get_json("v1/screen").await
    }
    pub async fn get_screen(&self, id: u32) -> ApiResult<Screen> {
        self.get_json(&format!("v1/screen/{}", id)).await
    }

    pub async fn get_site(&self) -> ApiResult<Site> {
        self.get_json("v1/site").await
    }

    pub async fn list_attributes(&self) -> ApiResult<Vec<Attribute>> {
        self.get_json("v1/attribute").await
    }
    pub async fn get_attribute(&self, id: &str) -> ApiResult<Attribute> {
        self.get_json(&format!("v1/attribute/{}", id)).await
    }
}

// Helpers used to deserialize `[{Id:1},{Id:2}]` into `vec![1, 2]`
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum Seating {
    Allocated,
    Select,
    Open,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum ShowType {
    Private,
    Public,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum SessionStatus {
    Open,
    Closed,
    Planned,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum FilmStatus {
    Active,
    Inactive,
    Deleted,
}

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

#[derive(Debug)]
pub struct SalesVia {
    pub kiosk: bool,
    pub pos: bool,
    pub www: bool,
    pub mx: bool,
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Person {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Session {
    pub id: u32,
    pub film_id: String,
    pub film_package_id: Option<String>,
    pub title: String,
    pub screen_id: u32,
    pub seating: Seating,
    pub are_complimentaries_allowed: bool,
    pub show_type: ShowType,
    pub sales_via: SalesVia,
    pub status: SessionStatus,
    pub pre_show_start_time: NaiveDateTime,
    pub sales_cut_off_time: NaiveDateTime,
    pub feature_start_time: NaiveDateTime,
    pub feature_end_time: NaiveDateTime,
    pub cleanup_end_time: NaiveDateTime,
    pub tickets_sold_out: bool,
    pub few_tickets_left: bool,
    pub seats_available: u32,
    pub seats_held: u32,
    pub seats_house: u32,
    pub seats_sold: u32,
    pub film_format: FilmFormat,
    pub price_card_name: String,
    pub attributes: Vec<String>,
    pub audio_language: Option<String>,
}
impl Session {
    pub async fn film(&self, client: &Client) -> ApiResult<Film> {
        client.get_film(&self.film_id).await
    }

    pub async fn film_package(&self, client: &Client) -> ApiResult<Option<FilmPackage>> {
        match &self.film_package_id {
            Some(id) => {
                let package = client.get_film_package(id.parse::<u32>()?).await?;
                Ok(Some(package))
            }
            None => Ok(None),
        }
    }

    pub async fn screen(&self, client: &Client) -> ApiResult<Screen> {
        client.get_screen(self.screen_id).await
    }

    pub async fn attributes(&self, client: &Client) -> ApiResult<Vec<Attribute>> {
        let mut attrs = Vec::new();
        for attr_id in &self.attributes {
            let attr = client.get_attribute(attr_id).await?;
            attrs.push(attr);
        }
        Ok(attrs)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Film {
    pub id: String,
    pub title: String,
    pub short_name: String,
    pub synopsis: Option<String>,
    pub genre: String,
    pub signage_text: String,
    pub distributor: String,
    pub opening_date: NaiveDateTime,
    pub rating: Option<String>,
    pub status: FilmStatus,
    pub content: Option<String>,
    pub duration: u32,
    pub display_sequence: u32,
    pub national_code: Option<String>,
    pub format: FilmFormat,
    pub is_restricted: bool,
    pub people: Vec<Person>,
    pub audio_language: Option<String>,
    pub government_film_title: Option<String>,
    pub film_poster_url: Option<String>,
    pub film_poster_thumbnail_url: String,
    pub backdrop_image_url: Option<String>,
    pub film_trailer_url: Option<String>,
}
impl Film {
    pub async fn sessions(&self, client: &Client) -> ApiResult<Vec<Session>> {
        let all_sessions = client.list_sessions().await?;
        let film_sessions: Vec<Session> = all_sessions
            .into_iter()
            .filter(|session| session.film_id == self.id)
            .collect();
        Ok(film_sessions)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PackageFilm {
    pub film_id: String,
    pub title: String,
    pub split_percent: f32,
    pub trailer_duration: u32,
    pub clean_up_duration: u32,
    pub order: u32,
}
impl PackageFilm {
    pub async fn film(&self, client: &Client) -> ApiResult<Film> {
        client.get_film(&self.film_id).await
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct FilmPackage {
    pub id: u32,
    pub title: String,
    pub status: FilmStatus,
    pub films: Vec<PackageFilm>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Screen {
    pub id: u32,
    pub name: String,
    pub screen_number: String,
    pub has_custom_layout: bool,
    pub total_seats: u32,
    pub house_seats: u32,
}
impl Screen {
    pub async fn sessions(&self, client: &Client) -> ApiResult<Vec<Session>> {
        let all_sessions = client.list_sessions().await?;
        let screen_sessions: Vec<Session> = all_sessions
            .into_iter()
            .filter(|session| session.screen_id == self.id)
            .collect();
        Ok(screen_sessions)
    }
}

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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Attribute {
    pub id: String,
    pub description: String,
    pub short_name: String,
    pub font_color: String,
    pub background_color: String,
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
