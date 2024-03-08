use crate::database::error::RecorderError;
use axum::{
    http::{self as AxumHttp},
    response as AxumResponse, Json as AxumJson,
};
use serde::Serialize;

// ===== Response =====
pub type ResponseResult<T> = Result<Response<T>, Response<()>>;
pub type ResponseContextResult<T> = Result<T, Response<()>>;

#[derive(Serialize)]
pub struct Response<T> {
    pub ok: bool,
    pub code: u16,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> Response<T> {
    pub fn ok(data: T) -> Self {
        Self {
            ok: true,
            code: 200,
            data: Some(data),
            error: None,
        }
    }

    pub fn fobidden(error: String) -> Self {
        Self {
            ok: false,
            code: 403,
            data: None,
            error: Some(error),
        }
    }

    pub fn incompatible(error: String) -> Self {
        Self {
            ok: false,
            code: 400,
            data: None,
            error: Some(error),
        }
    }

    pub fn internal(error: String) -> Self {
        Self {
            ok: false,
            code: 500,
            data: None,
            error: Some(error),
        }
    }

    pub fn strange(error: String) -> Self {
        Self {
            ok: false,
            code: 401,
            data: None,
            error: Some(error),
        }
    }
}

impl<T> AxumResponse::IntoResponse for Response<T>
where
    T: Serialize,
{
    fn into_response(self) -> AxumResponse::Response {
        let code = AxumHttp::StatusCode::from_u16(self.code).unwrap();
        let payload = AxumJson(self);

        (code, payload).into_response()
    }
}

impl<T> From<RecorderError> for Response<T> {
    fn from(value: RecorderError) -> Self {
        tracing::error!("{}", value);

        Response::internal(String::from("something wrong"))
    }
}
