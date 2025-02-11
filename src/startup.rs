use std::sync::Arc;

use axum::{
    extract::Request,
    http::HeaderName,
    routing::{get, post},
    Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{error, info_span};

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{confirm, health_check, subscribe},
};

#[derive(Debug)]
pub struct Application {
    port: u16,
    server: axum::serve::Serve<TcpListener, Router, Router>,
}

#[derive(Clone)]
pub struct AppState {
    pub database: PgPool,
    pub email_client: Arc<EmailClient>,
    pub base_url: String
}

impl Application {
    pub async fn build(settings: Settings) -> Self {
        let connection_pool = get_connection_pool(&settings.database);
        let sender_email = settings
            .email_client
            .sender()
            .expect("Invalid sender email address");

        let timeout = settings.email_client.timeout();
        let email_client = EmailClient::new(
            settings.email_client.base_url,
            sender_email,
            settings.email_client.authorization_token,
            timeout,
        );

        let address = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );

        let listener = TcpListener::bind(address)
            .await
            .expect("Unable to bind to address");
        let port = listener.local_addr().unwrap().port();
        let server = axum::serve(listener, run(connection_pool, email_client, settings.application.base_url));

        Self { port, server }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(settings: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(settings.with_db())
}

const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn run(db_pool: PgPool, email_client: EmailClient, base_url: String) -> Router {
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

    let state = AppState {
        database: db_pool,
        email_client: Arc::new(email_client),
        base_url
    };

    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .with_state(state)
        .layer(tracing_middleware)
}
