use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request},
    Json as AxumJson,
};
use serde::{Deserialize, Serialize};

use super::response::Response;

// ===== Json =====
#[derive(Debug, Serialize, Deserialize)]
pub struct Json<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for Json<T>
where
    AxumJson<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = Response<()>;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let req = Request::from_parts(parts, body);

        match AxumJson::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),

            Err(rejection) => {
                let response = Response {
                    ok: false,
                    data: None,
                    code: rejection.status().as_u16(),
                    error: Some(rejection.to_string()),
                };

                Err(response)
            }
        }
    }
}
