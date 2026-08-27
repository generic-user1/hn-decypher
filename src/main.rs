use clap::Parser;
use hn_decypher::{
    decrypt::DecryptError,
    headers::{HeaderDecryptError, decrypt_headers}
};
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let encrypted = fs::read_to_string(args.dec_path).map_err(Error::FileRead)?;
    let encrypted_headers = encrypted.split('\n').next().unwrap();
    let decrypted_headers = decrypt_headers(encrypted_headers)?;
    println!("{:?}", decrypted_headers);
    Ok(())
}

#[derive(Error, Debug)]
enum Error {
    #[error("Failed to read provided file")]
    FileRead(#[source] io::Error),

    #[error("Failed to write output file")]
    FileWrite(#[source] io::Error),

    #[error("Failed to decrypt headers of provided file")]
    DecryptHeaders(#[from] HeaderDecryptError),

    #[error("Failed to decrypt body of provided file")]
    DecryptBody(#[from] DecryptError)
}

/// Decrypts the \"dec\" file format from the video game Hacknet
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Path to the .dec file you want to decrypt
    #[arg(value_name = "FILE_PATH")]
    dec_path: PathBuf
}
