use clap::Parser;
use kvs::cli::{Client, Commands};
use kvs::engines::{
    kvs::CommandData,
    kvs_engine::{ErrKeyNotFound, Result},
};
use kvs::kvs_client::KvsClient;
use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
fn main() -> Result<()> {
    // parse arguments / command passed to the cli
    let cli = Client::parse();
    // fail if the addr is formatted incorrectly
    let addr: SocketAddr = cli
        .addr
        .to_socket_addrs()
        .map_err(|err| Into::<Box<dyn Error>>::into(err))?
        .next()
        // panic here, as it should never be the case this is is a nil value
        .unwrap();

    let mut client = KvsClient::init::<SocketAddr>(addr)?;
    let cmd: CommandData;

    match &cli.command {
        Commands::set(args) => {
            // must have key
            cmd = CommandData::Set {
                key: args.key.as_ref().unwrap().to_owned(),
                value: args.value.as_ref().unwrap().to_owned(),
            };
            // commands initialized, now send the request to server
        }
        Commands::get(args) => {
            // must have key
            cmd = CommandData::Get {
                key: args.key.as_ref().unwrap().to_owned(),
            };
            // commands initialized, now send the request to server
        }
        Commands::rm(args) => {
            // must have key
            cmd = CommandData::Rm {
                key: args.key.as_ref().unwrap().to_owned(),
            };
            // commands initialized, now send the request to server
        }
    }
    // commands initialized, now send the request to server
    let data = client.send(&cmd)?;
    if let Some(res) = data {
        println!("{}", res);
        // if this was an Rm fail out
        if let Commands::rm(key) = &cli.command {
            // return Err(Box::new(ErrKeyNotFound{key: key.key.clone().unwrap()}));
            return Err(Box::from(ErrKeyNotFound {
                key: key.key.to_owned().unwrap(),
            }));
        }
    }
    Ok(())
}
