//! Items related to reading a save file from the game
use std::{
    env, fs, io,
    path::{Path, PathBuf}
};
use thiserror::Error;
pub mod accounts_file;
pub mod save_file;

/// How to locate the XML save file
#[derive(Debug)]
pub enum SaveFindStrategy<'a, 'b> {
    /// Directly use a path to the XML save file
    DirectPath(&'a Path),

    /// Locate the XML save file by reading the "Accounts.txt" file
    /// and identifying what XML file corresponds to the given account name
    ///
    /// Requires the path to your save directory and the name of the account in question.
    /// The save directory defaults to the default save location for your platform,
    /// and the account name defaults to the last used account
    ByAccount {
        save_directory: Option<&'a Path>,
        account_name: Option<&'b str>
    }
}
impl<'a, 'b> Default for SaveFindStrategy<'a, 'b> {
    /// Returns [SaveFindStrategy::ByAccount] with both `save_directory` and `account_name` set to [None]
    ///
    /// Put differently, the default [SaveFindStrategy] is to assume the save directory is the default for your platform,
    /// and to use the last used account
    fn default() -> Self {
        SaveFindStrategy::ByAccount {
            save_directory: None,
            account_name: None
        }
    }
}

/// Ways locating the XML save file can fail
#[derive(Error, Debug)]
pub enum SaveFindError {
    #[error("Failed to read Accounts.txt file")]
    ReadAccounts(#[from] accounts_file::ReadAccountsError),

    #[error("No account name was specified, and the Accounts.txt file had no default account")]
    NoDefaultAccount,

    #[error("Account name '{0}' did not have an entry within the Accounts.txt file")]
    NonexistentAccount(String)
}

/// Options for how to read an in-game file from an in-game computer (by reading an XML save file)
#[derive(Debug)]
pub struct FileReadOptions<'a, 'input, 'b, 'c> {
    /// Options for locating your XML save file
    pub save: SaveFindStrategy<'a, 'b>,

    /// Options for locating the in-game computer within your save file
    pub computer: save_file::ComputerFindStrategy<'a, 'input, 'b>,

    /// The path to the target file on the in-game computer
    pub target: &'c str
}
impl<'a, 'input, 'b, 'c> Default for FileReadOptions<'a, 'input, 'b, 'c> {
    /// The default [FileReadOptions] is to open the last-used account's save file, and read a file from the player's in-game computer.
    ///
    /// **Note**: the default value of `target` is the empty string, which is unlikely to be a valid file path. You almost certainly
    /// want to set `target` explicitly, and only populate other fields using [Default]. For example:
    /// ```
    /// use hn_decypher::save_read::FileReadOptions;
    ///
    /// let options = FileReadOptions{target:"some/interesting/file.dec", ..Default::default()};
    /// ```
    //Ideally we would just derive Default, but since we want to have the above note on specifying the path, we have to write it manually
    fn default() -> Self {
        FileReadOptions {
            save: Default::default(),
            computer: Default::default(),
            target: Default::default()
        }
    }
}

#[derive(Error, Debug)]
pub enum ReadFileFromSaveError {
    #[error("failed to locate the XML save file")]
    SaveFindError(#[from] SaveFindError),

    #[error("failed to read the XML save file")]
    ReadError(#[from] io::Error),

    #[error("failed to parse the XML save file")]
    ParseError(#[from] roxmltree::Error),

    #[error("failed to find the target computer")]
    ComputerError(#[from] save_file::ComputerError),

    #[error("file with specified path not found on target computer")]
    FileNotFound
}

/// Read an in-game file from an in-game computer by reading an XML save file.
pub fn read_file_from_save(options: FileReadOptions) -> Result<String, ReadFileFromSaveError> {
    let save_file_content = match options.save {
        SaveFindStrategy::DirectPath(p) => fs::read_to_string(p)?,
        SaveFindStrategy::ByAccount {
            save_directory,
            account_name
        } => {
            let accounts =
                accounts_file::read_accounts(save_directory).map_err(SaveFindError::from)?;
            let account_name = account_name
                .or(accounts.last_used_profile.as_deref())
                .ok_or(SaveFindError::NoDefaultAccount)?;
            let account_file_name = &accounts
                .profiles
                .get(account_name)
                .ok_or(SaveFindError::NonexistentAccount(account_name.to_owned()))?
                .save_file_name;
            let save_file_path = if let Some(save_directory) = save_directory {
                save_directory.join(account_file_name)
            } else {
                default_save_dir().join(account_file_name)
            };
            fs::read_to_string(save_file_path)?
        }
    };

    let parsed = roxmltree::Document::parse(&save_file_content)?;
    let computer = save_file::Computer::try_new(&parsed, options.computer)?;
    computer
        .file_content(options.target)
        .ok_or(ReadFileFromSaveError::FileNotFound)
        .map(|c| c.to_owned())
}

// I only have the game installed on a Windows PC. Default save file locations for
// other platforms were retreived from https://www.pcgamingwiki.com/wiki/Hacknet#Save_game_data_location
// and may or may not be correct.
#[cfg(target_os = "windows")]
fn default_save_dir() -> PathBuf {
    [
        Path::new(&env::var_os("USERPROFILE").unwrap_or_default()),
        Path::new("Documents/My Games/Hacknet/Accounts")
    ]
    .into_iter()
    .collect()
}

#[cfg(target_os = "macos")]
fn default_save_dir() -> PathBuf {
    [
        Path::new(&env::var_os("HOME").unwrap_or_default()),
        Path::new("Library/Application Support/Hacknet/Accounts")
    ]
    .into_iter()
    .collect()
}

#[cfg(target_os = "linux")]
fn default_save_dir() -> PathBuf {
    [
        Path::new(&env::var_os("$XDG_DATA_HOME").unwrap_or_default()),
        Path::new("Hacknet/Accounts")
    ]
    .into_iter()
    .collect()
}
