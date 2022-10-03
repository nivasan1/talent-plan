use std::{error::Error, fmt};
use std::sync::Arc;
use parking_lot::Mutex;
/// type alias used for wrapping arbitrary error messages / returns in Result
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// embed dyn KvsEngine in an Arc, protected by a reference count and a Mutex,
// the impl KvsEngine will be stored on the heap, and de-allocated once Arc's
// reference count goes to zero
#[derive(Clone)]
pub struct SharedKvsEngine {
    engine: Arc<Mutex<dyn KvsEngine>>
}

impl SharedKvsEngine {
    /// instantiate a SharedKvsEngine as a locked. atomically reference counted pointer to
    /// the object on the heap
    pub fn from(engine:  impl KvsEngine) -> Self {
        SharedKvsEngine { 
            engine: Arc::from(Mutex::from(engine)),
        }
    }

    /// direct implementation of KvsEngine, as there cannot be cloned mutable refs between threads
    pub fn set(&self, key: String, val: String) -> Result<()> {
        // take lock
        let mut unlocked_engine = self.engine.lock();
        // return value from underlying KvsEngine
        unlocked_engine.set(key, val)
    }

    /// direct implementation of KvsEngine, as there cannot be cloned mutable refs between threads
    pub fn get(&self, key: String) -> Result<Option<String>> {
        // take lock
        let mut unlocked_engine = self.engine.lock();
        // return value from underlying KvsEngine
        unlocked_engine.get(key)    
    }

    /// direct implementation of KvsEngine, as there cannot be cloned mutable refs between threads
    pub fn remove(&self, key: String) -> Result<()> {
        // take lock
        let mut unlocked_engine = self.engine.lock();
        // return value from underlying KvsEngine
        unlocked_engine.remove(key)    
    }
}

/// this is the trait that both SledKvsEngine and KvStore implement, it is composed of
/// three methods
/// 1. set(&mut self, key: String, val: String) -> Result<()>
/// 2. get(&mut self, key: String) -> Result<Option<String>>
/// 3. remove(&mut self, key: String) -> Result<()>
pub trait KvsEngine: Send + 'static + Sync{
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
