use thiserror::Error;

use crate::decrypt::decrypt;

pub mod decrypt;
pub mod headers;

#[derive(Error, Debug)]
/// Reasons the process of decrypting a file can fail
pub enum DecypherError {
    #[error("Missing either headers or body.")]
    MissingHeadersOrBody,

    #[error("Failed to decrypt headers of provided file")]
    DecryptHeaders(#[from] headers::HeaderDecryptError),

    #[error("Failed to decrypt body of provided file")]
    DecryptBody(#[from] decrypt::DecryptError)
}

#[derive(Debug)]
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
    let body = decrypt(encrypted_body, header_values.passcode)?;
    Ok(Decyphered {
        header_values,
        body
    })
}
