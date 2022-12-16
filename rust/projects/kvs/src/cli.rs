use clap::{Args, Parser, Subcommand};
/// Cli object used for kvs Cli
/// # SubCommands
/// get <key> - get value for key
/// set <key> <value> - set (key, value) to be persisted in log / cache
/// rm  <key> - remove (key, value) pair from cache and log
#[derive(Parser)]
#[clap(author, version)]
pub struct Cli {
    #[clap(subcommand)]
    /// available commands for kvs
    pub command: Commands,
}

/// Cli interface for kvs-client
/// # SubCommands
/// get <key> - get value for key
/// set <key> <value> - set (key, value) to be persisted in log / cache
/// rm  <key> - remove (key, value) pair from cache and log
/// # Flags
/// addr <address:port> - ip address / port on which kvs-server is serving
#[derive(Parser)]
#[clap(author, version, infer_subcommands = true)]
pub struct Client {
    /// commands available for CLI kvs client, (get, rm, set)
    #[clap(subcommand)]
    pub command: Commands,
    /// optional argument, ipaddr / port that kvs-server is serving on
    #[clap(long, value_parser, action, default_value = "127.0.0.1:4000")]
    pub addr: String,
}

/// Cli interface for kvs-server
/// # Flags
/// addr <address:port> - ip address / port on which kvs-server is serving
/// engine <engine> - the kvs backend to be used, sled / kvs

#[derive(Parser)]
#[clap(author, version)]
pub struct Server {
    /// <address>:<port> to serve on
    #[clap(long, value_parser, action, default_value = "127.0.0.1:4000")]
    pub addr: String,
    /// kvs engine to be used
    #[clap(long, value_parser, action, default_value = "kvs")]
    pub engine: String,
}

/// Available commands for kvs / kvs-client
#[derive(Subcommand)]
pub enum Commands {
    // set value at key in state
    set(Set),
    //  get value at key from state
    get(Get),
    // remove value at key in state
    rm(Rm),
}

#[derive(Args)]
/// standard get commands for kvs / kvs-client
/// kvs: the key for which to receive value for
pub struct Get {
    /// key passed key from which to get value
    #[clap(value_parser)]
    pub key: Option<String>,
}

/// standard set command,
/// key: key for which to set value to
/// value: value that will be set with `key`
/// # Behavior
/// If key already exists, overwrites key
#[derive(Args)]
pub struct Set {
    /// key to set value to
    #[clap(value_parser)]
    pub key: Option<String>,
    /// value that will be set with key
    #[clap(value_parser)]
    pub value: Option<String>,
}

/// Standard Rm Command
/// # Behavior
/// Removes (key, value) pair from cache, on compactions of log
/// the log entry for all sets / gets / rms will be removed from the log
/// # Errors
/// ErrNotFound - Attempt to remove key associated with non-existent value
#[derive(Args)]
pub struct Rm {
    #[clap(value_parser)]
    /// key of (key, value) pair to be removed
    pub key: Option<String>,
}
