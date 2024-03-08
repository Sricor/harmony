use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::database::collection::PersonIdentifier;
use crate::time;

use super::general::AppState;
use super::response::Response;

// ===== Claim =====
type ClaimResult<T> = Result<T, ClaimError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    sub: PersonIdentifier,
    iat: i64,
    exp: i64,
}

impl Claim {
    fn with_approve(secret: &Secret, token: &str) -> ClaimResult<Self> {
        let token_data = decode::<Self>(token, &secret.decoding, &Validation::default())
            .map_err(|_| ClaimError::InvalidToken)?;

        if token_data.claims.is_expired() {
            return Err(ClaimError::ExpiredToken);
        }

        Ok(token_data.claims)
    }

    // Authorization
    pub fn approve(secret: &Secret, identifier: PersonIdentifier) -> ClaimResult<String> {
        let claim = Self {
            sub: identifier,
            iat: time::timestamp_millis(),
            exp: time::timestamp_millis_after_days(7),
        };

        let token = encode(&Header::default(), &claim, &secret.encoding);

        if let Ok(v) = token {
            return Ok(v);
        }

        Err(ClaimError::TokenCreation)
    }

    pub fn subject(&self) -> &PersonIdentifier {
        &self.sub
    }

    fn is_expired(&self) -> bool {
        time::is_timestamp_millis_expired(self.exp)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claim
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response<()>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| ClaimError::InvalidToken)?;

        let state = Arc::from_ref(state);

        // Decode the user data
        Ok(Claim::with_approve(state.secret(), bearer.token())?)
    }
}

// ===== Claim Error =====
#[derive(Debug)]
pub enum ClaimError {
    TokenCreation,
    InvalidToken,
    ExpiredToken,
}

impl Error for ClaimError {}

impl Display for ClaimError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::TokenCreation => "Token creation error",
            Self::InvalidToken => "Invalid token",
            Self::ExpiredToken => "Expired token",
        };

        write!(f, "{}", message)
    }
}

impl<T> From<ClaimError> for Response<T>
where
    T: Serialize,
{
    fn from(err: ClaimError) -> Self {
        let error: String = err.to_string();
        match err {
            ClaimError::InvalidToken => Self::strange(error),
            ClaimError::TokenCreation => Self::internal(error),
            ClaimError::ExpiredToken => Self::incompatible(error),
        }
    }
}

// ===== Secret =====
pub struct Secret {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Secret {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }

    pub fn with_str(str: &str) -> Self {
        Self::new(str.as_bytes())
    }
}
