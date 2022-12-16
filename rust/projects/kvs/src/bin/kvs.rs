use clap::Parser;
use kvs::cli::{Cli, Commands};
use kvs::engines::{
    kvs::KvStore,
    kvs_engine::{KvsEngine, Result},
};
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
