use ed25519_dalek::ed25519::signature;
use hex::FromHexError;
use http::StatusCode;
use serde_json::json;
use std::{env::VarError, fmt::Display};
use vercel_lambda::{error::VercelError, IntoResponse, Response};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Input: {0}")]
    InvalidInput(String),
    #[error("Invalid Environment Variable: {0}")]
    VarError(#[from] VarError),
    #[error("Decoding Error: {0}")]
    DecodingError(#[from] FromHexError),
    #[error("Decrypting Error: {0}")]
    DecryptingError(#[from] signature::Error),
    #[error("Parsing Body Error: {0}")]
    ParsingError(#[from] serde_json::Error),
    #[error("Request Error: {0}")]
    RequestError(#[from] reqwest::Error),
}

impl Into<VercelError> for Error {
    fn into(self) -> VercelError {
        VercelError::new(&self.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> http::Response<vercel_lambda::Body> {
        let error_message = &self.to_string();
        Response::builder()
            .status(match self {
                Error::InvalidInput(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
            .header("Content-Type", "text/json")
            .body(vercel_lambda::Body::from(
                json!({ "message": error_message }).to_string(),
            ))
            .expect("Internal Server Error")
    }
}
