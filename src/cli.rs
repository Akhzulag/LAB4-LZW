use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::Result;

use crate::LZW::{decode, encode};
mod huffman;

#[derive(Subcommand)]
enum Commands {
    Encode {
        input_file: String,
        output_file: Option<String>,
    },
    Decode {
        input_file: String,
        output_file: Option<String>,
    },
}
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Encode {
            input_file,
            output_file,
        } => {
            let output_file = match output_file {
                Some(out) => out,
                None => input_file.split('.').next().unwrap().to_string() + ".huf",
            };
            encode(&input_file, &output_file)?;
            println!("Encoded");
        }
        Commands::Decode {
            input_file,
            output_file,
        } => {
            let output_file = match output_file {
                Some(out) => out,
                None => input_file.split('.').next().unwrap().to_string() + ".dec",
            };
            decode(&input_file, &output_file)?;
            println!("Encoded");
        }
    }
    Ok(())
}
