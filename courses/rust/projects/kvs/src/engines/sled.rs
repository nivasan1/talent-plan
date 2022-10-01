use std::path::PathBuf;

use crate::engines::kvs_engine::{ErrKeyNotFound, KvsEngine, Result};
use sled::{Config, Db};
use std::error::Error;
use std::path::Path;
/// SledKvsEngine is a wrapper around a Sled embedded database for observing reads / writes
pub struct SledKvsEngine {
    // sled DB located in dir,
    Db: Db,
}

/// this method contains the methods for opening and returning a SledKvsEngine
impl SledKvsEngine {
    /// open a Db file at the specified path, and then return a newly instantiated SledKvsEngine
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        // open db at address
        let db = Config::new().path(path.as_ref()).open()?;
        // return db
        Ok(SledKvsEngine { Db: db })
    }
}

// implementation of KvsEngine for SledKvsEngine
impl KvsEngine for SledKvsEngine {
    /// passes a get method to the underlying Sled Db
    fn get(&mut self, key: String) -> Result<Option<String>> {
        let res = self.Db.get(&key)?;
        if let Some(vec) = res {
            // this is an ivec, convert to a string, and return the underlying value
            let data = String::from_utf8(vec.to_vec())?;
            // return the string if ok
            return Ok(Some(data));
        }
        Ok(None)
    }

    /// set a value to the underlying SledKvsEngine
    fn set(&mut self, key: String, val: String) -> Result<()> {
        // set key, value pair in the SledKvsEngine
        self.Db.insert(key.as_bytes(), val.as_bytes())?;
        // ignore last value if it was set
        Ok(())
    }

    /// remove a value from the underlying SledKvsEngine
    fn remove(&mut self, key: String) -> Result<()> {
        // remove key from the Db, return error and ignore result
        if let None = self.Db.remove(key.as_bytes())? {
            // return error if the key is not found
            return Err(Into::<Box<dyn Error>>::into(ErrKeyNotFound { key }));
        }
        Ok(())
    }
}
