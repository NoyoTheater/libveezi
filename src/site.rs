//! A particular site, or screening location.

use serde::Deserialize;
use std::fmt::Debug;

/// Information about the current Veezi site
#[derive(Deserialize, Debug, PartialEq, Eq)]
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
    #[serde(deserialize_with = "crate::utils::deserialize_id_array")]
    pub screens: Vec<u32>,
}
