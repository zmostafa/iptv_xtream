use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    NetworkError(String),
    InvalidResponse(String),
    AuthenticationFailed,
    RateLimited(String),
    Unknown(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            ApiError::InvalidResponse(msg) => write!(f, "Invalid Response: {}", msg),
            ApiError::AuthenticationFailed => write!(f, "Authentication Failed."),
            ApiError::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
            ApiError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}
