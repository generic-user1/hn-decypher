use std::num::{ParseIntError, TryFromIntError};

/// Reasons decryption can fail
#[derive(Debug)]
pub enum DecryptError {
    /// Failed to parse some value in the input string as an integer
    ParseIntError(ParseIntError),

    /// Failed to convert the decoded integer to a character
    ///
    /// This can be because the integer wasn't a valid [u32] (in which case, Some([TryFromIntError]) is included),
    /// or because the resulting [u32] wasn't a valid unicode [char] (in which case, None is included)
    CharConversionError(Option<TryFromIntError>)
}

impl From<ParseIntError> for DecryptError {
    fn from(value: ParseIntError) -> Self {
        DecryptError::ParseIntError(value)
    }
}
impl From<TryFromIntError> for DecryptError {
    fn from(value: TryFromIntError) -> Self {
        DecryptError::CharConversionError(Some(value))
    }
}

/// Decrypt given `data` with the provided `passcode`
///
/// This is more or less a direct translation from the game's own "Decypher_Test.cs"
/// file (found on "DEC Solutions Mainframe" under "Staff/J.Scott", saved as "Decypher_Test.dec")
pub fn decrypt(data: String, passcode: u16) -> Result<String, DecryptError> {
    const HALFMAX: i32 = (u16::MAX / 2) as i32;
    let passcode = passcode as i32;

    let mut out = Vec::new();
    for current_char in data.split(&[' ', '\n']) {
        let as_int = current_char.parse::<i32>()?;
        let new_val = (as_int - HALFMAX - passcode) / 1822;
        let as_char = char::from_u32(u32::try_from(new_val)?)
            .ok_or(DecryptError::CharConversionError(None))?;
        out.push(as_char);
    }

    //This is maybe inefficient - we take a vec of char, create a new String
    //(which might involve an allocation), then use trim (which gives a &str) and
    //use to_owned to turn that into a String (which might involve another allocation)
    Ok(out.into_iter().collect::<String>().trim().to_owned())
}
