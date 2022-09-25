use serde::{Deserialize, Serialize};
use serde_json;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
/// Example
/// ```rust
/// use kvs::kvs::KvStore;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut store = KvStore::open("./")?;
/// # store.set("hello".to_string(), "world".to_string());
/// # assert_eq!(store.get("hello".to_owned())?, Some("world".to_string()));
/// # store.remove("hello".to_string());
/// # assert_eq!(store.get("hello".to_string())?, None);
/// # Ok(())
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
    // set log pointers
    log_pointers: HashMap<String, Bound>,
    // number of actions made on log
    actions: u64,
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Bound {
    begin: usize,
    end: usize,
}

/// Total Order over (usize, usize), used to prepare buffer for
/// draining
impl Ord for Bound {
    fn cmp(&self, other: &Self) -> Ordering {
        self.begin.cmp(&other.begin)
    }
}
/// PartialOrd used to sort
impl PartialOrd for Bound {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// maximum number of actions needed before log compaction
const COMPACTION_SIZE: u64 = 10000;

#[derive(Debug, Clone)]
/// Error returned when the user attempts to remove a non-existent key
pub struct ErrKeyNotFound {
    key: String,
}

impl fmt::Display for ErrKeyNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "key not found: {}", self.key) // user-facing output
    }
}

impl Error for ErrKeyNotFound {}

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

/// type alias used for wrapping arbitrary error messages / returns in Result
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl KvStore {
    /// Instantiate a KvStore through opening a file, with the
    /// with the given path passed as argument
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        // create log file, in given dir
        let log_path = Path::new(&path.into()).join("log");
        // open file with given path, (write permissions must be given if creating file)
        File::options().create(true).write(true).open(&log_path)?;
        // return a KvStore at the path provided
        Ok(KvStore {
            map: HashMap::new(),
            file: log_path,
            dirty: true,
            actions: 0,
            log_pointers: HashMap::new(),
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
        self.write_log(CommandData::Set { key, value: val })
            .map(|_| {
                // update actions after write is successful
                self.dirty = true;
                ()
            })
    }

    /// Gets a value associated with the key in KvStore.map
    /// returns None if the key does not exist
    /// clones the string from the map if it exists
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        // read the logs
        self.read_log()?;
        // get value from map, return ErrKeyNotFound if the key DNE,
        let val = self.map.get(&key).map(|x| x.to_owned());
        if let None = val {
            // return the error if the key is not found
            return Ok(None);
        }
        // key is found, write to log
        self.write_log(CommandData::Get { key }).map(|_| {
            // can panic here as we have exhausted earlier check
            Some(val.unwrap())
        })
    }

    /// Remves the value associated with the key in KvStore.map
    /// if the key has no value, this is a no-op
    /// The cached (key, value) pairs will not reflect log state after this routine
    /// As such, future reads will read directly from log, and will cache results
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
            return Err(Into::<Box<dyn Error>>::into(ErrKeyNotFound { key }));
        }
        // write command to log
        self.write_log(CommandData::Rm { key }).and_then(|_| {
            self.dirty = true;
            Ok(())
        })
    }

    /// read_log reads the current log file, and updates the key to log pointer indices
    /// this is only called when the state is dirty, i.e, the cache does not reflect the
    /// log
    /// The dirtiness of the state is set to false after this read
    /// #Errors
    /// OS errors resulting from File opening /closing
    /// File must exist
    fn read_log(&mut self) -> Result<()> {
        // skip this step if the log is not dirty
        if !self.dirty {
            return Ok(());
        }
        // create buffer to hold file contents
        let mut vec = Vec::<u8>::new();
        // open file
        File::options()
            .read(true)
            .open(&self.file)
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
                        // update key from set
                        CommandData::Set { key, value: val } => {
                            // set cached state
                            self.map.insert(key.clone(), val);
                            // write to log_pointers for result
                            self.log_pointers.insert(
                                key.clone(),
                                Bound {
                                    begin,
                                    end: end - 1,
                                },
                            );
                        }
                        // remove key from map in Rm
                        CommandData::Rm { key, .. } => {
                            self.map.remove(&key);
                            // remove key from log_pointers
                            self.log_pointers.remove(&key);
                        }
                        // reads do not affect state
                        _ => (),
                    }
                    // update begin to end +1
                    begin = end;
                }
                // state is not dirty any more
                self.dirty = false;
                Ok(())
            })
            .collect()
    }

    /// compact, updates the log file, to only contain gets / sets from previous state
    /// This form of compaction, retains the latest state for reads / writes
    fn compact_log(&mut self) -> Result<()> {
        // only compact state once the log has reached comaption size
        if self.actions < COMPACTION_SIZE {
            return Ok(());
        }
        // initialize temporary buffer to make writes to
        let mut buf = Vec::<u8>::new();
        // if state is dirty, clean it
        if self.dirty {
            self.read_log()?;
        }
        // most updated state is cached, iterate over it and
        // write the serialized data to buffer
        File::options()
            .read(true)
            .open(&self.file)
            // Box err if it exists
            .map_err(|err| Into::<Box<dyn Error>>::into(err))?
            // read log contents to buffer, return Boxed error if needed
            .read_to_end(&mut buf)?;
        // data is read into buf, drain un-needed elements
        let (mut begin, mut end, mut drain_size) = (0, 0, 0);
        // sort the log_pointers so we are draining contiguous sections of un-needed space from vec
        // collect values of bound into vec
        let mut bounds = self.log_pointers.values().cloned().collect::<Vec<Bound>>();
        bounds.sort();
        // drain un-needed elements from buf
        for bound in bounds.iter() {
            // can remove new-line
            end = bound.begin - drain_size;
            // drain contents including ending new-line of erased entry
            buf.drain(begin..end);
            // adjust indices to the drained vec
            drain_size += end - begin;
            // keep trailing newline
            begin = bound.end + 1 - drain_size;
        }
        // finally drain from end if needed
        buf.drain(begin..buf.len());
        // finally, write buf
        // buf is drained of the un-needed sections, truncate original contents of file, and
        // write new buffer
        File::options()
            .write(true)
            .truncate(true)
            .open(&self.file)
            // return file Opening err if it exists
            .map_err(Into::<Box<dyn Error>>::into)?
            .write(&buf)
            .map_err(Into::<Box<dyn Error>>::into)?;
        Ok(())
    }

    /// write log appends the given log entry to the logfile, determined by command type
    /// #Errors
    ///    Resulting from OS / Serialization of CommandData
    /// After a successful write to log, the log is compacted to reduce Filesystem overhead
    fn write_log(&mut self, data: CommandData) -> Result<()> {
        File::options()
            .write(true)
            .append(true)
            .open(&self.file)
            // if opening the file resulted in an error, Box it
            .map_err(Into::<Box<dyn Error>>::into)
            // file exists, now write the serialized data to it
            .and_then(|mut file| {
                // ok the file is opened, lets first serialize CommandData::Set
                // update the number of actions taken
                self.actions = file.metadata()?.len();
                serde_json::to_string(&data)
                    // in case of error map err to Box<dyn Error>
                    .map_err(|err| Box::from(err))
                    // write the serialized data to file
                    .and_then(|serial| writeln!(file, "{}", serial).map_err(Box::from))
            })
            // this method returns Ok(())
            .map(|_| ())?;
        // compact log
        self.compact_log()
    }
}
