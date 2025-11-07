//! A particular site, or screening location.

use std::fmt::Debug;

use serde::Deserialize;

/// Information about the current Veezi site
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Site {
    /// The name of the site
    pub name: String,
    /// The short name of the site
    pub short_name: String,
    /// The legal name of the site
    pub legal_name: String,
    /// The national code of the site
    pub national_code: Option<String>,
    /// Line 1 of the site's address
    pub address_1: Option<String>,
    /// Line 2 of the site's address
    pub address_2: Option<String>,
    /// Line 3 of the site's address
    pub address_3: Option<String>,
    /// The site's postal code
    pub post_code: Option<String>,
    /// The site's primary phone number
    pub phone_1: Option<String>,
    /// The site's secondary phone number
    pub phone_2: Option<String>,
    /// The site's fax number
    pub fax: Option<String>,
    /// The site's sales tax registration number
    pub sales_tax_registration: Option<String>,
    /// Line 1 of the site's ticket message
    pub ticket_message_1: Option<String>,
    /// Line 2 of the site's ticket message
    pub ticket_message_2: Option<String>,
    /// Line 1 of the site's receipt message
    pub receipt_message_1: Option<String>,
    /// Line 2 of the site's receipt message
    pub receipt_message_2: Option<String>,
    /// Line 3 of the site's receipt message
    pub receipt_message_3: Option<String>,
    /// Line 4 of the site's receipt message
    pub receipt_message_4: Option<String>,
    /// Line 5 of the site's receipt message
    pub receipt_message_5: Option<String>,
    /// Line 6 of the site's receipt message
    pub receipt_message_6: Option<String>,
    /// The time zone identifier for the site
    pub time_zone_identifier: String,
    /// The country where the site is located
    pub country: String,
    /// The list of screen IDs associated with the site
    #[serde(deserialize_with = "crate::utils::deserialize_id_array")]
    pub screens: Vec<u32>,
}
