//! `DataLoader` for N+1 query prevention / DataLoader用于防止N+1查询
//! Equivalent to Spring for GraphQL's `DataLoader` + `BatchLoaderRegistry`

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

#[async_trait]
pub trait BatchLoader<K, V>: Send + Sync
where
    K: Clone + Eq + Hash + Debug + Send + Sync,
    V: Clone + Send + Sync,
{
    async fn load(&self, keys: &[K]) -> HashMap<K, V>;
}

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
    pub fn new(loader: impl BatchLoader<K, V> + 'static) -> Self {
        Self {
            loader: Arc::new(loader),
            pending: Arc::new(Mutex::new(HashMap::new())),
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_batch_size: 100,
        }
    }

    pub fn max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    pub async fn load(&self, key: K) -> Option<V> {
        {
            let cache = self.cache.lock().expect("lock poisoned");
            if let Some(v) = cache.get(&key) {
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

    pub async fn load_many(&self, keys: Vec<K>) -> HashMap<K, Option<V>> {
        let mut receivers: Vec<(K, tokio::sync::oneshot::Receiver<V>)> = Vec::new();
        let mut result = HashMap::new();
        {
            let cache = self.cache.lock().expect("lock poisoned");
            for key in &keys {
                if let Some(v) = cache.get(key) {
                    result.insert(key.clone(), Some(v.clone()));
                }
            }
        }
        let mut pending = self.pending.lock().expect("lock poisoned");
        for key in keys {
            if result.contains_key(&key) { continue; }
            let (tx, rx) = tokio::sync::oneshot::channel();
            pending.entry(key.clone()).or_default().push(tx);
            receivers.push((key, rx));
        }
        drop(pending);
        self.dispatch().await;
        for (key, rx) in receivers {
            result.insert(key, rx.await.ok());
        }
        result
    }

    pub fn prime(&self, key: K, value: V) {
        self.cache.lock().expect("lock poisoned").insert(key, value);
    }

    pub fn prime_many(&self, entries: HashMap<K, V>) {
        self.cache.lock().expect("lock poisoned").extend(entries);
    }

    pub fn clear(&self) {
        self.cache.lock().expect("lock poisoned").clear();
    }

    pub async fn dispatch(&self) -> usize {
        let pending_keys: Vec<K> = {
            self.pending.lock().expect("lock poisoned").keys().cloned().collect()
        };
        if pending_keys.is_empty() { return 0; }
        let batch_keys: Vec<K> = pending_keys.into_iter().take(self.max_batch_size).collect();
        let loaded = self.loader.load(&batch_keys).await;
        let mut pending = self.pending.lock().expect("lock poisoned");
        let mut cache = self.cache.lock().expect("lock poisoned");
        let mut count = 0;
        for key in &batch_keys {
            if let Some(value) = loaded.get(key) {
                cache.insert(key.clone(), value.clone());
                count += 1;
                if let Some(senders) = pending.remove(key) {
                    for tx in senders { let _ = tx.send(value.clone()); }
                }
            }
        }
        count
    }

    pub async fn dispatch_all(&self) {
        loop {
            let n = self.dispatch().await;
            if n == 0 { break; }
        }
    }
}

#[derive(Default)]
pub struct DataLoaderRegistry {
    name: String,
}

impl DataLoaderRegistry {
    pub fn new(name: impl Into<String>) -> Self { Self { name: name.into() } }
    pub fn name(&self) -> &str { &self.name }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestLoader;
    #[async_trait]
    impl BatchLoader<u64, String> for TestLoader {
        async fn load(&self, keys: &[u64]) -> HashMap<u64, String> {
            keys.iter().map(|&k| (k, format!("user_{k}"))).collect()
        }
    }

    #[tokio::test]
    async fn test_load() {
        let loader = DataLoader::new(TestLoader);
        assert_eq!(loader.load(1).await, Some("user_1".into()));
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let loader = DataLoader::new(TestLoader);
        loader.prime(1, "cached".into());
        assert_eq!(loader.load(1).await, Some("cached".into()));
    }

    #[tokio::test]
    async fn test_load_many() {
        let loader = DataLoader::new(TestLoader);
        let results = loader.load_many(vec![1, 2, 3]).await;
        assert_eq!(results[&1], Some("user_1".into()));
    }

    #[tokio::test]
    async fn test_clear() {
        let loader = DataLoader::new(TestLoader);
        loader.prime(1, "cached".into());
        loader.clear();
        assert_eq!(loader.load(1).await, Some("user_1".into()));
    }
}
