use sqlx::postgres::PgPoolOptions;
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
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let listener = TcpListener::bind(address)
        .await
        .expect("Unable to bind to address");

    tracing::debug!("Listening on: {}", listener.local_addr().unwrap());

    let server = axum::serve(listener, app(pool).into_make_service());

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}
