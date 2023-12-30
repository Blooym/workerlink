use serde::Deserialize;
use std::time::Duration;
use validator::Validate;

/// The request for creating/updating a Shortlink.
#[derive(Validate, Deserialize)]
pub struct CreateShortlinkRequestBody {
    #[validate(url)]
    pub url: String,
    #[serde(default)]
    pub overwrite: bool,
    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub expire_in: Option<Duration>,
    #[serde(default)]
    #[validate(range(min = 1))]
    pub max_views: Option<u64>,
    #[serde(default)]
    pub disabled: bool,
}
