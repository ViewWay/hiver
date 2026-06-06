//! `DataLoader` for N+1 query prevention / DataLoader用于防止N+1查询
//! Equivalent to Spring for GraphQL's `DataLoader` + `BatchLoaderRegistry`

#![allow(clippy::expect_used)]

use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

/// Trait for batch-loading values by a set of keys. / 按键集合批量加载值的 trait。
#[async_trait]
pub trait BatchLoader<K, V>: Send + Sync
where
    K: Clone + Eq + Hash + Debug + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Load values for the given keys, returning a map of found results. /
    /// 加载给定键对应的值，返回找到的结果映射。
    async fn load(&self, keys: &[K]) -> HashMap<K, V>;
}

/// Batching data loader to prevent N+1 queries. / 批量数据加载器，防止 N+1 查询。
pub struct DataLoader<K, V>
where
    K: Clone + Eq + Hash + Debug + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    loader: Arc<dyn BatchLoader<K, V>>,
    pending: Arc<Mutex<HashMap<K, Vec<tokio::sync::oneshot::Sender<V>>>>>,
    cache: Arc<Mutex<HashMap<K, V>>>,
    max_batch_size: usize,
}

impl<K, V> DataLoader<K, V>
where
    K: Clone + Eq + Hash + Debug + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new DataLoader with the given batch loader. / 使用给定的批量加载器创建 DataLoader。
    pub fn new(loader: impl BatchLoader<K, V> + 'static) -> Self
    {
        Self {
            loader: Arc::new(loader),
            pending: Arc::new(Mutex::new(HashMap::new())),
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_batch_size: 100,
        }
    }

    /// Set the maximum number of keys per batch. / 设置每批最大键数。
    pub fn max_batch_size(mut self, size: usize) -> Self
    {
        self.max_batch_size = size;
        self
    }

    /// Load a single value by key (uses cache and batching). / 按键加载单个值（使用缓存和批处理）。
    pub async fn load(&self, key: K) -> Option<V>
    {
        {
            let cache = self.cache.lock().expect("lock poisoned");
            if let Some(v) = cache.get(&key)
            {
                return Some(v.clone());
            }
        }
        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut pending = self.pending.lock().expect("lock poisoned");
            pending.entry(key.clone()).or_default().push(tx);
        }
        self.dispatch().await;
        rx.await.ok()
    }

    /// Load multiple values by keys. / 按多个键加载值。
    pub async fn load_many(&self, keys: Vec<K>) -> HashMap<K, Option<V>>
    {
        let mut receivers: Vec<(K, tokio::sync::oneshot::Receiver<V>)> = Vec::new();
        let mut result = HashMap::new();
        {
            let cache = self.cache.lock().expect("lock poisoned");
            for key in &keys
            {
                if let Some(v) = cache.get(key)
                {
                    result.insert(key.clone(), Some(v.clone()));
                }
            }
        }
        {
            let mut pending = self.pending.lock().expect("lock poisoned");
            for key in keys
            {
                if result.contains_key(&key)
                {
                    continue;
                }
                let (tx, rx) = tokio::sync::oneshot::channel();
                pending.entry(key.clone()).or_default().push(tx);
                receivers.push((key, rx));
            }
        }
        self.dispatch().await;
        for (key, rx) in receivers
        {
            result.insert(key, rx.await.ok());
        }
        result
    }

    /// Prime the cache with a single key-value pair. / 向缓存中预置单个键值对。
    pub fn prime(&self, key: K, value: V)
    {
        self.cache.lock().expect("lock poisoned").insert(key, value);
    }

    /// Prime the cache with multiple entries. / 向缓存中预置多个条目。
    pub fn prime_many(&self, entries: HashMap<K, V>)
    {
        self.cache.lock().expect("lock poisoned").extend(entries);
    }

    /// Clear the value cache. / 清除值缓存。
    pub fn clear(&self)
    {
        self.cache.lock().expect("lock poisoned").clear();
    }

    /// Dispatch pending keys to the batch loader. / 将待处理键分发给批量加载器。
    pub async fn dispatch(&self) -> usize
    {
        let pending_keys: Vec<K> = {
            self.pending
                .lock()
                .expect("lock poisoned")
                .keys()
                .cloned()
                .collect()
        };
        if pending_keys.is_empty()
        {
            return 0;
        }
        let batch_keys: Vec<K> = pending_keys.into_iter().take(self.max_batch_size).collect();
        let loaded = self.loader.load(&batch_keys).await;
        let mut pending = self.pending.lock().expect("lock poisoned");
        let mut cache = self.cache.lock().expect("lock poisoned");
        let mut count = 0;
        for key in &batch_keys
        {
            if let Some(value) = loaded.get(key)
            {
                cache.insert(key.clone(), value.clone());
                count += 1;
                if let Some(senders) = pending.remove(key)
                {
                    for tx in senders
                    {
                        let _ = tx.send(value.clone());
                    }
                }
            }
        }
        count
    }

    /// Dispatch all pending batches until none remain. / 分发所有待处理批次直到清空。
    pub async fn dispatch_all(&self)
    {
        loop
        {
            let n = self.dispatch().await;
            if n == 0
            {
                break;
            }
        }
    }
}

/// Registry for named DataLoader instances. / 命名 DataLoader 实例的注册表。
#[derive(Default)]
pub struct DataLoaderRegistry
{
    name: String,
}

impl DataLoaderRegistry
{
    /// Create a new registry with the given name. / 使用给定名称创建新注册表。
    pub fn new(name: impl Into<String>) -> Self
    {
        Self { name: name.into() }
    }

    /// Return the registry name. / 返回注册表名称。
    pub fn name(&self) -> &str
    {
        &self.name
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    struct TestLoader;
    #[async_trait]
    impl BatchLoader<u64, String> for TestLoader
    {
        async fn load(&self, keys: &[u64]) -> HashMap<u64, String>
        {
            keys.iter().map(|&k| (k, format!("user_{k}"))).collect()
        }
    }

    #[tokio::test]
    async fn test_load()
    {
        let loader = DataLoader::new(TestLoader);
        assert_eq!(loader.load(1).await, Some("user_1".into()));
    }

    #[tokio::test]
    async fn test_cache_hit()
    {
        let loader = DataLoader::new(TestLoader);
        loader.prime(1, "cached".into());
        assert_eq!(loader.load(1).await, Some("cached".into()));
    }

    #[tokio::test]
    async fn test_load_many()
    {
        let loader = DataLoader::new(TestLoader);
        let results = loader.load_many(vec![1, 2, 3]).await;
        assert_eq!(results[&1], Some("user_1".into()));
    }

    #[tokio::test]
    async fn test_clear()
    {
        let loader = DataLoader::new(TestLoader);
        loader.prime(1, "cached".into());
        loader.clear();
        assert_eq!(loader.load(1).await, Some("user_1".into()));
    }
}
