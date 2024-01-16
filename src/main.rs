use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::{app::app, configuration::get_configuration};

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Could not connect to Postgres.");

    let address = format!("0.0.0.0:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Unable to bind to address");

    let server = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app(pool).into_make_service());

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}
