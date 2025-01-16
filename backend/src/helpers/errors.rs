use actix_http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

#[derive(Debug, Clone)]
pub enum ClipErrorType {
    InvalidUri,
    Error,
}

#[derive(Debug)]
pub struct ClipError {
    pub message: Option<String>,
    pub err_type: ClipErrorType,
}

impl ClipError {
    pub fn _message(&self) {
        match &self.message {
            Some(msg) => msg.clone(),
            None => String::from(""),
        };
    }
    pub fn set_type(&mut self, err_type: ClipErrorType) {
        self.err_type = err_type;
    }
}

impl std::fmt::Display for ClipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<String> for ClipError {
    fn from(err: String) -> ClipError {
        ClipError {
            message: Some(err),
            err_type: ClipErrorType::Error,
        }
    }
}

impl ResponseError for ClipError {
    fn status_code(&self) -> actix_http::StatusCode {
        match self.err_type {
            ClipErrorType::InvalidUri => StatusCode::NOT_FOUND,
            ClipErrorType::Error => StatusCode::BAD_REQUEST,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.message.clone())
    }
}
