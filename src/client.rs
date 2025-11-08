//! The [`Client`] for interfacing with the Veezi API

use std::{fmt::Debug, time::Duration};

use chrono::{NaiveDate, NaiveDateTime};
use log::debug;
use moka::future::{Cache, CacheBuilder};
use reqwest::Url;
use serde::de::DeserializeOwned;

use crate::{
    attr::{Attribute, AttributeId},
    error::ApiResult,
    film::{Film, FilmId, FilmStatus},
    package::{FilmPackage, FilmPackageId},
    screen::{Screen, ScreenId},
    session::{Session, SessionId, SessionList},
    site::Site,
};

/// A structure for building a libveezi [`Client`] with various options
pub struct ClientBuilder {
    /// The underlying HTTP client
    http: reqwest::Client,
    /// The base URL for the Veezi API
    base_url: String,
    /// The access token for authenticating with the Veezi API
    token: String,
    /// Enable caching for [`Session`]s with the given TTL and max capacity
    pub session_cache: Option<(Duration, u64)>,
    /// Enable caching for [`Film`]s with the given TTL and max capacity
    pub film_cache: Option<(Duration, u64)>,
    /// Enable caching for [`FilmPackage`]s with the given TTL and max capacity
    pub film_package_cache: Option<(Duration, u64)>,
    /// Enable caching for [`Screen`]s with the given TTL and max capacity
    pub screen_cache: Option<(Duration, u64)>,
    /// Enable caching for [`Attribute`]s with the given TTL and max capacity
    pub attribute_cache: Option<(Duration, u64)>,
    /// Enable caching for the current [`Site`] with the given TTL
    pub site_cache: Option<Duration>,
}
impl ClientBuilder {
    /// Create a new [`ClientBuilder`] with the given base URL, access token,
    /// and underlying HTTP client
    #[must_use]
    pub fn new_with_http(base_url: &str, token: String, http_client: reqwest::Client) -> Self {
        Self {
            http: http_client,
            base_url: base_url.to_string(),
            token,
            session_cache: None,
            film_cache: None,
            film_package_cache: None,
            screen_cache: None,
            attribute_cache: None,
            site_cache: None,
        }
    }

    /// Create a new [`ClientBuilder`] with the given base URL and access token
    #[must_use]
    pub fn new(base_url: &str, token: String) -> Self {
        Self::new_with_http(base_url, token, reqwest::Client::new())
    }

    /// Build the [`Client`] from this builder
    ///
    /// # Errors
    ///
    /// This function will return an error if the URL provided is invalid.
    pub fn build(self) -> Result<Client, url::ParseError> {
        Client::from_builder(self)
    }

    /// Enable caching for [`Session`]s with the given TTL and max capacity
    #[must_use]
    pub const fn with_session_cache(mut self, ttl: Duration, max: u64) -> Self {
        self.session_cache = Some((ttl, max));
        self
    }

    /// Enable caching for [`Film`]s with the given TTL and max capacity
    #[must_use]
    pub const fn with_film_cache(mut self, ttl: Duration, max: u64) -> Self {
        self.film_cache = Some((ttl, max));
        self
    }

    /// Enable caching for [`FilmPackage`]s with the given TTL and max capacity
    #[must_use]
    pub const fn with_film_package_cache(mut self, ttl: Duration, max: u64) -> Self {
        self.film_package_cache = Some((ttl, max));
        self
    }

    /// Enable caching for [`Screen`]s with the given TTL and max capacity
    #[must_use]
    pub const fn with_screen_cache(mut self, ttl: Duration, max: u64) -> Self {
        self.screen_cache = Some((ttl, max));
        self
    }

    /// Enable caching for [`Attribute`]s with the given TTL and max capacity
    #[must_use]
    pub const fn with_attribute_cache(mut self, ttl: Duration, max: u64) -> Self {
        self.attribute_cache = Some((ttl, max));
        self
    }

    /// Enable caching for the current [`Site`] with the given TTL
    #[must_use]
    pub const fn with_site_cache(mut self, ttl: Duration) -> Self {
        self.site_cache = Some(ttl);
        self
    }

