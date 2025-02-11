use axum::{extract::{Query, State}, http::StatusCode};
use sqlx::PgPool;
use uuid::Uuid;

use crate::startup::AppState;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(params)
)]
pub async fn confirm(State(AppState { database, .. }): State<AppState>, Query(params): Query<Parameters>) -> StatusCode {
    let Ok(id) = get_subscriber_id_from_token(&database, &params.subscription_token).await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let Some(id) = id else {
        return StatusCode::UNAUTHORIZED;
    };

    if confirm_subscriber(&database, id).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    StatusCode::OK
}

#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(database, subscription_token)
)]
async fn get_subscriber_id_from_token(database: &PgPool, subscription_token: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id
           FROM subscription_tokens
           WHERE subscription_token = $1
        "#,
        subscription_token
    )
        .fetch_optional(database)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(database, id)
)]
async fn confirm_subscriber(database: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        id
    )
        .execute(database)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}

