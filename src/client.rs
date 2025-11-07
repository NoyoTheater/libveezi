//! The [`Client`] for interfacing with the Veezi API

use std::fmt::Debug;

use log::debug;
use reqwest::Url;
use serde::de::DeserializeOwned;

use crate::{
    AttributeId, FilmId, FilmPackageId, ScreenId, SessionId,
    attr::Attribute,
    error::ApiResult,
    film::Film,
    package::FilmPackage,
    screen::Screen,
    session::{Session, SessionList},
    site::Site,
};

/// The main client for interacting with the Veezi API
pub struct Client {
    /// The underlying HTTP client
    http: reqwest::Client,
    /// The base URL for the Veezi API
    base: Url,
    /// The access token for authenticating with the Veezi API
    token: String,
}
impl Client {
    /// Create a new Veezi API client from a given base URL, access token, and
    /// [`reqwest::Client`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the URL provided is invalid.
    pub fn new_with_http(
        base_url: &str,
        token: String,
        http_client: reqwest::Client,
    ) -> Result<Self, url::ParseError> {
        debug!("Spawning new libveezi Client for API base: {base_url}");
        let base = Url::parse(base_url)?;
        Ok(Self {
            http: http_client,
            base,
            token,
        })
    }

    /// Create a new Veezi API client from the given base URL and access token
    ///
    /// # Errors
    ///
    /// This function will return an error if the URL provided is invalid.
    pub fn new(base_url: &str, token: String) -> Result<Self, url::ParseError> {
        Self::new_with_http(base_url, token, reqwest::Client::new())
    }

    /// Internal helper to make a GET request to the Veezi API and parse the
    /// JSON response.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
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

        debug!(target: "libveezi-http", "OK: {resp:?}");

        Ok(resp)
    }

    /// Get a list of all future [Session]s.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_sessions(&self) -> ApiResult<SessionList> {
        Ok(self.get_json::<Vec<Session>>("v1/session").await?.into())
    }

    /// Get a list of all future [Session]s that should be available for online
    /// sales.
    ///
    /// This asserts the following for each [Session]:
    /// - [`Session::sales_cut_off_time`] is in the future
    /// - [`Session::status`] is `SessionStatus::Open`
    /// - [`Session::show_type`] is `ShowType::Public`
    /// - [`Session::sales_via`] allows [`SalesVia::www`] sales
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_web_sessions(&self) -> ApiResult<SessionList> {
        Ok(self.get_json::<Vec<Session>>("v1/websession").await?.into())
    }

    /// Get a specific [Session] by its ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_session(&self, id: SessionId) -> ApiResult<Session> {
        self.get_json(&format!("v1/session/{id}")).await
    }

    /// Get a list of all [Film]s in the Veezi system.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_films(&self) -> ApiResult<Vec<Film>> {
        self.get_json("v4/film").await
    }

    /// Get a specific [`Film`] by its ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_film(&self, id: &FilmId) -> ApiResult<Film> {
        self.get_json(&format!("v4/film/{id}")).await
    }

    /// Get a list of all [`FilmPackage`]s in the Veezi system.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_film_packages(&self) -> ApiResult<Vec<FilmPackage>> {
        self.get_json("v1/filmpackage").await
    }

    /// Get a specific [`FilmPackage`] by its ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_film_package(&self, id: FilmPackageId) -> ApiResult<FilmPackage> {
        self.get_json(&format!("v1/filmpackage/{id}")).await
    }

    /// Get a list of all [`Screen`]s in the current site.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_screens(&self) -> ApiResult<Vec<Screen>> {
        self.get_json("v1/screen").await
    }

    /// Get a specific [`Screen`] by its ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_screen(&self, id: ScreenId) -> ApiResult<Screen> {
        self.get_json(&format!("v1/screen/{id}")).await
    }

    /// Get the [`Site`] information for the current Veezi site.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_site(&self) -> ApiResult<Site> {
        self.get_json("v1/site").await
    }

    /// Get a list of all [`Attribute`]s set in the site.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_attributes(&self) -> ApiResult<Vec<Attribute>> {
        self.get_json("v1/attribute").await
    }

    /// Get a specific [`Attribute`] by its ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_attribute(&self, id: &AttributeId) -> ApiResult<Attribute> {
        self.get_json(&format!("v1/attribute/{id}")).await
    }
}
