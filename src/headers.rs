//! Items relating to processing of headers

use crate::decrypt::{DecryptError, decrypt};
use thiserror::Error;

/// Items from the header of a .dec file
///
/// The passcode isn't stored in the header directly, but
/// can be brute forced relatively easily.
#[derive(Debug)]
pub struct HeaderValues {
    pub passcode: u16,
    pub header_msg: String,
    pub src_ip: String,
    pub file_extension: Option<String>
}

/// Reasons decrypting headers can fail
#[derive(Error, Debug)]
pub enum HeaderDecryptError {
    #[error("provided header_data had only {0} element(s), expected at least 4")]
    InvalidHeaderLength(usize),

    #[error("provided header_data did not start with expected prefix \"#DEC_ENC\"")]
    MissingPrefix,

    #[error("failed to determine passcode (all possibilities attempted, none were successful")]
    DeterminePasscodeFail,

    #[error("provided input str could not be decrypted")]
    DecryptError(#[from] DecryptError)
}

pub fn decrypt_headers(header_data: &str) -> Result<HeaderValues, HeaderDecryptError> {
    let header_parts: Vec<_> = header_data.split("::").collect();

    if header_parts.len() < 4 {
        return Err(HeaderDecryptError::InvalidHeaderLength(header_parts.len()));
    }

    if header_parts[0] != "#DEC ENC" {
        return Err(HeaderDecryptError::MissingPrefix);
    }

    //try all passcodes until one leads to the check string being "ENCODED"
    for passcode in u16::MIN..=u16::MAX {
        let check_str = decrypt(header_parts[3], passcode)?;
        if check_str == "ENCODED" {
            let header_msg = decrypt(header_parts[1], passcode)?;
            let src_ip = decrypt(header_parts[2], passcode)?;
            let file_extension = header_parts
                .get(4)
                .map(|&h| decrypt(h, passcode))
                .map_or(Ok(None), |r| r.map(Some))?;

            return Ok(HeaderValues {
                passcode,
                header_msg,
                src_ip,
                file_extension
            });
        }
    }
    Err(HeaderDecryptError::DeterminePasscodeFail)
}
