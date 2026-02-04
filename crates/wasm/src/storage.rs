//! Storage abstraction for WASM builds.
//!
//! In browser environments, we use IndexedDB for persistent storage instead
//! of the filesystem. This module provides a trait-based abstraction that
//! allows the same code to work with different storage backends.

use {
    anyhow::Result,
    async_trait::async_trait,
    serde::{Deserialize, Serialize},
};

/// A key-value storage backend.
///
/// This trait abstracts over different storage implementations:
/// - Native: SQLite database or filesystem
/// - WASM: IndexedDB or localStorage
#[async_trait(?Send)]
pub trait Storage: Send + Sync {
    /// Get a value by key.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Set a value by key.
    async fn set(&self, key: &str, value: &[u8]) -> Result<()>;

    /// Delete a value by key.
    async fn delete(&self, key: &str) -> Result<()>;

    /// List all keys with a given prefix.
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;

    /// Clear all data.
    async fn clear(&self) -> Result<()>;
}

/// In-memory storage for testing and ephemeral sessions.
#[derive(Default)]
pub struct MemoryStorage {
    data: std::sync::RwLock<std::collections::HashMap<String, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait(?Send)]
impl Storage for MemoryStorage {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data = self
            .data
            .read()
            .map_err(|e| anyhow::anyhow!("lock error: {e}"))?;
        Ok(data.get(key).cloned())
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|e| anyhow::anyhow!("lock error: {e}"))?;
        data.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|e| anyhow::anyhow!("lock error: {e}"))?;
        data.remove(key);
        Ok(())
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let data = self
            .data
            .read()
            .map_err(|e| anyhow::anyhow!("lock error: {e}"))?;
        Ok(data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }

    async fn clear(&self) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|e| anyhow::anyhow!("lock error: {e}"))?;
        data.clear();
        Ok(())
    }
}

/// A typed wrapper around Storage for serializable values.
pub struct TypedStorage<S: Storage> {
    storage: S,
    prefix: String,
}

impl<S: Storage> TypedStorage<S> {
    pub fn new(storage: S, prefix: impl Into<String>) -> Self {
        Self {
            storage,
            prefix: prefix.into(),
        }
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let prefixed = self.prefixed_key(key);
        match self.storage.get(&prefixed).await? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)?;
                Ok(Some(value))
            },
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let prefixed = self.prefixed_key(key);
        let bytes = serde_json::to_vec(value)?;
        self.storage.set(&prefixed, &bytes).await
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let prefixed = self.prefixed_key(key);
        self.storage.delete(&prefixed).await
    }

    pub async fn list_keys(&self) -> Result<Vec<String>> {
        let prefix = format!("{}:", self.prefix);
        let keys = self.storage.list_keys(&prefix).await?;
        Ok(keys
            .into_iter()
            .map(|k| k.strip_prefix(&prefix).unwrap_or(&k).to_string())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use {super::*, futures::executor::block_on};

    #[test]
    fn test_memory_storage() {
        block_on(async {
            let storage = MemoryStorage::new();

            // Test set and get
            storage.set("key1", b"value1").await.unwrap();
            let value = storage.get("key1").await.unwrap();
            assert_eq!(value, Some(b"value1".to_vec()));

            // Test non-existent key
            let value = storage.get("nonexistent").await.unwrap();
            assert!(value.is_none());

            // Test delete
            storage.delete("key1").await.unwrap();
            let value = storage.get("key1").await.unwrap();
            assert!(value.is_none());

            // Test list_keys
            storage.set("prefix:a", b"1").await.unwrap();
            storage.set("prefix:b", b"2").await.unwrap();
            storage.set("other:c", b"3").await.unwrap();

            let keys = storage.list_keys("prefix:").await.unwrap();
            assert_eq!(keys.len(), 2);
            assert!(keys.contains(&"prefix:a".to_string()));
            assert!(keys.contains(&"prefix:b".to_string()));

            // Test clear
            storage.clear().await.unwrap();
            let keys = storage.list_keys("").await.unwrap();
            assert!(keys.is_empty());
        });
    }

    #[test]
    fn test_typed_storage() {
        block_on(async {
            #[derive(Debug, Serialize, Deserialize, PartialEq)]
            struct TestData {
                name: String,
                value: i32,
            }

            let storage = MemoryStorage::new();
            let typed: TypedStorage<MemoryStorage> = TypedStorage::new(storage, "test");

            let data = TestData {
                name: "example".to_string(),
                value: 42,
            };

            typed.set("item1", &data).await.unwrap();
            let retrieved: Option<TestData> = typed.get("item1").await.unwrap();
            assert_eq!(retrieved, Some(data));

            typed.delete("item1").await.unwrap();
            let retrieved: Option<TestData> = typed.get("item1").await.unwrap();
            assert!(retrieved.is_none());
        });
    }
}
