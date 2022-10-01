use crate::engines::{
    kvs::{CommandData, KvStore},
    kvs_engine::{ErrKeyNotFound, KvsEngine, Result},
    sled::SledKvsEngine,
};
use log::*;
use serde_json;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};
use stderrlog;
/// the kvs-server is composed of three parts
/// 1. A TcpListener - this listener is spawned
/// 2. A storage engine - impl KvStore, this is what will be
/// used to process requests from clients
/// 3. A slog::Logger, this will be used to log messages from server running in prod
pub struct KvsServer {
    engine: Box<dyn KvsEngine>,
    listener: TcpListener,
    log: stderrlog::StdErrLog,
}

impl KvsServer {
    /// KvsServer init, this method instantiates a KvStore in the current directory,
    /// binds a TcpListener to the provided socket, and instantiates a logger to stderr
    /// This returns a Result<KvsServer>
    pub fn init<A: ToSocketAddrs>(addr: A, is_sled: bool) -> Result<KvsServer> {
        // first bind to the socket provided, and return the boxed error if necessary
        let listener = TcpListener::bind(addr).map_err(|err| Into::<Box<dyn Error>>::into(err))?;
        // now that server is listening on provided port, open a KvStore in ./
        let engine: Box<dyn KvsEngine>;
        if is_sled {
            engine = Box::from(SledKvsEngine::open("./db")?);
        } else {
            engine = Box::from(KvStore::open("./")?)
        }

        // finally create the logger and recieve requests from the stream
        let log = stderrlog::new().verbosity(3).to_owned();
        Ok(KvsServer {
            engine: Box::from(engine),
            listener: listener,
            log: log,
        })
    }
    /// KvsServer serve, this method instantiates a KvStore in the current directory
    /// Instantiates it's logger, and begins serving on the designated port / address
    /// It returns a Result<()>
    pub fn serve(&mut self) -> Result<()> {
        // init logger
        self.log.init().map_err(Box::<dyn Error>::from)?;
        // iterate over all active connections
        for stream in self.listener.try_clone()?.incoming() {
            match stream {
                Ok(mut stream) => {
                    // log client request
                    info!("connection request: {:?}", stream);
                    // read the data and return err if needed
                    let mut buf = [0 as u8; 100];
                    // read data from stream into buf
                    // if there is a failure reading, close the connection on both sides
                    let cmd: CommandData;
                    // read into fixed size buf so the reader doesn't block for FIN packet
                    match stream.read(&mut buf) {
                        Err(e) => {
                            info!("error: {:?}", e);
                            // shutdown stream, `send` FIN packet to client to stop reading stream
                            stream.shutdown(Shutdown::Both)?;
                            return Err(Box::<dyn Error>::from(e));
                        }
                        Ok(size) => {
                            // read [0..size] into Command data
                            // deserialize
                            cmd = serde_json::from_slice(&buf[0..size])
                                .map_err(Box::<dyn Error>::from)?;
                        }
                    }
                    // handle request
                    self.handle_request(&cmd, stream)?;
                }
                Err(e) => {
                    // return the error if there is error in recv of TcpStream
                    return Err(Box::from(e));
                }
            }
        }
        Ok(())
    }

    /// KvsServer handle_request, this is a private method, it does 3 things
    /// 1. Match on Command Received from caller
    /// 2. Pass command to underlying storage engine
    /// 3. Return result to client, whatever it may be
    fn handle_request(&mut self, cmd: &CommandData, mut stream: TcpStream) -> Result<()> {
        // match on CommandData and execute requests as necessary
        match cmd {
            CommandData::Get { key } => {
                // get key from log
                match self.engine.get(key.to_owned())? {
                    None => {
                        // write the result back to client
                        info!("sending response: {:?}", "Key not found");
                        stream.write("Key not found".as_bytes())?;
                        // shutdown stream
                        stream.shutdown(Shutdown::Both)?;
                    }
                    Some(data) => {
                        // write the result back to client
                        info!("sending response: {:?}", data);
                        stream.write(data.as_bytes())?;
                        // shutdown stream
                        stream.shutdown(Shutdown::Both)?;
                    }
                }
                // write the data back to stream
            }
            CommandData::Set { key, value } => {
                // remove key from log
                self.engine.set(key.to_owned(), value.to_owned())?;
            }
            CommandData::Rm { key } => {
                // set (key, value) in log
                match self.engine.remove(key.to_owned()) {
                    // return nothing if successful
                    Ok(_) => (),
                    Err(e) => {
                        if e.is::<ErrKeyNotFound>() {
                            // write the result back to client
                            info!("sending response: {:?}", "Key not found");
                            stream.write("Key not found".as_bytes())?;
                            // shutdown stream
                            stream.shutdown(Shutdown::Both)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
