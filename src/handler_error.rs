use std::fmt::Display;

use hyper::{http::HeaderValue, StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct HandlerError {
    pub message: String,
    pub code: StatusCode,
    pub origin: Option<HeaderValue>,
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl HandlerError {
    pub fn new(message: impl Into<String>, code: StatusCode) -> Self {
        HandlerError { message: message.into(), code, origin: None }
    }

    pub fn new_with_origin(message: impl Into<String>, code: StatusCode, origin: HeaderValue) -> Self {
        HandlerError { message: message.into(), code, origin: Some(origin) }
    }
}
