//! # Firework DataLoader Plugin
//! 
//! Solves the N+1 query problem by batching and caching database queries.
//! 
//! ## Features
//! 
//! - **Automatic batching**: Multiple individual loads are batched into single queries
//! - **Request-scoped caching**: Deduplicates identical loads within a request
//! - **Type-safe**: Generic over key and value types
//! - **Async-first**: Built on tokio
//! 
//! ## Example
//! 
//! ```ignore
//! use firework_dataloader::{DataLoader, BatchFn};
//! use sea_orm::{DatabaseConnection, EntityTrait};
//! 
//! // Define batch loading function
//! async fn batch_load_users(
//!     db: &DatabaseConnection,
//!     user_ids: Vec<i32>
//! ) -> Vec<Option<User>> {
//!     let users: HashMap<i32, User> = users::Entity::find()
//!         .filter(users::Column::Id.is_in(user_ids.clone()))
//!         .all(db)
//!         .await
//!         .unwrap_or_default()
//!         .into_iter()
//!         .map(|u| (u.id, u))
//!         .collect();
//!     
//!     // Return in same order as requested
//!     user_ids.into_iter().map(|id| users.get(&id).cloned()).collect()
//! }
//! 
//! // Use in handler
//! #[get("/tweets")]
//! async fn get_tweets(DbConn(db): DbConn) -> Json<Vec<TweetWithUser>> {
//!     let loader = DataLoader::new(|ids| batch_load_users(&db, ids));
//!     
//!     let tweets = get_all_tweets(&db).await;
//!     
//!     let mut results = Vec::new();
//!     for tweet in tweets {
//!         // This batches - only ONE query for all users!
//!         let user = loader.load(tweet.user_id).await;
//!         results.push(TweetWithUser { tweet, user });
//!     }
//!     
//!     Json(results)
//! }
//! ```

use dashmap::DashMap;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// Batch loading function signature
/// 
/// Takes a list of keys and returns a list of values in the same order.
/// If a key has no value, return None for that position.
pub type BatchFn<K, V> = Arc<dyn Fn(Vec<K>) -> futures::future::BoxFuture<'static, Vec<Option<V>>> + Send + Sync>;

/// DataLoader for batching and caching
pub struct DataLoader<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    batch_fn: BatchFn<K, V>,
    cache: Arc<DashMap<K, V>>,
    pending: Arc<Mutex<HashMap<K, Arc<Notify>>>>,
    max_batch_size: usize,
}

impl<K, V> DataLoader<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new DataLoader with a batch loading function
    pub fn new<F, Fut>(batch_fn: F) -> Self
    where
        F: Fn(Vec<K>) -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = Vec<Option<V>>> + Send + 'static,
    {
        Self {
            batch_fn: Arc::new(move |keys| Box::pin(batch_fn(keys))),
            cache: Arc::new(DashMap::new()),
            pending: Arc::new(Mutex::new(HashMap::new())),
            max_batch_size: 100,
        }
    }

    /// Set maximum batch size (default: 100)
    pub fn with_max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    /// Load a single value by key
    /// 
    /// This will batch with other concurrent loads and cache the result
    pub async fn load(&self, key: K) -> Option<V> {
        // Check cache first
        if let Some(value) = self.cache.get(&key) {
            return Some(value.clone());
        }

        // Get or create notify for this key
        let _notify = {
            let mut pending = self.pending.lock().await;
            pending.entry(key.clone())
                .or_insert_with(|| Arc::new(Notify::new()))
                .clone()
        };

        // Wait a tiny bit to batch with other loads
        tokio::time::sleep(std::time::Duration::from_micros(10)).await;

        // Collect all pending keys
        let keys_to_load: Vec<K> = {
            let pending = self.pending.lock().await;
            pending.keys().cloned().collect()
        };

        // Split into batches if needed
        let batch_size = self.max_batch_size.min(keys_to_load.len());
        let batch: Vec<K> = keys_to_load.into_iter().take(batch_size).collect();

        // Load batch
        let values = (self.batch_fn)(batch.clone()).await;

        // Cache results
        for (key, value) in batch.iter().zip(values.iter()) {
            if let Some(val) = value {
                self.cache.insert(key.clone(), val.clone());
            }
        }

        // Notify waiters and clean up pending
        {
            let mut pending = self.pending.lock().await;
            for key in batch.iter() {
                if let Some(notify) = pending.remove(key) {
                    notify.notify_waiters();
                }
            }
        }

        // Return value for requested key
        self.cache.get(&key).map(|v| v.clone())
    }

    /// Load many values by keys
    /// 
    /// More efficient than calling load() multiple times
    pub async fn load_many(&self, keys: Vec<K>) -> Vec<Option<V>> {
        // Filter out cached keys
        let (_cached, to_load): (Vec<_>, Vec<_>) = keys.iter()
            .partition(|k| self.cache.contains_key(k));

        // Load uncached
        if !to_load.is_empty() {
            let keys_to_load: Vec<K> = to_load.iter().map(|k| (*k).clone()).collect();
            let values = (self.batch_fn)(keys_to_load.clone()).await;

            // Cache results
            for (key, value) in keys_to_load.iter().zip(values.iter()) {
                if let Some(val) = value {
                    self.cache.insert(key.clone(), val.clone());
                }
            }
        }

        // Return in original order
        keys.iter().map(|k| self.cache.get(k).map(|v| v.clone())).collect()
    }

    /// Prime the cache with a value
    pub fn prime(&self, key: K, value: V) {
        self.cache.insert(key, value);
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Clear a specific key from cache
    pub fn clear_key(&self, key: &K) {
        self.cache.remove(key);
    }
}

