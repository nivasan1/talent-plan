use std::{error::Error, fmt};
/// type alias used for wrapping arbitrary error messages / returns in Result
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// this is the trait that both SledKvsEngine and KvStore implement, it is composed of
/// three methods
/// 1. set(&mut self, key: String, val: String) -> Result<()>
/// 2. get(&mut self, key: String) -> Result<Option<String>>
/// 3. remove(&mut self, key: String) -> Result<()>
pub trait KvsEngine {
    /// Inserts a (key, value) pair into map
    /// serialized set, key, value
    /// overwrites existing value is key exists
    fn set(&mut self, key: String, val: String) -> Result<()>;

    /// Gets a value associated with the key in KvStore.map
    /// returns None if the key does not exist
    /// clones the string from the map if it exists
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// Remves the value associated with the key in KvStore.map
    /// if the key has no value, this is a no-op
    fn remove(&mut self, key: String) -> Result<()>;
}

/// Key not found error returned from both kvs_engines
/// Error returned when the user attempts to remove a non-existent key
#[derive(Debug, Clone)]
pub struct ErrKeyNotFound {
    pub key: String,
}

impl fmt::Display for ErrKeyNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "key not found: {}", self.key) // user-facing output
    }
}

impl Error for ErrKeyNotFound {}
