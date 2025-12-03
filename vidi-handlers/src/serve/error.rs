use vidi_core::{IntoResponse, Response, StatusCode, ThisError};

/// Static file serving Error.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Method Not Allowed
    #[error("method not allowed")]
    MethodNotAllowed,

    /// Invalid path
    #[error("invalid path")]
    InvalidPath,

    /// Precondition failed
    #[error("precondition failed")]
    PreconditionFailed,

    /// Range could not be satisfied
    #[error("range could not be satisfied")]
    RangeUnsatisfied(u64),

    /// Io error
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            match self {
                Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
                Self::InvalidPath => StatusCode::BAD_REQUEST,
                Self::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
                Self::RangeUnsatisfied(_) => StatusCode::RANGE_NOT_SATISFIABLE,
                Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            self.to_string(),
        )
            .into_response()
    }
}

impl From<Error> for vidi_core::Error {
    fn from(e: Error) -> Self {
        e.into_error()
    }
}
