use std::{collections::HashMap, path::PathBuf};
use failure::Error;
/// Example
/// ```rust
/// use kvs::kvs::new;
/// # fn main() {
/// # let mut store = new();
/// # store.set("hello".to_string(), "world".to_string());
/// # assert_eq!(store.get("hello".to_owned()), Some("world".to_string()));
/// # store.remove("hello".to_string());
/// # assert_eq!(store.get("hello".to_string()), None);
/// # }
/// ```
/// KvStore object contains a HashMap taking Keys to Values
/// The KvStore implements the following methods
/// fn set(&mut self, key: String, value: String)
/// fn get(&mut self, key: String) -> Option<String>
/// fn rm(&mut self, key: String)
/// This is an in-memory kv-store, it does not persist state to disk
pub struct KvStore {
}

pub type Result<T> = std::result::Result<T, Error>;

/// Instantiate a new empty kv_store object
/// along with a fresh HashMap<String, String>
pub fn new() -> KvStore {
    KvStore {
    }
}

/// Instantiate a KvStore through opening a file, with the 
/// with the given path passed as argument
pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
    
}

impl KvStore {
    // type alias for Result types used
    /// Inserts a (key, value) pair into map
    /// overwrites existing value is key exists
    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        todo!()
    }

    /// Gets a value associated with the key in KvStore.map
    /// returns None if the key does not exist
    /// clones the string from the map if it exists
    pub fn get(&self, key: String) -> Result<()> {
        todo!()
    }

    /// Remves the value associated with the key in KvStore.map
    /// if the key has no value, this is a no-op
    pub fn remove(&mut self, key: String) -> Result<()> {
        todo!()
    }
}
