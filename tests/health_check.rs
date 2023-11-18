use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    spawn_app();

    let client = reqwest::Client::new();
    client
        .get("http://0.0.0.0:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request");
}

#[allow(clippy::let_underscore_future)]
fn spawn_app() {
    let app = zero2prod::app();

    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();
    let s = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());

    let _ = tokio::spawn(s);
}
