use thiserror::Error;

#[derive(Error, Debug)]
pub enum X402Error {
    #[error("Invalid payment header: {0}")]
    InvalidHeader(String),

    #[error("Invalid payment token: {0}")]
    InvalidToken(String),

    #[error("Token expired at {0}")]
    TokenExpired(String),

    #[error("Signature verification failed: {0}")]
    SignatureInvalid(String),

    #[error("Credential validation failed: {0}")]
    CredentialInvalid(String),

    #[error("Unsupported payment scheme: {0}")]
    UnsupportedScheme(String),

    #[error("Payment failed: {0}")]
    Payment(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl serde::Serialize for X402Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
