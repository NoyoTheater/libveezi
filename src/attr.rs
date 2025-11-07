//! [`Attribute`]s that can be associated with [`Session`]s and [`Film`]s

use std::fmt::Debug;

use serde::Deserialize;

use crate::{client::Client, error::ApiResult, session::SessionList};

/// An attribute that can be associated with [Session]s
#[derive(Deserialize, Debug, PartialEq, Eq)]
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
    /// Whether to show this attribute on sessions that have no complimentary
    /// tickets
    pub show_on_sessions_with_no_comps: bool,
}
impl Attribute {
    /// Get a list of all future [Session]s containing this [Attribute]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn sessions(&self, client: &Client) -> ApiResult<SessionList> {
        Ok(client
            .list_sessions()
            .await?
            .filter_containing_attribute(&self.id))
    }
}
