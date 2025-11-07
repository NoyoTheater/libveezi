//! A [`Screen`] on a specific Veezi [`Site`]

use std::fmt::Debug;

use serde::Deserialize;

use crate::{client::Client, error::ApiResult, session::SessionList};

/// A particular screen (auditorium) in the Veezi system
#[derive(Deserialize, Debug, PartialEq, Eq)]
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
    /// Get a list of all future [Session]s for this [`Screen`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn sessions(&self, client: &Client) -> ApiResult<SessionList> {
        Ok(client.list_sessions().await?.filter_by_screen(self.id))
    }
}
