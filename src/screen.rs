//! A [`Screen`] on a specific Veezi [`Site`]

use std::fmt::{self, Debug, Display, Formatter};

use serde::Deserialize;

use crate::{client::Client, error::ApiResult, session::SessionList};

/// The unique ID of a [`Screen`]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(transparent)]
pub struct ScreenId(u32);
impl ScreenId {
    /// Get the numeric ID of this [`ScreenId`]
    #[must_use]
    pub const fn into_u32(self) -> u32 {
        self.0
    }

    /// Fetch the full [`Screen`] associated with this [`ScreenId`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn fetch(self, client: &Client) -> ApiResult<Screen> {
        client.get_screen(self).await
    }
}
impl Display for ScreenId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A particular screen (auditorium) in the Veezi system
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Screen {
    /// The unique ID of the screen
    pub id: ScreenId,
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
