pub mod cloudflare_storage_driver;

use async_trait::async_trait;

#[async_trait(?Send)]
/// Represents a generic storage driver that can be used to store keys and values.
pub trait StorageDriver {
    /// Get the value for the given key.
    async fn get_value(&self, key: &str) -> Option<String>;

    /// Set a key's value.
    async fn set_value(&self, key: &str, value: &str) -> bool;

    /// Deletes the key's value.
    async fn delete_value(&self, key: &str) -> bool;
}
