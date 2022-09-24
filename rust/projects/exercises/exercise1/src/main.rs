pub mod prac;
use clap::{Args, Parser, Subcommand};
use std::fs;

#[derive(Parser)]
#[clap(author, version)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // part1
    part1(Part1),
    // part2
    part2(Part2),
}
#[derive(Args)]
struct Part1 {
    #[clap(value_parser)]
    val: Option<u64>,
}

#[derive(Args)]
struct Part2 {
    #[clap(value_parser)]
    val: Option<u64>,
}

fn main() {
    let cli = Cli::parse();
    //  initialize random direction
    let mut m = prac::Move {
        number: 0,
        dir: prac::Direction::Up,
    };
    // create file which serialized data will be written to
    fs::File::create("first").unwrap();
    // parse command data
    match &cli.command {
        Commands::part1(args) => {
            m.number = args.val.unwrap();
            // now print un-serialized data
            println!("unserialized: {}", m);
            // serialize to file
            m.to_file_json("first").unwrap();
            // now read data from file,
            let m2 = prac::from_file_json("first").unwrap();
            // print value
            println!("serialized -> deserialized: {}", m2)
        }
        Commands::part2(args) => {
            m.number = args.val.unwrap();
            // print m unserialized
            println!("unserialized: {}", m);
            // serialize to vec
            let buf = m.to_buf_ron().unwrap();
            //  print serialized data
            let m2 = prac::from_ron(&buf).unwrap();
            println!("serialized -> deserialized: {}", m2)
        }
    }
    // panic on an error
    fs::remove_file("first").unwrap()
}
