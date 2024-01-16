use axum::Form;
use axum::http::StatusCode;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(Form(data): Form<FormData>) -> StatusCode {
    StatusCode::OK
}
