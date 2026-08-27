//! Items related directly to decryption
use std::num::{ParseIntError, TryFromIntError};
use thiserror::Error;

/// Reasons decryption can fail
#[derive(Error, Debug)]
pub enum DecryptError<'a> {
    /// Failed to parse some value in the input string as an integer
    ///
    /// Contains the erroring substring and the underlying [ParseIntError]
    #[error("value '{0}' could not be parsed as an i32")]
    ParseIntError(&'a str, #[source] ParseIntError),

    /// Failed to convert decoded value from i32 to u32
    ///
    /// Contains the still-encoded [i32], the decoded [i32], and the underlying [TryFromIntError]
    #[error("value {encoded} when decoded to {decoded} couldn't convert to u32")]
    UnsignedConversionError {
        encoded: i32,
        decoded: i32,
        #[source]
        err: TryFromIntError
    },

    /// Failed to convert the decoded integer to a character
    ///
    /// Contains the [u32] that couldn't be decoded
    #[error("value {0} could not be decoded as a utf-8 char")]
    CharConversionError(u32)
}

impl<'a> From<(&'a str, ParseIntError)> for DecryptError<'a> {
    fn from(value: (&'a str, ParseIntError)) -> Self {
        DecryptError::ParseIntError(value.0, value.1)
    }
}
impl<'a> From<(i32, i32, TryFromIntError)> for DecryptError<'a> {
    fn from(value: (i32, i32, TryFromIntError)) -> Self {
        DecryptError::UnsignedConversionError {
            encoded: value.0,
            decoded: value.1,
            err: value.2
        }
    }
}

/// Decrypt given `data` with the provided `passcode`
///
/// This is more or less a direct translation from the game's own "Decypher_Test.cs"
/// file (found on "DEC Solutions Mainframe" under "Staff/J.Scott", saved as "Decypher_Test.dec")
pub fn decrypt<'a>(data: &'a str, passcode: u16) -> Result<String, DecryptError<'a>> {
    const HALFMAX: i32 = (u16::MAX / 2) as i32;
    let passcode = passcode as i32;

    let mut out = Vec::new();
    for current_char in data.split_whitespace() {
        let as_int = current_char.parse::<i32>().map_err(|e| (current_char, e))?;
        let new_val = (as_int - HALFMAX - passcode) / 1822;
        let as_u32 = u32::try_from(new_val).map_err(|e| (as_int, new_val, e))?;
        let as_char = char::from_u32(as_u32).ok_or(DecryptError::CharConversionError(as_u32))?;
        out.push(as_char);
    }

    //This is maybe inefficient - we take a vec of char, create a new String
    //(which might involve an allocation), then use trim (which gives a &str) and
    //use to_owned to turn that into a String (which might involve another allocation)
    Ok(out.into_iter().collect::<String>().trim().to_owned())
}
