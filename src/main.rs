use axum::{extract::Path, Router, routing::get};

async fn greet(path: Option<Path<String>>) -> String {
    let name = match path {
        Some(Path(name)) => name,
        None => "World".to_string(),
    };

    format!("Hello {}!", name)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(greet))
        .route("/:name", get(greet));

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
