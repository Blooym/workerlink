use super::StorageDriver;
use async_trait::async_trait;
use worker::kv::KvStore;

/// The binding name for the KV namespace that stores the links and their IDs.
pub const CLOUDFLARE_KV_BINDING: &str = "locations";

/// A driver that uses Cloudflare's Key-Value storage.
pub struct CloudflareStorageDriver {
    kv_store: KvStore,
}

impl CloudflareStorageDriver {
    /// Create a new instance of [`CloudflareStorageDriver`].
    pub fn new(store: KvStore) -> CloudflareStorageDriver {
        CloudflareStorageDriver { kv_store: store }
    }
}

#[async_trait(?Send)]
impl StorageDriver for CloudflareStorageDriver {
    async fn get_value(&self, key: &str) -> Option<String> {
        self.kv_store.get(key).text().await.unwrap()
    }

    async fn set_value(&self, key: &str, value: &str) -> bool {
        self.kv_store
            .put(key, value)
            .unwrap()
            .execute()
            .await
            .is_ok()
    }

    async fn delete_value(&self, key: &str) -> bool {
        self.kv_store.delete(key).await.is_ok()
    }
}
