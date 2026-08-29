use thiserror::Error;

///The default passcode used by files without a password. Always used for encoding of header fields besides "ENCODED"
pub const DEFAULT_PASSCODE: u16 = 4065;

pub mod decrypt;
pub mod headers;
pub mod save_read;

#[derive(Error, Debug, Clone)]
/// Reasons the process of decrypting a file can fail
pub enum DecypherError {
    #[error("Missing either headers or body.")]
    MissingHeadersOrBody,

    #[error("Failed to decrypt headers of provided file")]
    DecryptHeaders(#[from] headers::HeaderDecryptError),

    #[error("Failed to decrypt body of provided file")]
    DecryptBody(#[from] decrypt::DecryptError)
}

#[derive(Debug, Clone)]
/// A decrypted file along with its header values
pub struct Decyphered {
    pub header_values: headers::HeaderValues,
    pub body: String
}

pub fn decypher_str(file_content: &str) -> Result<Decyphered, DecypherError> {
    let mut split = file_content.split('\n');
    let encrypted_headers = split.next().ok_or(DecypherError::MissingHeadersOrBody)?;
    let encrypted_body = split.next().ok_or(DecypherError::MissingHeadersOrBody)?;

    let header_values = headers::decrypt_headers(encrypted_headers)?;
    let body = decrypt::decrypt(encrypted_body, header_values.passcode)?;
    Ok(Decyphered {
        header_values,
        body
    })
}
