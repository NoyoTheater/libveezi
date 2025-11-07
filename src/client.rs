//! The [`Client`] for interfacing with the Veezi API

use std::{fmt::Debug, time::Duration};

use log::debug;
use moka::future::{Cache, CacheBuilder};
use reqwest::Url;
use serde::de::DeserializeOwned;

use crate::{
    attr::{Attribute, AttributeId},
    error::ApiResult,
    film::{Film, FilmId},
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
            .with_film_cache(Duration::from_mins(5), 500)
            .with_film_package_cache(Duration::from_mins(5), 500)
            .with_screen_cache(Duration::from_hours(1), 100)
            .with_attribute_cache(Duration::from_mins(5), 500)
            .with_site_cache(Duration::from_mins(5))
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
}
