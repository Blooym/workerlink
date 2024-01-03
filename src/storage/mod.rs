pub mod cloudflare_kv_driver;

use serde::{de::DeserializeOwned, Serialize};

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

    /// Check if a key exists.
    async fn exists(&self, key: &str) -> bool;

    /// Delete a key.
    async fn delete(&self, key: &str) -> bool;
}
