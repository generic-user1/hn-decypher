use clap::{Args, Parser, Subcommand};
use hn_decypher::{
    DecypherError, decypher_str,
    save_read::{
        FileReadOptions, ReadFileFromSaveError, SaveFindStrategy, read_file_from_save,
        save_file::ComputerFindStrategy
    }
};
use std::{fs, io, path::PathBuf};
use thiserror::Error;

fn main() -> Result<(), HnDecypherError> {
    let args = MainArgs::parse().mode;

    let input = args.get_input()?;
    let common_args = args.into_common();
    let output = match common_args.behavior {
        OutputBehavior::Verbatim => input,
        OutputBehavior::DecryptQuiet => decypher_str(&input)?.body,
        OutputBehavior::DecryptWithHeaders => {
            let decyphered = decypher_str(&input)?;
            print!("{}", decyphered.header_values);
            decyphered.body
        }
    };

    if let Some(output_path) = common_args.output_path {
        fs::write(&output_path, output).map_err(HnDecypherError::FileWrite)
    } else {
        println!("{}", output);
        Ok(())
    }
}

#[derive(Error, Debug)]
enum HnDecypherError {
    #[error("Failed to read provided file from disk")]
    FileRead(#[source] io::Error),

    #[error("Failed to read file from save data")]
    SaveRead(#[from] ReadFileFromSaveError),

    #[error("Failed to decypher file")]
    DecypherError(#[from] DecypherError),

    #[error("Failed to write output file to disk")]
    FileWrite(#[source] io::Error)
}

/// Decrypts the "dec" file format from the video game Hacknet without needing any password
///
/// Comes with capabilities to read a file from your actual filesystem, or read an in-game file
/// from an in-game computer.
#[derive(Parser, Debug)]
#[command(version, about)]
struct MainArgs {
    #[command(subcommand)]
    mode: Mode
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// Read an actual file from your actual filesystem
    Real {
        /// Quiet mode. Don't output header details (like source IP address and source file extension)
        /// to stdout, just output the file.
        #[arg(short, long)]
        quiet: bool,

        /// Path to an actual file on your computer to read from
        #[arg(value_name = "INPUT_FILE_PATH")]
        file_path: PathBuf,

        /// Path to write decrypted output to. If unspecified, will write to stdout
        #[arg(value_name = "OUTPUT_FILE_PATH")]
        output_path: Option<PathBuf>
    },

    /// Read an in-game file from an in-game computer's filesystem
    ///
    /// There are options for defining both what save file to read from, and what computer
    /// to read from within that save file. By default, will look for the last used in-game account
    /// in your platform's default save location, and read the file from the player's computer.
    Game {
        #[command(flatten)]
        save_find_args: SaveFindArgs,

        #[command(flatten)]
        computer_find_args: ComputerFindArgs,

        /// Quiet mode. Don't output header details (like source IP address and source file extension)
        /// to stdout, just output the file. Implied by `verbatim`, since without decrypting a file, there are no header details to output.
        #[arg(short, long)]
        quiet: bool,

        /// Don't decrypt the in-game file, just output it verbatim
        #[arg(short, long)]
        verbatim: bool,

        /// Absolute path to an in-game file on an in-game computer to read from
        #[arg(value_name = "INPUT_FILE_PATH")]
        file_path: String,

        /// Path to write decrypted output to. If unspecified, will write to stdout
        #[arg(value_name = "OUTPUT_FILE_PATH")]
        output_path: Option<PathBuf>
    }
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
struct SaveFindArgs {
    /// Direct path to your XML save file. Incompatibile with `save_dir` and `account_name`
    #[arg(short, long, value_name = "SAVE_FILE_PATH")]
    direct: Option<PathBuf>,

    /// Determine your XML save file from an in-game account name
    #[command(flatten)]
    from_profile: Option<ProfileArgs>
}

#[derive(Args, Debug)]
#[group(required = false, multiple = true)]
struct ProfileArgs {
    /// The path to the directory which your save data is stored in. Defaults to the default location for your platform
    #[arg(short, long, value_name = "SAVE_DIRECTORY")]
    save_dir: Option<PathBuf>,

    /// The name of the in-game account you wish to read from. Defaults to the last-used account.
    #[arg(short, long, value_name = "ACCOUNT_NAME")]
    account_name: Option<String>
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
struct ComputerFindArgs {
    /// Identify the in-game computer by its IP address
    #[arg(short, long, value_name = "IP_ADDRESS")]
    ip_address: Option<String>,

    /// Identify the in-game computer by its name
    #[arg(short, long, value_name = "COMPUTER_NAME")]
    name: Option<String>,

    /// Identify the in-game computer by its internal ID. The default behavior is to identify the in-game
    /// computer with internal ID "playerComp", which is the player's computer.
    #[arg(short, long, value_name = "COMPUTER_ID")]
    computer_id: Option<String>
}

/// Things we'd like to keep common between multiple subcommands, but have to be seperate
/// due to the way clap works
///
/// We want the command to be `hn-decypher [mode] [arguments]`, but to accomplish that, arguments
/// have to be descendants of each mode. If the arguments were children of MainArgs directly, the command
/// format would instead be `hn-decypher [some arguments] [mode] [other arguments]` which we don't want.
///
/// This struct exists to abstract some of that away, so that main can refer to these in a common way, but
/// clap still provides the command format we want.
struct CommonArgs {
    pub behavior: OutputBehavior,
    pub output_path: Option<PathBuf>
}

/// Represents combinations of `quiet` and `verbatim` that are actually valid
enum OutputBehavior {
    /// Output the file content verbatim and do not print any headers
    Verbatim,
    /// Output the decrypted file content, but do not print any headers
    DecryptQuiet,
    /// Output the decrypted file content and print headers
    DecryptWithHeaders
}

impl Mode {
    fn get_input(&self) -> Result<String, HnDecypherError> {
        match self {
            Self::Real { file_path, .. } => {
                fs::read_to_string(file_path).map_err(HnDecypherError::FileRead)
            }
            Self::Game {
                save_find_args,
                computer_find_args,
                file_path,
                ..
            } => {
                let save: SaveFindStrategy = match save_find_args {
                    SaveFindArgs {
                        direct: None,
                        from_profile: Some(from_profile)
                    } => SaveFindStrategy::ByAccount {
                        save_directory: from_profile.save_dir.as_deref(),
                        account_name: from_profile.account_name.as_deref()
                    },
                    SaveFindArgs {
                        direct: Some(save_path),
                        from_profile: None
                    } => SaveFindStrategy::DirectPath(save_path),
                    SaveFindArgs {
                        direct: None,
                        from_profile: None
                    } => SaveFindStrategy::default(),
                    SaveFindArgs {
                        direct: Some(_),
                        from_profile: Some(_)
                    } => {
                        //This state is invalid; we should never have both items being Some. Ideally our SavefindArgs could encode this in the type system,
                        //but clap doesn't seem to have a way to do that - though it should guarantee that this never actually happens.
                        panic!("invalid argument combination")
                    }
                };

                let computer: ComputerFindStrategy = match computer_find_args {
                    ComputerFindArgs {
                        ip_address: Some(ip),
                        name: None,
                        computer_id: None
                    } => ComputerFindStrategy::ByIp(ip),
                    ComputerFindArgs {
                        ip_address: None,
                        name: Some(name),
                        computer_id: None
                    } => ComputerFindStrategy::ByName(name),
                    ComputerFindArgs {
                        ip_address: None,
                        name: None,
                        computer_id: Some(id)
                    } => ComputerFindStrategy::ById(id),
                    ComputerFindArgs {
                        ip_address: None,
                        name: None,
                        computer_id: None
                    } => ComputerFindStrategy::default(),
                    _ => {
                        //Any other states (i.e. states where multiple members are Some) are invalid. Again, we would ideally encode this in the type system,
                        //which clap doesn't have a way to do, but clap should prevent us from getting to this point in this state
                        panic!("invalid argument combination")
                    }
                };
                read_file_from_save(FileReadOptions {
                    save,
                    computer,
                    target: file_path
                })
                .map_err(HnDecypherError::from)
            }
        }
    }

    fn into_common(self) -> CommonArgs {
        match self {
            Self::Real {
                quiet, output_path, ..
            } => CommonArgs {
                behavior: if quiet {
                    OutputBehavior::DecryptQuiet
                } else {
                    OutputBehavior::DecryptWithHeaders
                },
                output_path
            },
            Self::Game {
                quiet,
                verbatim,
                output_path,
                ..
            } => {
                let behavior = match (verbatim, quiet) {
                    (true, _) => OutputBehavior::Verbatim,
                    (false, true) => OutputBehavior::DecryptQuiet,
                    (false, false) => OutputBehavior::DecryptWithHeaders
                };
                CommonArgs {
                    behavior,
                    output_path
                }
            }
        }
    }
}
