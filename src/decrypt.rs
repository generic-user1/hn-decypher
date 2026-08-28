//! Items related directly to decryption
use std::num::{ParseIntError, TryFromIntError};
use thiserror::Error;

/// Reasons decryption can fail
#[derive(Error, Debug, Clone)]
pub enum DecryptError {
    #[error("value '{0}' could not be parsed as an i32")]
    ParseIntError(String, #[source] ParseIntError),

    #[error("value {encoded} when decoded to {decoded} couldn't convert to u32")]
    UnsignedConversionError {
        encoded: i32,
        decoded: i32,
        #[source]
        err: TryFromIntError
    },

    #[error("value {0} could not be decoded as a utf-8 char")]
    CharConversionError(u32)
}

impl From<(&str, ParseIntError)> for DecryptError {
    fn from(value: (&str, ParseIntError)) -> Self {
        DecryptError::ParseIntError(value.0.to_owned(), value.1)
    }
}
impl From<(i32, i32, TryFromIntError)> for DecryptError {
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
pub fn decrypt(data: &str, passcode: u16) -> Result<String, DecryptError> {
    /*
    The code snippit in-game operates on values of the C# `int` type, which is a 32 bit signed integer.
    It has to be 32-bit (rather than 8 or 16) because the encoding process makes the number larger
    (through additions and a multiply).

    I'm not sure it has to be *signed*. In theory, an encrypted output value should only ever be positive,
    since it's calculated by taking the (presumably positive) ordinal value of a character,
    adding a positive constant and an unsigned passcode, and multiplying by a positive
    constant. Making it unsigned eliminates a possible error case for us, since we need to convert
    from i32 to u32 before we can convert to char.

    In practice, I'd rather mimic the original behavior more closely than eliminate an error case
    that's unlikely to actually come up and a conversion that's fairly cheap, so we use i32 rather than u32.
    */
    const HALFMAX: i32 = (u16::MAX / 2) as i32;
    let passcode = passcode as i32;

    let mut out = Vec::new();
    for current_char in data.split_whitespace().filter(|&c| !c.is_empty()) {
        let as_int = current_char.parse::<i32>().map_err(|e| (current_char, e))?;
        let new_val = (as_int - HALFMAX - passcode) / 1822;
        let as_u32 = u32::try_from(new_val).map_err(|e| (as_int, new_val, e))?;
        let as_char = char::from_u32(as_u32).ok_or(DecryptError::CharConversionError(as_u32))?;
        out.push(as_char);
    }

    //This feels inefficient since we're building a string, getting
    //a slice from it via trim, then immediately building a new string,
    //but I think it's the only real way to do this
    Ok(out.into_iter().collect::<String>().trim().to_owned())
}