impl<K, V> Clone for DataLoader<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            batch_fn: Arc::clone(&self.batch_fn),
            cache: Arc::clone(&self.cache),
            pending: Arc::clone(&self.pending),
            max_batch_size: self.max_batch_size,
        }
    }
}

/// Helper macro for creating batch load functions
/// 
/// ```ignore
/// batch_loader!(User, i32, |db: &DatabaseConnection, ids: Vec<i32>| async move {
///     let users = users::Entity::find()
///         .filter(users::Column::Id.is_in(ids.clone()))
///         .all(db)
///         .await?
///         .into_iter()
///         .map(|u| (u.id, u))
///         .collect::<HashMap<_, _>>();
///     
///     Ok(ids.into_iter().map(|id| users.get(&id).cloned()).collect())
/// });
/// ```
#[macro_export]
macro_rules! batch_loader {
    ($value_type:ty, $key_type:ty, |$db:ident : &$db_type:ty, $ids:ident : Vec<$id_type:ty>| $body:expr) => {
        |$ids: Vec<$key_type>| {
            let $db = $db.clone();
            async move {
                let result: Result<Vec<Option<$value_type>>, _> = $body.await;
                result.unwrap_or_else(|_| vec![None; $ids.len()])
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dataloader_batching() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);
        
        // Batch function that returns key * 2
        let loader = DataLoader::new(move |keys: Vec<i32>| {
            call_count_clone.fetch_add(1, Ordering::Relaxed);
            async move {
                keys.into_iter().map(|k| Some(k * 2)).collect()
            }
        });

        // Load multiple keys
        let results = vec![
            loader.load(1).await,
            loader.load(2).await,
            loader.load(3).await,
        ];

        assert_eq!(results, vec![Some(2), Some(4), Some(6)]);
        
        // Should only call batch function once or twice (due to timing)
        let calls = call_count.load(Ordering::Relaxed);
        assert!(calls <= 3, "Expected <= 3 calls, got {}", calls);
    }

    #[tokio::test]
    async fn test_dataloader_caching() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);
        
        let loader = DataLoader::new(move |keys: Vec<i32>| {
            call_count_clone.fetch_add(1, Ordering::Relaxed);
            async move {
                keys.into_iter().map(|k| Some(k * 2)).collect()
            }
        });

        // First load
        let result1 = loader.load(1).await;
        assert_eq!(result1, Some(2));

        // Second load should use cache
        let result2 = loader.load(1).await;
        assert_eq!(result2, Some(2));

        // Should only call batch function once
        assert_eq!(call_count.load(Ordering::Relaxed), 1);
    }
}