    /// Enable caching for all supported types with default settings
    #[must_use]
    pub const fn with_default_caching(self) -> Self {
        self.with_session_cache(Duration::from_secs(30), 1000)
            .with_film_cache(Duration::from_secs(300), 500)
            .with_film_package_cache(Duration::from_secs(300), 500)
            .with_screen_cache(Duration::from_secs(3600), 100)
            .with_attribute_cache(Duration::from_secs(300), 500)
            .with_site_cache(Duration::from_secs(300))
    }
}

#[allow(clippy::doc_markdown)]
/// The main client for interacting with the Veezi API
pub struct Client {
    /// The underlying HTTP client
    http: reqwest::Client,
    /// The base URL for the Veezi API
    base: Url,
    /// The access token for authenticating with the Veezi API
    token: String,

    // Some of these caches use `()` as the key type to cache the full list responses
    // We cannot just list all items from the individual item caches because they may expire
    /// The MiniLFU cache for [`Session`]s
    session_cache: Option<Cache<SessionId, Session>>,
    /// The MiniLFU cache for the full [`SessionList`]
    session_list_cache: Option<Cache<(), SessionList>>,
    /// The MiniLFU cache for the full web [`SessionList`]
    web_session_list_cache: Option<Cache<(), SessionList>>,
    /// The MiniLFU cache for [`Film`]s
    film_cache: Option<Cache<FilmId, Film>>,
    /// The MiniLFU cache for the full list of [`Film`]s
    film_list_cache: Option<Cache<(), Vec<Film>>>,
    /// The MiniLFU cache for [`FilmPackage`]s
    film_package_cache: Option<Cache<FilmPackageId, FilmPackage>>,
    /// The MiniLFU cache for the full list of [`FilmPackage`]s
    film_package_list_cache: Option<Cache<(), Vec<FilmPackage>>>,
    /// The MiniLFU cache for [`Screen`]s
    screen_cache: Option<Cache<ScreenId, Screen>>,
    /// The MiniLFU cache for the full list of [`Screen`]s
    screen_list_cache: Option<Cache<(), Vec<Screen>>>,
    /// The MiniLFU cache for [`Attribute`]s
    attribute_cache: Option<Cache<AttributeId, Attribute>>,
    /// The MiniLFU cache for the full list of [`Attribute`]s
    attribute_list_cache: Option<Cache<(), Vec<Attribute>>>,
    /// The MiniLFU cache for the current [`Site`]
    site_cache: Option<Cache<(), Site>>,
}
impl Client {
    /// Create a new Veezi API client from a given base URL, access token, and
    /// [`reqwest::Client`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the URL provided is invalid.
    pub fn from_builder(builder: ClientBuilder) -> Result<Self, url::ParseError> {
        let ClientBuilder {
            http: http_client,
            base_url,
            token,
            session_cache,
            film_cache,
            film_package_cache,
            screen_cache,
            attribute_cache,
            site_cache,
        } = builder;

        debug!("Spawning new libveezi Client for API base: {base_url}");
        let base = Url::parse(&base_url)?;
        Ok(Self {
            http: http_client,
            base,
            token,

            session_cache: session_cache
                .map(|(ttl, max)| CacheBuilder::new(max).time_to_live(ttl).build()),
            session_list_cache: session_cache
                .map(|(ttl, _)| CacheBuilder::new(1).time_to_live(ttl).build()),
            web_session_list_cache: session_cache
                .map(|(ttl, _)| CacheBuilder::new(1).time_to_live(ttl).build()),
            film_cache: film_cache
                .map(|(ttl, max)| CacheBuilder::new(max).time_to_live(ttl).build()),
            film_list_cache: film_cache
                .map(|(ttl, _)| CacheBuilder::new(1).time_to_live(ttl).build()),
            film_package_cache: film_package_cache
                .map(|(ttl, max)| CacheBuilder::new(max).time_to_live(ttl).build()),
            film_package_list_cache: film_package_cache
                .map(|(ttl, _)| CacheBuilder::new(1).time_to_live(ttl).build()),
            screen_cache: screen_cache
                .map(|(ttl, max)| CacheBuilder::new(max).time_to_live(ttl).build()),
            screen_list_cache: screen_cache
                .map(|(ttl, _)| CacheBuilder::new(1).time_to_live(ttl).build()),
            attribute_cache: attribute_cache
                .map(|(ttl, max)| CacheBuilder::new(max).time_to_live(ttl).build()),
            attribute_list_cache: attribute_cache
                .map(|(ttl, _)| CacheBuilder::new(1).time_to_live(ttl).build()),
            site_cache: site_cache.map(|ttl| CacheBuilder::new(1).time_to_live(ttl).build()),
        })
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

    /// Invalidate all cached data
    pub fn invalidate_all_caches(&self) {
        self.invalidate_all_cached_sessions();
        self.invalidate_all_cached_web_sessions();
        self.invalidate_all_cached_films();
        self.invalidate_all_cached_film_packages();
        self.invalidate_all_cached_screens();
        self.invalidate_all_cached_attributes();
        self.invalidate_cached_site();
    }

    /// Get a list of all future [Session]s.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_sessions(&self) -> ApiResult<SessionList> {
        let fetch_raw = async {
            Ok(SessionList::from(
                self.get_json::<Vec<Session>>("v1/session").await?,
            ))
        };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.session_list_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(&()).await {
            debug!("SessionList cache hit");
            return Ok(cached);
        }

        debug!("SessionList cache miss, fetching from API");
        let sessions = fetch_raw.await?;
        cache.insert((), sessions.clone()).await;
        if let Some(session_cache) = &self.session_cache {
            for session in sessions.iter() {
                session_cache.insert(session.id, session.clone()).await;
            }
        }
        Ok(sessions)
    }
    /// Invalidate a cached [`Session`] by its ID
    ///
    /// As a side effect, this also invalidates the full session list cache
    pub async fn invalidate_cached_session(&self, id: SessionId) {
        if let Some(cache) = &self.session_cache {
            cache.invalidate(&id).await;
        }
        if let Some(cache) = &self.session_list_cache {
            cache.invalidate_all();
        }
    }
    /// Invalidate all cached [`Session`]s
    pub fn invalidate_all_cached_sessions(&self) {
        if let Some(cache) = &self.session_cache {
            cache.invalidate_all();
        }
        if let Some(cache) = &self.session_list_cache {
            cache.invalidate_all();
        }
    }

