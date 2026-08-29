//! Items related to reading a save file from the game
#[cfg(target_os = "windows")]
use std::{
    collections::HashMap,
    env, fs, io,
    path::{Path, PathBuf}
};
use thiserror::Error;

use crate::{
    DEFAULT_PASSCODE,
    decrypt::{DecryptError, decrypt}
};

// I only have the game installed on a Windows PC. Default save file locations for
// other platforms were retreived from https://www.pcgamingwiki.com/wiki/Hacknet#Save_game_data_location
// and may or may not be correct.
#[cfg(target_os = "windows")]
fn default_save_dir() -> PathBuf {
    let mut out = PathBuf::from(env::var_os("USERPROFILE").unwrap_or_default());
    out.push("Documents/My Games/Hacknet/Accounts");
    out
}

#[cfg(target_os = "macos")]
fn default_save_dir() -> PathBuf {
    let mut out = PathBuf::from(env::var_os("HOME").unwrap_or_default());
    out.push("Library/Application Support/Hacknet/Accounts");
    out
}

#[cfg(target_os = "linux")]
fn default_save_dir() -> PathBuf {
    let mut out = PathBuf::from(env::var_os("$XDG_DATA_HOME").unwrap_or_default());
    out.push("Hacknet/Accounts");
    out
}

fn accounts_file_path(save_dir: Option<PathBuf>) -> PathBuf {
    save_dir
        .unwrap_or_else(default_save_dir)
        .join(Path::new("Accounts.txt"))
}

/// One item from "Accounts.txt"; represents information about one in-game profile
///
/// The exact meanings of each attribute are educated guesses. They may be incorrect.
#[derive(Debug, Clone)]
pub struct ProfileData {
    /// Password for the profile
    pub password: String,

    /// The last time the profile was used
    ///
    /// This is stored as a string (rather than any other type used to store date and time like those provided by [chrono](https://docs.rs/chrono/latest/chrono/))
    /// because the exact format may or may not be locale-specific. To avoid
    /// any possible locale-dependent format changes, we store the string directly
    /// rather than attempting to parse it and getting it wrong.
    pub last_used_datetime: String,

    /// The name of the xml file which stores data for this account
    pub save_file_name: String
}

/// The content of "Accounts.txt"; represents information about all in-game profiles
#[derive(Debug, Clone)]
pub struct Accounts {
    /// The name of the most recently used profile.
    pub last_used_profile: Option<String>,

    /// Profile data, keyed by the profile name
    pub profiles: HashMap<String, ProfileData>
}

#[derive(Error, Debug)]
/// Reasons reading the "Accounts.txt" may fail
pub enum ReadAccountsError {
    #[error("failed to read \"Accounts.txt\"")]
    FileRead(#[from] io::Error),

    #[error("a profile in \"Accounts.txt\" was missing a required element")]
    //TODO: perhaps indicate which specific element?
    MissingElement,

    #[error("failed to decrypt password for a profile")]
    PasswordDecrypt(#[from] DecryptError)
}

/// Read the "Accounts.txt" file to get information on each saved profile
///
/// `save_dir` points to the directory containing the game save data. This
/// will try to default to the game's default save location for your platform.
///
/// This function is based on educated guesses about the "Accounts.txt" file's structure.
/// It may not do the same thing as (or interpret data in the same way as) the actual game when reading the file.
pub fn read_accounts(save_dir: Option<PathBuf>) -> Result<Accounts, ReadAccountsError> {
    let path = accounts_file_path(save_dir);
    let file_content = fs::read_to_string(path)?;

    //sections of the file appear to be delimited with this
    let mut parts = file_content.split("%------%");

    //beyond it possibly not being there at all, we also want to ensure that the profile name
    //actually points to a profile that exists
    let possible_last_used_profile = parts.next().map(|p| p.trim());

    let mut profiles = HashMap::new();

    for part in parts {
        /*
        each part has the following structure (content in {} indicates a field, content in <> is mine, added for clarity)
        %------%{PROFILE_NAME}
        __#DEC_ENC::::::164371 180769 160727 182591 162549 164371 162549 <this is a constant; it spells ENCODED with passcode 4065>
        {ENCRYPTED_PASSWORD} <the password is also encrypted in the DEC format with passcode 4095>
        __{LAST_USED_DATETIME}
        __{SAVE_FILE_NAME}

        The file ends with a %------% delimiter that has no profile name after it.
        */

        let mut fields = part.split("__");
        let name = fields
            .next()
            .ok_or(ReadAccountsError::MissingElement)?
            .trim();
        if name.is_empty() {
            //we're at the end of the file
            break;
        }
        let password_with_header = fields.next().ok_or(ReadAccountsError::MissingElement)?;
        let last_used_datetime = fields
            .next()
            .ok_or(ReadAccountsError::MissingElement)?
            .trim();
        let save_file_name = fields
            .next()
            .ok_or(ReadAccountsError::MissingElement)?
            .trim();

        let encrypted_password = password_with_header
            .split("\n")
            .nth(1)
            .ok_or(ReadAccountsError::MissingElement)?
            .trim();

        let password = decrypt(encrypted_password, DEFAULT_PASSCODE)?;
        profiles.insert(
            name.to_owned(),
            ProfileData {
                password,
                last_used_datetime: last_used_datetime.to_owned(),
                save_file_name: save_file_name.to_owned()
            }
        );
    }

    let last_used_profile = possible_last_used_profile
        .filter(|&n| profiles.contains_key(n))
        .map(|n| n.to_owned());
    Ok(Accounts {
        last_used_profile,
        profiles
    })
}
