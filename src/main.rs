use zero2prod::{app::app, configuration::get_configuration};

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("0.0.0.0:{}", configuration.application_port)
        .parse()
        .expect("Unable to parse socket addr");

    let server = axum::Server::bind(&address).serve(app().into_make_service());

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}
