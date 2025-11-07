//! [`Attribute`]s that can be associated with [`Session`]s and [`Film`]s

use std::fmt::{self, Debug, Display, Formatter};

use serde::Deserialize;

use crate::{client::Client, error::ApiResult, session::SessionList};

/// The unique ID of an [`Attribute`]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(transparent)]
pub struct AttributeId(String);
impl AttributeId {
    /// Get the string representation of this [`AttributeId`]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Fetch the full [`Attribute`] associated with this [`AttributeId`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails.
    pub async fn fetch(&self, client: &Client) -> ApiResult<Attribute> {
        client.get_attribute(self).await
    }
}
impl Display for AttributeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An attribute that can be associated with [Session]s
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Attribute {
    /// The unique ID of the attribute
    pub id: AttributeId,
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
