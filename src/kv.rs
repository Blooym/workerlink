use worker::{kv::KvStore, Result, RouteContext};

/// The binding name for the KV namespace that stores the links and their IDs.
const LOCATIONS_KV_BINDING: &str = "locations";

/// Gets the KV store.
pub fn get_kv_store(ctx: RouteContext<()>) -> Result<KvStore> {
    ctx.kv(LOCATIONS_KV_BINDING)
}
