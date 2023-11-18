use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let url = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", url))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[allow(clippy::let_underscore_future)]
fn spawn_app() -> String {
    let app = zero2prod::app();

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let s = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());

    let _ = tokio::spawn(s);
    format!("http://127.0.0.1:{}", port)
}
