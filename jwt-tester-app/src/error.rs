use serde_json::{json, Value};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidToken,
    InvalidSignature,
    InvalidClaims,
    InvalidKey,
    Internal,
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    pub details: Option<Value>,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            details: None,
        }
    }

    pub fn invalid_token(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidToken, message)
    }

    pub fn invalid_signature(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidSignature, message)
    }

    pub fn invalid_claims(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidClaims, message)
    }

    pub fn invalid_key(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidKey, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    pub fn code(&self) -> &'static str {
        match self.kind {
            ErrorKind::InvalidToken => "INVALID_TOKEN",
            ErrorKind::InvalidSignature => "INVALID_SIGNATURE",
            ErrorKind::InvalidClaims => "INVALID_CLAIMS",
            ErrorKind::InvalidKey => "INVALID_KEY",
            ErrorKind::Internal => "INTERNAL_ERROR",
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self.kind {
            ErrorKind::InvalidToken => 10,
            ErrorKind::InvalidSignature => 11,
            ErrorKind::InvalidClaims => 12,
            ErrorKind::InvalidKey => 13,
            ErrorKind::Internal => 14,
        }
    }

    pub fn as_json(&self) -> Value {
        let mut error = json!({
            "code": self.code(),
            "message": self.message,
        });
        if let Some(details) = &self.details {
            error["details"] = details.clone();
        }
        json!({ "ok": false, "error": error })
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind as JwtErr;
        match err.kind() {
            JwtErr::InvalidSignature => AppError::invalid_signature(err.to_string()),
            JwtErr::ExpiredSignature
            | JwtErr::ImmatureSignature
            | JwtErr::InvalidIssuer
            | JwtErr::InvalidAudience
            | JwtErr::InvalidSubject
            | JwtErr::MissingRequiredClaim(_) => AppError::invalid_claims(err.to_string()),
            JwtErr::InvalidToken | JwtErr::Base64(_) | JwtErr::Json(_) | JwtErr::Utf8(_) => {
                AppError::invalid_token(err.to_string())
            }
            JwtErr::InvalidAlgorithm
            | JwtErr::MissingAlgorithm
            | JwtErr::InvalidAlgorithmName
            | JwtErr::InvalidKeyFormat
            | JwtErr::InvalidEcdsaKey
            | JwtErr::InvalidRsaKey(_) => AppError::invalid_key(err.to_string()),
            JwtErr::RsaFailedSigning | JwtErr::Crypto(_) => AppError::internal(err.to_string()),
            _ => AppError::internal(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppError, ErrorKind};
    use serde_json::json;

    #[test]
    fn codes_and_exit_codes_match() {
        let err = AppError::invalid_token("oops");
        assert_eq!(err.code(), "INVALID_TOKEN");
        assert_eq!(err.exit_code(), 10);

        let err = AppError::invalid_signature("sig");
        assert_eq!(err.code(), "INVALID_SIGNATURE");
        assert_eq!(err.exit_code(), 11);

        let err = AppError::invalid_claims("claims");
        assert_eq!(err.code(), "INVALID_CLAIMS");
        assert_eq!(err.exit_code(), 12);

        let err = AppError::invalid_key("key");
        assert_eq!(err.code(), "INVALID_KEY");
        assert_eq!(err.exit_code(), 13);

        let err = AppError::internal("boom");
        assert_eq!(err.code(), "INTERNAL_ERROR");
        assert_eq!(err.exit_code(), 14);
    }

    #[test]
    fn as_json_includes_details_when_set() {
        let mut err = AppError::new(ErrorKind::InvalidToken, "bad");
        err.details = Some(json!({ "field": "value" }));
        let value = err.as_json();
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "INVALID_TOKEN");
        assert_eq!(value["error"]["details"]["field"], "value");
    }
}
