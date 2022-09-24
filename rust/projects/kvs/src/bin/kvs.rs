use clap::{Args, Parser, Subcommand};
use kvs::kvs::{KvStore, Result};

#[derive(Parser)]
#[clap(author, version)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // set value at key in state
    set(Set),
    //  get value at key from state
    get(Get),
    // remove value at key in state
    rm(Rm),
}

#[derive(Args)]
struct Get {
    #[clap(value_parser)]
    key: Option<String>,
}
#[derive(Args)]
struct Set {
    #[clap(value_parser)]
    key: Option<String>,
    #[clap(value_parser)]
    value: Option<String>,
}

#[derive(Args)]
struct Rm {
    #[clap(value_parser)]
    key: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::set(args) => {
            todo!()
        }
        Commands::get(args) => {
            todo!()
        }
        Commands::rm(args) => {
            todo!()
        }
    }
}