    /// Get a list of all future [`Session`]s that should be available for
    /// online sales.
    ///
    /// This asserts the following for each [`Session`]:
    /// - [`Session::sales_cut_off_time`] is in the future
    /// - [`Session::status`] is [`crate::session::SessionStatus::Open`]
    /// - [`Session::show_type`] is [`crate::session::ShowType::Public`]
    /// - [`Session::sales_via`] allows [`crate::session::SalesVia::www`] sales
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_web_sessions(&self) -> ApiResult<SessionList> {
        let fetch_raw = async {
            Ok(SessionList::from(
                self.get_json::<Vec<Session>>("v1/websession").await?,
            ))
        };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.web_session_list_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(&()).await {
            debug!("Web SessionList cache hit");
            return Ok(cached);
        }

        debug!("Web SessionList cache miss, fetching from API");
        let sessions = fetch_raw.await?;
        cache.insert((), sessions.clone()).await;
        if let Some(session_cache) = &self.session_cache {
            for session in sessions.iter() {
                // Although we are operating on only a subset of sessions, cache what we have
                session_cache.insert(session.id, session.clone()).await;
            }
        }

        Ok(sessions)
    }
    /// Invalidate all cached web [`Session`]s
    pub fn invalidate_all_cached_web_sessions(&self) {
        if let Some(cache) = &self.web_session_list_cache {
            cache.invalidate_all();
        }
    }

    /// Get a specific [Session] by its ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_session(&self, id: SessionId) -> ApiResult<Session> {
        let fetch_raw = async { self.get_json::<Session>(&format!("v1/session/{id}")).await };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.session_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(&id).await {
            debug!("Session cache hit for ID {id}");
            return Ok(cached);
        }

