use clap::Parser;
use kvs::cli::Server;
use kvs::engines::kvs_engine::Result;
use kvs::kvs_server::KvsServer;
use kvs::thread_pool::{shared_queue::SharedQueueThreadPool, ThreadPool, naive::NaiveThreadPool};
use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
fn main() -> Result<()> {
    let cli = Server::parse();

    // receive addr to serve on
    let addr: SocketAddr = cli
        .addr
        .to_socket_addrs()
        .map_err(|err| Into::<Box<dyn Error>>::into(err))?
        .next()
        .unwrap();

    let mut server: KvsServer;
    // unwrap engine
    match &cli.engine[..] {
        "kvs" => {
            // initialize server with kvs engine
            server = KvsServer::init::<SocketAddr>(addr, false)?;
        }
        "sled" => {
            // initialize server with sled engine
            server = KvsServer::init::<SocketAddr>(addr, true)?;
        }
        _ => panic!(),
    }
    // now serve requests
    server.serve(*(SharedQueueThreadPool::new(4)?))
}
