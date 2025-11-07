//! Internal utilities for libveezi

use serde::{Deserialize, Deserializer};

/// Helper function used to deserialize `[{Id:1},{Id:2}]` into `vec![1, 2]`
pub fn deserialize_id_array<'de, D>(deserializer: D) -> Result<Vec<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    #[allow(clippy::missing_docs_in_private_items)]
    struct IdHelper {
        id: u32,
    }

    let helper_vec: Vec<IdHelper> = Deserialize::deserialize(deserializer)?;
    Ok(helper_vec.into_iter().map(|attr| attr.id).collect())
}
