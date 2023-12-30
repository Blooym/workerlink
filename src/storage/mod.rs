pub mod cloudflare_storage_driver;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use worker::Url;

// TODO: Swap out boolean returns for Result types.
// TODO: Improve error bubbling up instead of mindlessly calling unwrap()s

/// Represents a generic storage driver that can be used to store keys and values.
pub trait StorageDriver {
    /// Get the value of a key.
    async fn get(&self, key: &str) -> Option<String>;

    /// Get the value of a key with automatic deserialization into the given struct from JSON.
    async fn get_from_json<T: DeserializeOwned>(&self, key: &str) -> Option<T>;

    /// Set the value of a key.
    async fn set(&self, key: &str, value: &str) -> bool;

    /// Set the value of a key with automatic serialization of the given struct into JSON.
    async fn set_as_json<T: Serialize>(&self, key: &str, value: T) -> bool;

    /// See if a key exists.
    async fn exists(&self, key: &str) -> bool;

    /// Delete a key.
    async fn delete(&self, key: &str) -> bool;
}

/// Represents a Shortlink with all its customization fields.
#[derive(Serialize, Deserialize, Debug)]
pub struct ShortlinkModel {
    /// The URL that this shortlink directs to.
    pub url: Url,
    /// The amount of views this shortlink has received.
    pub views: u64,
    /// The maximum amount of times this link can be visited before become invalid.
    pub max_views: Option<u64>,
    /// The UNIX timestamp for when the shortlink will become invalid.
    pub expiry_timestamp: Option<u64>,
    /// The time this shortlink was created.
    pub created_at_timestamp: u64,
    /// The time this shortlink was last visited.
    pub last_visited_timestamp: Option<u64>,
    /// Whether or not this shortlink is disabled.
    pub disabled: bool,
}
