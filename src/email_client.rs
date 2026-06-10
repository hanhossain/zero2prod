use crate::domain::SubscriberEmail;
use reqwest::Client;
use serde::Serialize;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
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
            .await?;
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
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use wiremock::matchers::{header, method, path};
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

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender);

        Mock::given(method("POST"))
            .and(path("/api/v1/send"))
            .and(header("Content-Type", "application/json"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
