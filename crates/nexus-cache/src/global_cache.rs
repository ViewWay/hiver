//! Global string-keyed cache store used by `#[cacheable]` proc-macros.
//! 供 `#[cacheable]` 过程宏使用的全局字符串键缓存存储。

use std::sync::OnceLock;

use dashmap::DashMap;

static GLOBAL_CACHES: OnceLock<DashMap<String, DashMap<String, String>>> = OnceLock::new();

fn caches() -> &'static DashMap<String, DashMap<String, String>> {
    GLOBAL_CACHES.get_or_init(DashMap::new)
}

/// Returns the global cache manager (cache-name → key-value store).
/// 返回全局缓存管理器（缓存名 → 键值存储）。
pub fn global_cache_manager() -> &'static DashMap<String, DashMap<String, String>> {
    caches()
}

/// Get a cached JSON string value.
/// 获取缓存的 JSON 字符串值。
pub async fn cache_get(cache_name: &str, key: &str) -> Option<String> {
    caches()
        .get(cache_name)
        .and_then(|c| c.get(key).map(|v| v.clone()))
}

/// Put a JSON string value into the cache.
/// 将 JSON 字符串值放入缓存。
pub async fn cache_put(cache_name: &str, key: String, value: String) {
    let entry = caches()
        .entry(cache_name.to_string())
        .or_insert_with(DashMap::new);
    entry.insert(key, value);
}

/// Evict a key from the named cache.
/// 从指定缓存中驱逐键。
pub async fn cache_evict_key(cache_name: &str, key: &str) {
    if let Some(cache) = caches().get(cache_name) {
        cache.remove(key);
    }
}
