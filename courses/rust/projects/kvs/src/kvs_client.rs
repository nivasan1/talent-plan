use crate::engines::{kvs::CommandData, kvs_engine::Result};
use log::*;
use serde_json;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
/// kvs-client is composed of
/// 1. StdErrLog, as well as a
/// 2. TcpStream connected to the addr passed in KvsClient::init()
pub struct KvsClient {
    stream: TcpStream,
    log: stderrlog::StdErrLog,
}

impl KvsClient {
    /// KvsClient init, this method instantiates a TcpStream with the provided address
    /// and a StdErrLog
    pub fn init<A: ToSocketAddrs>(addr: A) -> Result<KvsClient> {
        // first connect to socket provided,  and return the boxed err if necessary
        let stream = TcpStream::connect(addr).map_err(|err| Box::<dyn Error>::from(err))?;
        // return the KvsClient to caller
        Ok(KvsClient {
            stream: stream,
            log: stderrlog::new().verbosity(3).to_owned(),
        })
    }

    ///KvsClient send, this method  sends a serialized command over the TcpStream
    /// to the KvsServer
    pub fn send(&mut self, cmd: &CommandData) -> Result<Option<String>> {
        self.log.init()?;
        // write serialized bytes to TcpStream
        info!("sending request: {:?}", cmd);
        // send request
        let mut buf = Vec::<u8>::new();
        serde_json::to_writer(&mut buf, cmd).map_err(|err| Box::<dyn Error>::from(err))?;
        // write the buffer to TcpStream
        self.stream.write(&buf)?;
        // now receive the request
        match cmd {
            CommandData::Get { key: _ } => {
                info!("receiving response");
                let mut buf = String::new();
                // receive data from stream
                self.stream.read_to_string(&mut buf)?;
                // return value
                Ok(Some(buf))
            }
            CommandData::Rm { key: _ } => {
                info!("receiving response");
                let mut buf = String::new();
                // receive data from stream
                let size = self.stream.read_to_string(&mut buf)?;
                if size != 0 {
                    return Ok(Some(buf));
                }
                Ok(None)
                // return value
            }
            _ => Ok(None),
        }
    }
}
