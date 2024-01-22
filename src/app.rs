use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::routes::*;

pub fn app(db_pool: PgPool) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(db_pool)
        .layer(TraceLayer::new_for_http())
}
