use serde::{Deserialize, Serialize};
use url::Url;
use worker::Date;

/// Represents a link.
#[derive(Debug, Serialize, Deserialize)]
pub struct LinkModel {
    /// The URL to redirect to upon visiting this link.
    pub url: Url,
    /// Whether or not this link is disabled.
    // TODO: See if there is a better term than 'disabled' to represent this value?
    pub disabled: bool,
    /// The amount times this link has been viewed.
    pub views: u64,
    /// The maximum amount of times this link can be viewed before it becomes invalid.
    pub max_views: Option<u64>,
    /// The UNIX timestamp for when the link will become invalid.
    pub expiry_timestamp: Option<u64>,
    /// The time this link was viewed..
    pub last_viewed_timestamp: Option<u64>,
    /// The UNIX timestamp of original creation.
    pub created_at_timestamp: u64,
    /// The UNIX timestamp of last modification.
    pub modified_at_timestamp: u64,
}

/// Arguments for building a link.
pub struct LinkBuilderArgs {
    /// The URL to redirect to.
    pub url: Url,
    /// Whether or not this link has been disabled.
    pub disabled: bool,
    /// The maximum amount of times this link can be viewed before it becomes invalid.
    pub max_views: Option<u64>,
    /// The UNIX timestamp for when the link will become invalid.
    pub expiry_timestamp: Option<u64>,
}

impl LinkModel {
    /// Create a new model using the given builder while setting some default values.
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

    /// Consume the current model and creates a modified version of it with of the original data.
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

    /// Increment the number of visits for this link in-place.
    pub fn increment_visits(&mut self) {
        self.last_viewed_timestamp = Some(Date::now().as_millis());
        self.views += 1;
    }

    /// Whether or not this link is still considered valid after checking:
    ///     - It's expiry date compared to the current date.
    ///     - It's max view count compared to current view count
    pub fn is_valid(&self) -> bool {
        if let Some(expires_at_ms) = self.expiry_timestamp {
            if Date::now().as_millis() > expires_at_ms {
                return false;
            }
        }

        if let Some(max_views) = self.max_views {
            if self.views >= max_views {
                return false;
            }
        }

        true
    }
}
