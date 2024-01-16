use zero2prod::app::app;

#[tokio::main]
async fn main() {
    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app().into_make_service())
        .await
        .unwrap();
}
