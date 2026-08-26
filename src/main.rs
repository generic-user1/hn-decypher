use clap::Parser;
use std::path::PathBuf;

fn main() {
    let args = Args::parse();

    println!("{:?}", args.dec_path);
}

/// Decrypts the \"dec\" file format from the video game Hacknet
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Path to the .dec file you want to decrypt
    #[arg(value_name = "FILE_PATH")]
    dec_path: PathBuf
}
