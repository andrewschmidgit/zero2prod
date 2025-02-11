use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{Executor, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{domain::{NewSubscriber, SubscriberEmail, SubscriberName}, email_client::EmailClient, startup::AppState};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { name, email })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(data, database, email_client),
    fields(
        subscriber_email = %data.email,
        subscriber_name = %data.name,
    )
)]
pub async fn subscribe(
    State(AppState { database, email_client, base_url, .. }): State<AppState>,
    Form(data): Form<FormData>) -> StatusCode {
    let new_subscriber = match data.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let Ok(mut transaction) = database.begin().await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let Ok(subscriber_id) = insert_subscriber(&mut transaction, &new_subscriber).await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let subscription_token = generate_random_subscription_token();
    if store_token(&mut transaction, subscriber_id, &subscription_token).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if transaction.commit().await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if send_confirmation_email(&email_client, new_subscriber, base_url, &subscription_token).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    );
    transaction.execute(query) .await .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: String,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscriptions/confirm?subscription_token={}", base_url, subscription_token);

    let plain_body = format!("Welcome to our newsletter!\nVisit {} to confirm your subscription.", confirmation_link);
    let html_body = format!("Welcome to our newsletter!<bt />\
            Click <a href=\"{}\">here</a> to confirm your subscription.", confirmation_link);

    email_client.send_email(
        new_subscriber.email,
        "Welcome!",
        &html_body,
        &plain_body
    ).await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(transaction, new_subscriber)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')"#,
        id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        OffsetDateTime::now_utc()
    );
    transaction.execute(query)
        .await
        .map_err(|e| {
            tracing::error!("failed to execute query: {:?}", e);
            e
        })?;
    Ok(id)
}

fn generate_random_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
