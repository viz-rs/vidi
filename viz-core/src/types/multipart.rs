//! Represents a Multipart extractor.

use form_data::FormData;

use crate::{
    async_trait, Error, FromRequest, Incoming, IntoResponse, Request, RequestExt, Response,
    StatusCode,
};

use super::{Payload, PayloadError};

pub use form_data::{Error as MultipartError, Limits as MultipartLimits};

/// Extracts the data from the multipart body of a request.
pub type Multipart<T = Incoming> = FormData<T>;

impl<T> Payload for Multipart<T> {
    const NAME: &'static str = "multipart";

    // 2MB
    const LIMIT: u64 = 1024 * 1024 * 2;

    fn detect(m: &mime::Mime) -> bool {
        m.type_() == mime::MULTIPART && m.subtype() == mime::FORM_DATA
    }

    fn mime() -> mime::Mime {
        mime::MULTIPART_FORM_DATA
    }
}

#[async_trait]
impl FromRequest for Multipart {
    type Error = PayloadError;

    #[inline]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.multipart().await
    }
}

impl IntoResponse for MultipartError {
    fn into_response(self) -> Response {
        (
            match self {
                MultipartError::InvalidHeader
                | MultipartError::InvalidContentDisposition
                | MultipartError::FileTooLarge(_)
                | MultipartError::FieldTooLarge(_)
                | MultipartError::PartsTooMany(_)
                | MultipartError::FieldsTooMany(_)
                | MultipartError::FilesTooMany(_)
                | MultipartError::FieldNameTooLong(_) => StatusCode::BAD_REQUEST,
                MultipartError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
                MultipartError::Stream(_)
                | MultipartError::BoxError(_)
                | MultipartError::TryLockError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            self.to_string(),
        )
            .into_response()
    }
}

impl From<MultipartError> for Error {
    fn from(e: MultipartError) -> Self {
        e.into_error()
    }
}
