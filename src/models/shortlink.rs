use serde::{Deserialize, Serialize};
use url::Url;
use worker::Date;

/// Represents a Shortlink.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShortlinkModel {
    /// The URL that this Shortlink redirects to.
    pub url: Url,
    /// Whether or not this shortlink is disabled.
    // TODO: See if there is a better term than 'disabled' to represent this value?
    pub disabled: bool,
    /// The amount times this Shortlink has been viewed.
    pub views: u64,
    /// The maximum amount of times this Shortlink can be viewed before it becomes invalid.
    pub max_views: Option<u64>,
    /// The UNIX timestamp for when the Shortlink will become invalid.
    pub expiry_timestamp: Option<u64>,
    /// The time this shortlink was last viewed.
    pub last_viewed_timestamp: Option<u64>,
    /// The UNIX timestamp from when the Shortlink was created.
    pub created_at_timestamp: u64,
    /// The UNIX timestamp from when the Shortlink was last modified.
    pub modified_at_timestamp: u64,
}

/// Arguments for building a Shortlink.
pub struct ShortlinkBuilderArgs {
    /// The URL that this Shortlink redirects to.
    pub url: Url,
    /// Whether or not this shortlink has been disabled.
    pub disabled: bool,
    /// The maximum amount of times this Shortlink can be viewed before it becomes invalid.
    pub max_views: Option<u64>,
    /// The UNIX timestamp for when the Shortlink will become invalid.
    pub expiry_timestamp: Option<u64>,
}

impl ShortlinkModel {
    /// Creates a new Shortlink using the given builder while setting some default values.
    pub fn new(args: ShortlinkBuilderArgs) -> Self {
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

    /// Consumes the current Shortlink and creates a modified version of it while retaining some of the original data.
    pub fn modify(self, args: ShortlinkBuilderArgs) -> Self {
        Self {
            url: args.url,
            disabled: args.disabled,
            max_views: args.max_views,
            expiry_timestamp: args.expiry_timestamp,
            modified_at_timestamp: Date::now().as_millis(),
            ..self
        }
    }

    /// Increments the number of visits for this Shortlink in-place.
    pub fn increment_visits(&mut self) -> () {
        self.last_viewed_timestamp = Some(Date::now().as_millis());
        self.views += 1;
    }
}
