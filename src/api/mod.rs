// ==== API General State =====
use axum::extract::State as TractState;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::database;

mod http;
use self::http::claim::Claim;
use self::http::general::AppState;
use self::http::json::Json;
use self::http::response::{Response, ResponseContextResult, ResponseResult};

// ===== Public Struct =====
pub use self::http::claim::Secret;
pub use self::http::general::State;

// ===== Public API =====
use axum::routing::{delete, get, post};
use axum::Router;

mod general;
pub fn router_general(state: AppState) -> Router {
    Router::new()
        .route("/health", get(general::health::get::request))
        .with_state(state)
}

mod person;
pub fn router_person(state: AppState) -> Router {
    Router::new()
        .route("/create", post(person::create::post::request))
        .route("/verify", post(person::verify::post::request))
        .with_state(state)
}

mod admin;
pub fn router_admin(state: AppState) -> Router {
    Router::new()
        .route("/person", post(admin::person::post))
        .with_state(state)
}

mod binance;
pub fn router_binance(state: AppState) -> Router {
    Router::new()
        .route("/secret", get(binance::secret::get::request))
        .route("/secret", post(binance::secret::post::request))
        .route("/spot", get(binance::spot::get::request))
        .route("/spot", post(binance::spot::post::request))
        .route("/spot/buy", post(binance::spot::buy::post::request))
        .route("/spot/sell", post(binance::spot::sell::post::request))
        .route("/spot/order", get(binance::spot::order::get::request))
        .route("/spot/limit", get(binance::spot::limit::get::request))
        .route("/spot/limit", post(binance::spot::limit::post::request))
        .route("/spot/limit", delete(binance::spot::limit::delete::request))
        .route("/spot/predict", post(binance::spot::predict::post::request))
        .with_state(state)
}

fn require_request_payload<T>(value: Option<T>) -> ResponseContextResult<T> {
    match value {
        Some(v) => Ok(v),
        None => Err(Response::incompatible(String::from(
            "please provide valid request payload",
        ))),
    }
}
