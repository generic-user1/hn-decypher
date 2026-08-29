//! Items related to reading a save file from the game
use std::{
    env,
    path::{Path, PathBuf}
};

pub mod accounts_file;

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
