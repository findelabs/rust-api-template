use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response}
};

#[derive(Error, Debug)]
pub enum Error {
	#[error("{{\"error\": \"Cannot get config: Forbidden\"}}")]
	Forbidden,
	#[error("{{\"error\": \"Cannot get config: Unauthorized\"}}")]
	Unauthorized,
	#[error("{{\"error\": \"Cannot get config: Not found\"}}")]
	NotFound,

	#[error(transparent)]
    OpenTelemetry {
        #[from]
		source: opentelemetry::trace::TraceError
    },
	#[error(transparent)]
	TracingErr {
		#[from]
		source: tracing::dispatcher::SetGlobalDefaultError
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
