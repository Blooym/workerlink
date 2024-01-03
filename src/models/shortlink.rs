use serde::{Deserialize, Serialize};
use url::Url;
use worker::Date;

/// Represents a Shortlink with all its customization fields.
#[derive(Serialize, Deserialize, Debug)]
pub struct ShortlinkModel {
    /// The URL that this Shortlink redirects to.
    pub url: Url,
    /// Whether or not this shortlink is disabled.
    pub disabled: bool,
    /// The amount times this Shortlink has been viewed.
    pub views: u64,
    /// The maximum amount of times this Shortlink can be viewed before it becomes invalid.
    pub max_views: Option<u64>,
    /// The UNIX timestamp for when the Shortlink will become invalid.
    pub expiry_timestamp: Option<u64>,
    /// The time this shortlink was last visited.
    pub last_visited_timestamp: Option<u64>,
    /// The UNIX timestamp from when the Shortlink was created.
    pub created_at_timestamp: u64,
    /// The UNIX timestamp from when the Shortlink was last updated.
    pub modified_at_timestamp: u64,
}

pub struct ShortlinkCreateArgs {
    pub url: Url,
    pub disabled: bool,
    pub max_views: Option<u64>,
    pub expiry_timestamp: Option<u64>,
}

impl ShortlinkModel {
    pub fn new(args: ShortlinkCreateArgs) -> Self {
        Self {
            url: args.url,
            disabled: args.disabled,
            views: 0,
            max_views: args.max_views,
            expiry_timestamp: args.expiry_timestamp,
            last_visited_timestamp: None,
            created_at_timestamp: Date::now().as_millis(),
            modified_at_timestamp: Date::now().as_millis(),
        }
    }

    pub fn update(self, args: ShortlinkCreateArgs) -> Self {
        Self {
            url: args.url,
            disabled: args.disabled,
            max_views: args.max_views,
            expiry_timestamp: args.expiry_timestamp,
            modified_at_timestamp: Date::now().as_millis(),
            ..self
        }
    }

    pub fn increment_visits(&mut self) -> &mut Self {
        self.last_visited_timestamp = Some(Date::now().as_millis());
        self.views += 1;
        self
    }
}
