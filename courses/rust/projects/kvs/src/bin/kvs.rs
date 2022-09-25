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
    // create File
    match &cli.command {
        // set command
        Commands::set(args) => {
            // open store at the current log directory
            let mut store = KvStore::open("./")?;
            store.set(
                args.key.as_ref().unwrap().to_owned(),
                args.value.as_ref().unwrap().to_owned(),
            )
        }
        Commands::get(args) => {
            let mut store: KvStore = KvStore::open("./")?;
            match store.get(args.key.as_ref().unwrap().to_owned()) {
                Ok(data) => match data {
                    Some(val) => {
                        println!("{}", val);
                        Ok(())
                    }
                    None => {
                        println!("Key not found");
                        Ok(())
                    }
                },
                Err(err) => Err(err),
            }
        }
        Commands::rm(args) => {
            let mut store = KvStore::open("./")?;
            store.remove(args.key.as_ref().unwrap().to_owned())
        }
    }
}
