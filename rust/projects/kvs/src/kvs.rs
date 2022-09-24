use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Write;
use std::{collections::HashMap, path::PathBuf};
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
    // map containing sha256(command, key, value?) -> file_offset
    map: HashMap<String, u64>,
    // file to be used during sets, gets, rm
    file: String,
}

/// CommandData is an enum representing the data that will ultimately
/// be serialized and written to the logfile, the enum contains
/// (rm, key, value)
/// (set, key, value)
/// (get, key, value)
#[derive(Deserialize, Serialize, Debug)]
enum CommandData {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String, value: String },
}

pub type Result<T> = std::result::Result<T, dyn Error>;

impl KvStore {
    /// Instantiate a KvStore through opening a file, with the
    /// with the given path passed as argument
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        todo!()
    }

    /// Inserts a (key, value) pair into map
    /// serialized set, key, value
    /// overwrites existing value is key exists
    /// The user invokes kvs set mykey myvalue
    /// kvs creates a value representing the "set" command, containing its key and value
    /// It then serializes that command to a String
    /// It then appends the serialized command to a file containing the log
    /// If that succeeds, it exits silently with error code 0
    ///If it fails, it exits by printing the error and returning a non-zero error code
    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        // open file 
        match File::options()
            // with write permissions
            .write(true)
            .open(&self.file)
            // return err or mutate file
            .map(|mut writer| {
                // file is opened, attempt to serialize the command data
                serde_json::to_string(&CommandData::Set{
                    key: key,
                    value: val,
                }).map(|data_ser| data_ser.as_bytes().to_owned()).map(|data_bytes| {
                    writer.write(&data_bytes)
                })
            }) {
                Ok(_) => Ok(()),
                Err(err) => Err(err)
            }
    }

    /// Gets a value associated with the key in KvStore.map
    /// returns None if the key does not exist
    /// clones the string from the map if it exists
    pub fn get(&self, key: String) -> Result<Option<String>> {
        todo!()
    }

    /// Remves the value associated with the key in KvStore.map
    /// if the key has no value, this is a no-op
    pub fn remove(&mut self, key: String) -> Result<()> {
        todo!()
    }
}
