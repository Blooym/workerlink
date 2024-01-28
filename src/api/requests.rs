use serde::Deserialize;
use std::time::Duration;
use url::Url;
use validator::Validate;

/// Represents the request body for creating/updating a Link.
#[derive(Debug, Validate, Deserialize)]
pub struct CreateLinkRequestBody {
    pub url: Url,
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
