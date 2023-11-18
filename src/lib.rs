use axum::{http::StatusCode, routing::get, Router};

async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub fn app() -> Router {
    Router::new().route("/health_check", get(health_check))
}
