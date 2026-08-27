use clap::Parser;
use hn_decypher::{DecryptError, decrypt};
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let encrypted = fs::read_to_string(args.dec_path).map_err(Error::FileRead)?;
    let decrypted = decrypt(&encrypted, 0)?;
    fs::write("out.txt", decrypted).map_err(Error::FileWrite)?;

    println!("success!");
    Ok(())
}

#[derive(Error, Debug)]
enum Error {
    #[error("Failed to read provided file")]
    FileRead(#[source] io::Error),

    #[error("Failed to write output file")]
    FileWrite(#[source] io::Error),

    #[error("Failed to decrypt provided file")]
    Decrypt(#[from] DecryptError)
}

/// Decrypts the \"dec\" file format from the video game Hacknet
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Path to the .dec file you want to decrypt
    #[arg(value_name = "FILE_PATH")]
    dec_path: PathBuf
}
