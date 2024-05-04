//use serde_json::error::Error as SerdeError;
use axum::{
    body::{self},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Forbidden,
    Unauthorized,
    NotFound,
    OpenTelemetry(opentelemetry::trace::TraceError),
	TracingErr(tracing::dispatcher::SetGlobalDefaultError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Forbidden => f.write_str("{\"error\": \"Cannot get config: Forbidden\"}"),
            Error::Unauthorized => f.write_str("{\"error\": \"Cannot get config: Unauthorized\"}"),
            Error::NotFound => f.write_str("{\"error\": \"Cannot get config: Not found\"}"),
            Error::OpenTelemetry(ref err) => write!(f, "{}", err),
			Error::TracingErr(ref err) => write!(f, "{}", err),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let payload = self.to_string();
        let body = axum::body::Body::new(payload);

		let status_code = match &self {
            Error::Unauthorized | Error::Forbidden => StatusCode::UNAUTHORIZED,
            Error::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Response::builder()
            .status(status_code)
            .body(body)
            .unwrap()
    }
}

impl From<tracing::dispatcher::SetGlobalDefaultError> for Error {
    fn from(err: tracing::dispatcher::SetGlobalDefaultError) -> Error {
        Error::TracingErr(err)
    }
}

impl From<opentelemetry::trace::TraceError> for Error {
    fn from(err: opentelemetry::trace::TraceError) -> Error {
        Error::OpenTelemetry(err)
    }
}
