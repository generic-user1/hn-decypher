use clap::Parser;
use hn_decypher::{DecypherError, decypher_str};
use std::path::PathBuf;
use std::{fs, io};
use thiserror::Error;

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let encrypted = fs::read_to_string(args.in_path).map_err(Error::FileRead)?;
    let decyphered = decypher_str(&encrypted)?;
    println!("{:?}", decyphered.header_values);
    fs::write(args.out_path, decyphered.body).map_err(Error::FileWrite)?;
    Ok(())
}

#[derive(Error, Debug)]
enum Error {
    #[error("Failed to read provided file")]
    FileRead(#[source] io::Error),

    #[error("Failed to write output file")]
    FileWrite(#[source] io::Error),

    #[error("Failed to decypher file")]
    DecypherError(#[from] DecypherError)
}

/// Decrypts the \"dec\" file format from the video game Hacknet
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Path to the .dec file you want to decrypt
    #[arg(value_name = "INPUT_FILE_PATH")]
    in_path: PathBuf,

    /// Path that the decrypted file should be written to
    #[arg(value_name = "OUTPUT_FILEPATH", default_value = "./out.txt")]
    out_path: PathBuf
}
