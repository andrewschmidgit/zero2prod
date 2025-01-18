use axum::{
    extract::Request,
    http::HeaderName,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{error, info_span};

use crate::routes::*;

const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn app(db_pool: PgPool) -> Router {
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

    let tracing_middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = request.headers().get(REQUEST_ID_HEADER);

                match request_id {
                    Some(request_id) => {
                        info_span!("http_request", request_id = ?request_id)
                    }
                    None => {
                        error!("could not extract request_id");
                        info_span!("http_request")
                    }
                }
            }),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id));

    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(db_pool)
        .layer(tracing_middleware)
}
