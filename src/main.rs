use axum::{serve, Router};
use std::sync::Arc;
use tokio::net::TcpListener;

use harmony::{api, database, service};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database = database::Database::new(None);
    let secret = api::Secret::with_str("harmony");

    let state = Arc::new(api::State::new(database, secret));

    service::promise::initial_service_promise(state.clone()).await;

    let app = Router::new()
        .nest("/", api::router_general(state.clone()))
        .nest("/admin", api::router_admin(state.clone()))
        .nest("/person", api::router_person(state.clone()))
        .nest("/binance", api::router_binance(state.clone()));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    serve(listener, app).await.unwrap();
}
