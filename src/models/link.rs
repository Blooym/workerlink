use serde::{Deserialize, Serialize};
use url::Url;
use worker::Date;

/// Represents a Link.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkModel {
    /// The URL that this Link redirects to.
    pub url: Url,
    /// Whether or not this link is disabled.
    // TODO: See if there is a better term than 'disabled' to represent this value?
    pub disabled: bool,
    /// The amount times this Link has been viewed.
    pub views: u64,
    /// The maximum amount of times this Link can be viewed before it becomes invalid.
    pub max_views: Option<u64>,
    /// The UNIX timestamp for when the Link will become invalid.
    pub expiry_timestamp: Option<u64>,
    /// The time this link was last viewed.
    pub last_viewed_timestamp: Option<u64>,
    /// The UNIX timestamp from when the Link was created.
    pub created_at_timestamp: u64,
    /// The UNIX timestamp from when the Link was last modified.
    pub modified_at_timestamp: u64,
}

/// Arguments for building a Link.
pub struct LinkBuilderArgs {
    /// The URL that this Link redirects to.
    pub url: Url,
    /// Whether or not this link has been disabled.
    pub disabled: bool,
    /// The maximum amount of times this Link can be viewed before it becomes invalid.
    pub max_views: Option<u64>,
    /// The UNIX timestamp for when the Link will become invalid.
    pub expiry_timestamp: Option<u64>,
}

impl LinkModel {
    /// Creates a new Link using the given builder while setting some default values.
    pub fn new(args: LinkBuilderArgs) -> Self {
        Self {
            url: args.url,
            disabled: args.disabled,
            views: 0,
            max_views: args.max_views,
            expiry_timestamp: args.expiry_timestamp,
            last_viewed_timestamp: None,
            created_at_timestamp: Date::now().as_millis(),
            modified_at_timestamp: Date::now().as_millis(),
        }
    }

    /// Consumes the current Link and creates a modified version of it while retaining some of the original data.
    pub fn modify(self, args: LinkBuilderArgs) -> Self {
        Self {
            url: args.url,
            disabled: args.disabled,
            max_views: args.max_views,
            expiry_timestamp: args.expiry_timestamp,
            modified_at_timestamp: Date::now().as_millis(),
            ..self
        }
    }

    /// Increments the number of visits for this Link in-place.
    pub fn increment_visits(&mut self) -> () {
        self.last_viewed_timestamp = Some(Date::now().as_millis());
        self.views += 1;
    }
}
