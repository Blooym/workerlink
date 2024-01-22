use crate::models::shortlink::ShortlinkModel;
use serde::Serialize;
use url::Url;

/// Represents the response body for successfully creating a Shortlink.
#[derive(Serialize)]
pub struct CreateShortlinkResponse {
    pub url: String,
    pub expiry_timestamp: Option<u64>,
    pub max_views: Option<u64>,
    pub disabled: bool,
}

impl CreateShortlinkResponse {
    pub fn from_model(shortlink_model: &ShortlinkModel, web_url: Url) -> Self {
        CreateShortlinkResponse {
            url: web_url.to_string(),
            expiry_timestamp: shortlink_model.expiry_timestamp,
            max_views: shortlink_model.max_views,
            disabled: shortlink_model.disabled,
        }
    }
}