        debug!("Session cache miss for ID {id}, fetching from API");
        let session = fetch_raw.await?;
        cache.insert(id, session.clone()).await;
        Ok(session)
    }

    /// Get a list of all [Film]s in the Veezi system.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_films(&self) -> ApiResult<Vec<Film>> {
        // v4/film

        let fetch_raw = async { self.get_json::<Vec<Film>>("v4/film").await };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.film_list_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(&()).await {
            debug!("Film list cache hit");
            return Ok(cached);
        }

        debug!("Film list cache miss, fetching from API");
        let films = fetch_raw.await?;
        cache.insert((), films.clone()).await;
        if let Some(film_cache) = &self.film_cache {
            for film in &films {
                film_cache.insert(film.id.clone(), film.clone()).await;
            }
        }
        Ok(films)
    }
    /// Invalidate all cached [`Film`]s
    pub fn invalidate_all_cached_films(&self) {
        if let Some(cache) = &self.film_cache {
            cache.invalidate_all();
        }
        if let Some(cache) = &self.film_list_cache {
            cache.invalidate_all();
        }
    }
    /// Invalidate a cached [`Film`] by its ID
    ///
    /// As a side effect, this also invalidates the full film list cache
    pub async fn invalidate_cached_film(&self, id: &FilmId) {
        if let Some(cache) = &self.film_cache {
            cache.invalidate(id).await;
        }
        if let Some(cache) = &self.film_list_cache {
            cache.invalidate_all();
        }
    }

    /// Get a specific [`Film`] by its ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_film(&self, id: &FilmId) -> ApiResult<Film> {
        let fetch_raw = async {
            self.get_json::<Film>(&format!("v4/film/{}", id.as_str()))
                .await
        };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.film_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(id).await {
            debug!("Film cache hit for ID {id}");
            return Ok(cached);
        }

        debug!("Film cache miss for ID {id}, fetching from API");
        let film = fetch_raw.await?;
        cache.insert(id.clone(), film.clone()).await;
        Ok(film)
    }

    /// Get a specific [`Film`] by its exact [`Film::title`]. If multiple films
    /// have the same title, the first one found will be returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails, or None if
    /// no film with the given title is found.
    pub async fn get_film_by_title(&self, title: &str) -> ApiResult<Option<Film>> {
        let films = self.list_films().await?;
        Ok(films.into_iter().find(|film| film.title == title))
    }

    /// Get a specific [`Film`] by its exact [`Film::short_name`]. If multiple
    /// films have the same short name, the first one found will be
    /// returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails, or None if
    /// no film with the given short name is found.
    pub async fn get_film_by_short_name(&self, short_name: &str) -> ApiResult<Option<Film>> {
        let films = self.list_films().await?;
        Ok(films.into_iter().find(|film| film.short_name == short_name))
    }

    /// Get a specific [`Film`] by its exact [`Film::signage_text`]. If multiple
    /// films have the same signage text, the first one found will be
    /// returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails, or None if
    /// no film with the given signage text is found.
    pub async fn get_film_by_signage_text(&self, signage_text: &str) -> ApiResult<Option<Film>> {
        let films = self.list_films().await?;
        Ok(films
            .into_iter()
            .find(|film| film.signage_text == signage_text))
    }

    /// Get a list of all [`Film`]s with a specific [`Film::genre`] string.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_films_by_genre(&self, genre: &str) -> ApiResult<Vec<Film>> {
        let films = self.list_films().await?;
        Ok(films
            .into_iter()
            .filter(|film| film.genre == genre)
            .collect())
    }

    /// Get a list of all [`Film`]s by a specific [`Film::distributor`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_films_by_distributor(&self, distributor: &str) -> ApiResult<Vec<Film>> {
        let films = self.list_films().await?;
        Ok(films
            .into_iter()
            .filter(|film| film.distributor == distributor)
            .collect())
    }

    /// Get only the films that have sessions scheduled in the included time
    /// range.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_films_with_sessions_in_time_range(
        &self,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> ApiResult<Vec<Film>> {
        // using our existing methods, no http
        self.list_sessions()
            .await?
            .filter_by_time_range(start, end)
            .films(self)
            .await
    }

    /// Get only the films that have sessions scheduled in the included date
    /// range.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_films_with_sessions_in_date_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> ApiResult<Vec<Film>> {
        // using our existing methods, no http
        self.list_sessions()
            .await?
            .filter_by_date_range(start, end)
            .films(self)
            .await
    }

    /// Get a list of all [`FilmPackage`]s in the Veezi system.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_film_packages(&self) -> ApiResult<Vec<FilmPackage>> {
        let fetch_raw = async { self.get_json::<Vec<FilmPackage>>("v1/filmpackage").await };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.film_package_list_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(&()).await {
            debug!("FilmPackage list cache hit");
            return Ok(cached);
        }

        debug!("FilmPackage list cache miss, fetching from API");
        let packages = fetch_raw.await?;
        cache.insert((), packages.clone()).await;
        if let Some(package_cache) = &self.film_package_cache {
            for package in &packages {
                package_cache.insert(package.id, package.clone()).await;
            }
        }
        Ok(packages)
    }
    /// Invalidate all cached [`FilmPackage`]s
    pub fn invalidate_all_cached_film_packages(&self) {
        if let Some(cache) = &self.film_package_cache {
            cache.invalidate_all();
        }
        if let Some(cache) = &self.film_package_list_cache {
            cache.invalidate_all();
        }
    }
    /// Invalidate a cached [`FilmPackage`] by its ID
    ///
    /// As a side effect, this also invalidates the full film package list cache
    pub async fn invalidate_cached_film_package(&self, id: FilmPackageId) {
        if let Some(cache) = &self.film_package_cache {
            cache.invalidate(&id).await;
        }
        if let Some(cache) = &self.film_package_list_cache {
            cache.invalidate_all();
        }
    }

    /// Get a specific [`FilmPackage`] by its exact [`FilmPackage::title`]. If
    /// multiple packages have the same title, the first one found will be
    /// returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails, or None if
    /// no package with the given title is found.
    pub async fn get_film_package_by_title(&self, title: &str) -> ApiResult<Option<FilmPackage>> {
        let packages = self.list_film_packages().await?;
        Ok(packages.into_iter().find(|package| package.title == title))
    }

    /// Get a list of all [`FilmPackage`]s containing a specific [`FilmId`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_film_packages_by_film_id(
        &self,
        film_id: &FilmId,
    ) -> ApiResult<Vec<FilmPackage>> {
        let packages = self.list_film_packages().await?;
        Ok(packages
            .into_iter()
            .filter(|package| package.films.iter().any(|pf| pf.film_id == *film_id))
            .collect())
    }

    /// Get a specific [`FilmPackage`] by its ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_film_package(&self, id: FilmPackageId) -> ApiResult<FilmPackage> {
        let fetch_raw = async {
            self.get_json::<FilmPackage>(&format!("v1/filmpackage/{id}"))
                .await
        };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.film_package_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(&id).await {
            debug!("FilmPackage cache hit for ID {id}");
            return Ok(cached);
        }

        debug!("FilmPackage cache miss for ID {id}, fetching from API");
        let package = fetch_raw.await?;
        cache.insert(id, package.clone()).await;
        Ok(package)
    }

    /// Get a list of all [`Screen`]s in the current site.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_screens(&self) -> ApiResult<Vec<Screen>> {
        let fetch_raw = async { self.get_json::<Vec<Screen>>("v1/screen").await };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.screen_list_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(&()).await {
            debug!("Screen list cache hit");
            return Ok(cached);
        }

        debug!("Screen list cache miss, fetching from API");
        let screens = fetch_raw.await?;
        cache.insert((), screens.clone()).await;
        if let Some(screen_cache) = &self.screen_cache {
            for screen in &screens {
                screen_cache.insert(screen.id, screen.clone()).await;
            }
        }
        Ok(screens)
    }
    /// Invalidate all cached [`Screen`]s
    pub fn invalidate_all_cached_screens(&self) {
        if let Some(cache) = &self.screen_cache {
            cache.invalidate_all();
        }
        if let Some(cache) = &self.screen_list_cache {
            cache.invalidate_all();
        }
    }
    /// Invalidate a cached [`Screen`] by its ID
    ///
    /// As a side effect, this also invalidates the full screen list cache
    pub async fn invalidate_cached_screen(&self, id: ScreenId) {
        if let Some(cache) = &self.screen_cache {
            cache.invalidate(&id).await;
        }
        if let Some(cache) = &self.screen_list_cache {
            cache.invalidate_all();
        }
    }

    /// Get a specific [`Screen`] by its ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_screen(&self, id: ScreenId) -> ApiResult<Screen> {
        let fetch_raw = async { self.get_json::<Screen>(&format!("v1/screen/{id}")).await };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.screen_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(&id).await {
            debug!("Screen cache hit for ID {id}");
            return Ok(cached);
        }

        debug!("Screen cache miss for ID {id}, fetching from API");
        let screen = fetch_raw.await?;
        cache.insert(id, screen.clone()).await;
        Ok(screen)
    }

    /// Get a specific [`Screen`] by its exact [`Screen::screen_number`]. If
    /// multiple screens have the same screen number, the first one found will
    /// be returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails, or None if
    /// no screen with the given screen number is found.
    pub async fn get_screen_by_number(&self, screen_number: String) -> ApiResult<Option<Screen>> {
        let screens = self.list_screens().await?;
        Ok(screens
            .into_iter()
            .find(|screen| screen.screen_number == screen_number))
    }

    /// Get the [`Site`] information for the current Veezi site.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_site(&self) -> ApiResult<Site> {
        let fetch_raw = async { self.get_json::<Site>("v1/site").await };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.site_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(&()).await {
            debug!("Site cache hit");
            return Ok(cached);
        }

        debug!("Site cache miss, fetching from API");
        let site = fetch_raw.await?;
        cache.insert((), site.clone()).await;
        Ok(site)
    }
    /// Invalidate the cached [`Site`]
    pub fn invalidate_cached_site(&self) {
        if let Some(cache) = &self.site_cache {
            cache.invalidate_all();
        }
    }

    /// Get a list of all [`Attribute`]s set in the site.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_attributes(&self) -> ApiResult<Vec<Attribute>> {
        let fetch_raw = async { self.get_json::<Vec<Attribute>>("v1/attribute").await };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.attribute_list_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(&()).await {
            debug!("Attribute list cache hit");
            return Ok(cached);
        }

        debug!("Attribute list cache miss, fetching from API");
        let attributes = fetch_raw.await?;
        cache.insert((), attributes.clone()).await;
        if let Some(attribute_cache) = &self.attribute_cache {
            for attribute in &attributes {
                attribute_cache
                    .insert(attribute.id.clone(), attribute.clone())
                    .await;
            }
        }
        Ok(attributes)
    }
    /// Invalidate all cached [`Attribute`]s
    pub fn invalidate_all_cached_attributes(&self) {
        if let Some(cache) = &self.attribute_cache {
            cache.invalidate_all();
        }
        if let Some(cache) = &self.attribute_list_cache {
            cache.invalidate_all();
        }
    }
    /// Invalidate a cached [`Attribute`] by its ID
    ///
    /// As a side effect, this also invalidates the full attribute list cache
    pub async fn invalidate_cached_attribute(&self, id: &AttributeId) {
        if let Some(cache) = &self.attribute_cache {
            cache.invalidate(id).await;
        }
        if let Some(cache) = &self.attribute_list_cache {
            cache.invalidate_all();
        }
    }

    /// Get a specific [`Attribute`] by its ID.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_attribute(&self, id: &AttributeId) -> ApiResult<Attribute> {
        let fetch_raw = async {
            self.get_json::<Attribute>(&format!("v1/attribute/{}", id.as_str()))
                .await
        };

        // Fetch from API if no cache is configured
        let Some(cache) = &self.attribute_cache else {
            return fetch_raw.await;
        };

        // Try to get from cache
        if let Some(cached) = cache.get(id).await {
            debug!("Attribute cache hit for ID {id}");
            return Ok(cached);
        }

        debug!("Attribute cache miss for ID {id}, fetching from API");
        let attribute = fetch_raw.await?;
        cache.insert(id.clone(), attribute.clone()).await;
        Ok(attribute)
    }

    /// Get a specific [`Attribute`] by its exact [`Attribute::short_name`]. If
    /// multiple attributes have the same short name, the first one found
    /// will be returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails, or None if
    /// no attribute with the given short name is found.
    pub async fn get_attribute_by_short_name(
        &self,
        short_name: &str,
    ) -> ApiResult<Option<Attribute>> {
        let attributes = self.list_attributes().await?;
        Ok(attributes
            .into_iter()
            .find(|attr| attr.short_name == short_name))
    }

    /// Get a specific [`Attribute`] by its exact [`Attribute::description`]. If
    /// multiple attributes have the same description, the first one found
    /// will be returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails, or None if
    /// no attribute with the given description is found.
    pub async fn get_attribute_by_description(
        &self,
        description: &str,
    ) -> ApiResult<Option<Attribute>> {
        let attributes = self.list_attributes().await?;
        Ok(attributes
            .into_iter()
            .find(|attr| attr.description == description))
    }

    /// Get all sessions scheduled for today (in UTC).
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_sessions_today(&self) -> ApiResult<SessionList> {
        Ok(self.list_sessions().await?.filter_today())
    }

    /// Get all sessions scheduled for tomorrow (in UTC).
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_sessions_tomorrow(&self) -> ApiResult<SessionList> {
        Ok(self.list_sessions().await?.filter_tomorrow())
    }

    /// Get all sessions that are currently open for sales.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_sessions_open_for_sales(&self) -> ApiResult<SessionList> {
        Ok(self.list_sessions().await?.filter_open_for_sales())
    }

    /// Get all currently active films (films with status Active).
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_active_films(&self) -> ApiResult<Vec<Film>> {
        let films = self.list_films().await?;
        Ok(films
            .into_iter()
            .filter(|film| film.status == FilmStatus::Active)
            .collect())
    }

    /// Get all films that are currently showing (have sessions scheduled).
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn list_films_now_showing(&self) -> ApiResult<Vec<Film>> {
        let sessions = self.list_sessions().await?;
        sessions.films(self).await
    }

    /// Get a map of film IDs to their session counts.
    ///
    /// This is useful for determining which films have the most scheduled
    /// sessions.
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn get_session_counts_by_film(&self) -> ApiResult<Vec<(FilmId, usize)>> {
        let sessions = self.list_sessions().await?;
        let grouped = sessions.group_by_film();
        Ok(grouped
            .into_iter()
            .map(|(film_id, sessions)| (film_id.clone(), sessions.len()))
            .collect())
    }

    /// Fetch multiple films by their IDs in a single batch operation.
    ///
    /// This is more convenient than calling `get_film` multiple times when you
    /// need to fetch multiple films.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the API requests fail.
    pub async fn get_films_batch(&self, film_ids: &[FilmId]) -> ApiResult<Vec<Film>> {
        let mut films = Vec::new();
        for film_id in film_ids {
            let film = self.get_film(film_id).await?;
            films.push(film);
        }
        Ok(films)
    }

    /// Fetch multiple sessions by their IDs in a single batch operation.
    ///
    /// This is more convenient than calling `get_session` multiple times when
    /// you need to fetch multiple sessions.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the API requests fail.
    pub async fn get_sessions_batch(&self, session_ids: &[SessionId]) -> ApiResult<Vec<Session>> {
        let mut sessions = Vec::new();
        for session_id in session_ids {
            let session = self.get_session(*session_id).await?;
            sessions.push(session);
        }
        Ok(sessions)
    }
}
