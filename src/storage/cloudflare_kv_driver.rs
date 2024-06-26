use super::StorageDriver;
use serde::{de::DeserializeOwned, Serialize};
use worker::kv::KvStore;

/// The binding name for the KV namespace that stores Link data.
pub const CLOUDFLARE_KV_BINDING: &str = "links";

/// A driver for Cloudflare KV.
///
/// https://developers.cloudflare.com/kv/
pub struct CloudflareKVDriver {
    /// The underlying Cloudflare Key-Value struct.
    kv_store: KvStore,
}

impl CloudflareKVDriver {
    /// Create a new instance of [`CloudflareKVDriver`].
    pub fn new(store: KvStore) -> CloudflareKVDriver {
        CloudflareKVDriver { kv_store: store }
    }
}

impl StorageDriver for CloudflareKVDriver {
    // async fn exists(&self, key: &str) -> bool {
    // self.get(key).await.is_some()
    // }

    async fn get(&self, key: &str) -> Option<String> {
        self.kv_store.get(key).text().await.unwrap()
    }

    async fn get_deserialized_json<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let raw_json = match self.get(key).await {
            Some(raw_json) => raw_json,
            None => return None,
        };

        match serde_json::from_str::<T>(&raw_json) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    async fn set(&self, key: &str, value: &str) -> bool {
        self.kv_store
            .put(key, value)
            .unwrap()
            .execute()
            .await
            .is_ok()
    }

    async fn set_serialized_json<T: Serialize>(&self, key: &str, value: T) -> bool {
        let serialized = match serde_json::to_string(&value) {
            Ok(serialized) => serialized,
            Err(_) => return false,
        };
        self.set(key, &serialized).await
    }

    async fn delete(&self, key: &str) -> bool {
        self.kv_store.delete(key).await.is_ok()
    }
}
