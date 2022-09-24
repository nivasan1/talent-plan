use ron;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;
use std::fs;
use std::io::Write;
use std::str;
/// struct representing the possible types of moves in a flat board game
#[derive(Serialize, Deserialize, Debug)]
pub struct Move {
    // which move in the game is this?
    pub number: u64,
    // which direction is the move going in?
    pub dir: Direction,
}

/// Direction is an enum of the possible types of moves that can be made
#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

/// implementation for Move, methods detail serialization to file
/// deserialization from file
impl Move {
    /// serialize a move object to file passed
    /// @param self, move object to be serialized
    /// @param fname, &str to be serialized in json format
    pub fn to_file_json(&self, fname: &str) -> Result<(), String> {
        // first open file passed, and return err if exists
        match fs::File::options()
            .write(true)
            .open(fname)
            .and_then(|mut file| {
                // we have the file, serialize self,
                // return err if there is any
                file.write(&(serde_json::to_vec(self).unwrap()))
            }) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }
    /// serialze a move object in RON notation to a vec<u8>
    /// @param serf, move object to be serialized
    /// @return, Result<vec<u8>, Error> Result buffer, or Error
    pub fn to_buf_ron(&self) -> Result<Vec<u8>, String> {
        // serialize self to string first,
        match ron::to_string(self) {
            Ok(val) => Ok(val.as_bytes().to_owned()),
            Err(err) => Err(err.to_string()),
        }
    }
}

/// deserialize a move object from a file passed as argument,
/// @param file, file to read from,
/// @return, return a Result<Move, String> with the error string on errors
/// returned from deserialization
pub fn from_file_json(fname: &str) -> Result<Move, String> {
    // first open file passed, and return err if exists
    // file defaults to opening in read only mode
    match fs::read(fname).and_then::<Move, _>(|contents| {
        // convert the vec<u8> into slice,
        // attempt to deserialized the data into move object
        Ok(serde_json::from_slice::<Move>(contents.as_ref()).unwrap())
    }) {
        Ok(val) => Ok(val),
        Err(err) => Err(err.to_string()),
    }
}

/// deserialze a move object formatted in RON,
/// @param, &[u8], buffer holding data, to be deserialized
/// @return, Result<Move, String>, error or Move object
pub fn from_ron(buf: &[u8]) -> Result<Move, String> {
    // convert bytes to string, and then to ron
    match str::from_utf8(buf) {
        Ok(s) => (Ok(ron::from_str(s).unwrap())),
        Err(err) => Err(err.to_string()),
    }
}

/// impl Display for Move
impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.number, self.dir)
    }
}
