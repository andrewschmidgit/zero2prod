use secrecy::ExposeSecret;
use sqlx::PgPool;
use tokio::net::TcpListener;
use zero2prod::{
    app::app,
    configuration::get_configuration,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() {
    let env_filter = format!(
        "{}=debug,tower_http=debug,axum::rejection=trace",
        env!("CARGO_CRATE_NAME")
    );
    let subscriber = get_subscriber("zero2prod".into(), env_filter, std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("Could not connect to Postgres.");

    let address = format!("0.0.0.0:{}", configuration.application_port);
    let listener = TcpListener::bind(address)
        .await
        .expect("Unable to bind to address");

    let server = axum::serve(listener, app(pool).into_make_service());

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}
