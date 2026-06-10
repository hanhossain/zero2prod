use crate::domain::SubscriberEmail;
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, timeout: Duration) -> Self {
        Self {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            base_url,
            sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/api/v1/send", self.base_url);
        let request_body = SendEmailRequest {
            from: Recipient {
                email: self.sender.as_ref(),
            },
            to: vec![Recipient {
                email: recipient.as_ref(),
            }],
            subject,
            html: html_content,
            text: text_content,
        };
        self.http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: Recipient<'a>,
    #[serde(rename = "HTML")]
    html: &'a str,
    subject: &'a str,
    text: &'a str,
    to: Vec<Recipient<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Recipient<'a> {
    email: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claims::{assert_err, assert_ok};
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use std::time::Duration;
    use wiremock::matchers::{any, header, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("From")
                    .map(|x| x.as_object())
                    .flatten()
                    .is_some_and(|x| x.contains_key("Email"))
                    && body
                        .get("To")
                        .map(|x| x.as_array())
                        .flatten()
                        .is_some_and(|x| {
                            !x.is_empty()
                                && x.iter()
                                    .flat_map(|y| y.as_object())
                                    .all(|y| y.get("Email").is_some())
                        })
                    && body.get("Subject").is_some()
                    && body.get("HTML").is_some()
                    && body.get("Text").is_some()
            } else {
                false
            }
        }
    }

    /// Generate a random email subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// Generate a random subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    /// Get a test instance of `EmailClient`.
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(base_url, email(), Duration::from_millis(200))
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(method("POST"))
            .and(path("/api/v1/send"))
            .and(header("Content-Type", "application/json"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;
        assert_err!(outcome);
    }
}
