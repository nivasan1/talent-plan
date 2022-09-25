use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
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
    map: HashMap<String, String>,
    // file to be used during sets, gets, rm
    file: PathBuf,
    // the log has been modified since last read
    dirty: bool,
}

#[derive(Debug, Clone)]
pub struct ErrNotFound {
    key: String,
}

impl fmt::Display for ErrNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "key not found: {}", self.key) // user-facing output
    }
}

impl Error for ErrNotFound {}

/// CommandData is an enum representing the data that will ultimately
/// be serialized and written to the logfile, the enum contains
/// (rm, key, value)
/// (set, key, value)
/// (get, key, value)
#[derive(Deserialize, Serialize, Debug)]
enum CommandData {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl KvStore {
    /// Instantiate a KvStore through opening a file, with the
    /// with the given path passed as argument
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        // return a KvStore at the path provided
        Ok(KvStore {
            map: HashMap::new(),
            file: path.into(),
            dirty: true,
        })
    }

    /// Inserts a (key, value) pair into map
    /// serialized set, key, value
    /// overwrites existing value is key exists
    /// The user invokes kvs set mykey myvalue
    /// kvs creates a value representing the "set" command, containing its key and value
    /// It then serializes that command to a String
    /// It then appends the serialized command to a file containing the log
    /// If that succeeds, it exits silently with error code 0
    /// If it fails, it exits by printing the error and returning a non-zero error code
    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        self.write_log(CommandData::Set {
            key: key,
            value: val,
        })
    }

    /// Gets a value associated with the key in KvStore.map
    /// returns None if the key does not exist
    /// clones the string from the map if it exists
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        // read the logs
        self.read_log()?;
        // get value from map, return ErrNotFound if the key DNE
        let val = self.map.get(&key);
        if let None = val {
            // return the error if the key is not found
            return Err(Into::<Box<dyn Error>>::into(ErrNotFound { key: key }));
        }
        // key is found, write to log
        self.write_log(CommandData::Get { key: key })
            .and_then(|_| Ok(Some(val.unwrap().to_owned())))
    }

    /// Remves the value associated with the key in KvStore.map
    /// if the key has no value, this is a no-op
    // The user invokes kvs rm mykey
    // Same as the "get" command, kvs reads the entire log to build the in-memory index
    // It then checks the map if the given key exists
    // If the key does not exist, it prints "Key not found", and exits with a non-zero error code
    // If it succeeds
    // It creates a value representing the "rm" command, containing its key
    // It then appends the serialized command to the log
    // If that succeeds, it exits silently with error code 0
    pub fn remove(&mut self, key: String) -> Result<()> {
        // update hashmap from log
        self.read_log()?;
        // remove value from hashmap
        if let None = self.map.remove(&key) {
            // return error if the key is not found
            return Err(Into::<Box<dyn Error>>::into(ErrNotFound { key: key }));
        }
        // write command to log
        self.write_log(CommandData::Rm { key: key })
    }

    /// read_log reads the current log file, and updates the key to log pointer indices
    /// this is only called when the state is dirty, i.e, the cache does not reflect the
    /// log
    fn read_log(&mut self) -> Result<()> {
        // skip this step if the log is not dirty
        if !self.dirty {
            return Ok(());
        }
        // create buffer to hold file contents
        let mut vec = Vec::<u8>::new();
        // open file
        File::open(&self.file)
            // Box err if it exists
            .map_err(|err| Into::<Box<dyn Error>>::into(err))?
            // read log contents to buffer, return Boxed error if needed
            .read_to_end(&mut vec)?;
        // now data from buffer and return log pointer of most recent recording
        let (mut begin, mut end) = (0, 0);
        vec.iter()
            .map(|byte| {
                end += 1;
                // iterate through
                if *byte == b'\n' {
                    // unmarshal data between &vec[begin..end]
                    let cmd: CommandData = serde_json::from_slice(&vec[begin..end - 1])?;
                    match cmd {
                        // update key in get
                        CommandData::Set { key, value: val } => {
                            self.map.insert(key, val);
                        }
                        // remove key from map in Rm
                        CommandData::Rm { key, .. } => {
                            self.map.remove(&key);
                        }
                        _ => (),
                    }
                    // update begin to end +1
                    begin = &end + 1;
                }
                // state is not dirty any more
                self.dirty = true;
                Ok(())
            })
            .collect()
    }

    /// write log appends the given log entry to the logfile, determined by command type
    fn write_log(&self, data: CommandData) -> Result<()> {
        File::options()
            .write(true)
            .append(true)
            .open(&self.file)
            // if opening the file resulted in an error, Box it
            .map_err(|err| Into::<Box<dyn Error>>::into(err))
            // file exists, now write the serialized data to it
            .and_then(|mut file| {
                // ok the file is opened, lets first serialize CommandData::Set
                serde_json::to_string(&data)
                    // in case of error map err to Box<dyn Error>
                    .map_err(|err| Box::from(err))
                    // write the serialized data to file
                    .and_then(|serial| writeln!(file, "{}", serial).map_err(|err| Box::from(err)))
            })
            // this method returns Ok(())
            .and_then(|_| Ok(()))
    }
}
