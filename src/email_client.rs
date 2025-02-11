use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

use crate::domain::SubscriberEmail;

#[derive(Debug)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: SecretString,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html: &str,
        text: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: EmailAgent {
                email: self.sender.as_ref(),
                name: None
            },
            to: vec![EmailAgent {
                email: recipient.as_ref(),
                name: None
            }],
            subject,
            html,
            text,
        };

        self.http_client
            .post(url)
            .bearer_auth(self.authorization_token.expose_secret())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailRequest<'a> {
    from: EmailAgent<'a>,
    to: Vec<EmailAgent<'a>>,
    subject: &'a str,
    html: &'a str,
    text: &'a str,
}

#[derive(serde::Serialize)]
struct EmailAgent<'a> {
    email: &'a str,
    name: Option<&'a str>,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use claims::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use wiremock::{
        matchers::{bearer_token, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            SecretString::new(Faker.fake::<String>().into()),
            Duration::from_millis(200),
        )
    }

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                let is_from_valid = body.get("from").is_some_and(|f| f.get("email").is_some());
                let is_to_valid = body.get("to").is_some_and(|t|
                    t.get(0).is_some_and(|t|
                        t.get("email").is_some()));
                
                is_from_valid
                    && is_to_valid
                    && body.get("subject").is_some()
                    && body.get("html").is_some()
                    && body.get("text").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(bearer_token(
            email_client.authorization_token.expose_secret(),
        ))
        .and(header("Content-Type", "application/json"))
        .and(path("/email"))
        .and(method("POST"))
        .and(SendEmailBodyMatcher)
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

        // Act
        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(bearer_token(
            email_client.authorization_token.expose_secret(),
        ))
        .and(header("Content-Type", "application/json"))
        .and(path("/email"))
        .and(method("POST"))
        .and(SendEmailBodyMatcher)
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(bearer_token(
            email_client.authorization_token.expose_secret(),
        ))
        .and(header("Content-Type", "application/json"))
        .and(path("/email"))
        .and(method("POST"))
        .and(SendEmailBodyMatcher)
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&mock_server)
        .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200).set_delay(Duration::from_secs(60));
        Mock::given(bearer_token(
            email_client.authorization_token.expose_secret(),
        ))
        .and(header("Content-Type", "application/json"))
        .and(path("/email"))
        .and(method("POST"))
        .and(SendEmailBodyMatcher)
        .respond_with(response)
        .expect(1)
        .mount(&mock_server)
        .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_err!(outcome);
    }
}
