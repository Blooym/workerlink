use crate::models::link::LinkModel;
use serde::Serialize;
use url::Url;

/// Represents the response body for successfully creating a Link.
#[derive(Debug, Serialize)]
pub struct CreateLinkResponse {
    pub url: String,
    pub expiry_timestamp: Option<u64>,
    pub max_views: Option<u64>,
    pub disabled: bool,
}

impl CreateLinkResponse {
    pub fn from_model(link_model: &LinkModel, web_url: Url) -> Self {
        CreateLinkResponse {
            url: web_url.to_string(),
            expiry_timestamp: link_model.expiry_timestamp,
            max_views: link_model.max_views,
            disabled: link_model.disabled,
        }
    }
}
