use sqlx::PgPool;
use tracing_subscriber::fmt::TestWriter;
use tracing_subscriber::util::SubscriberInitExt;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app(db_pool: PgPool) -> TestApp {
    let _ = telemetry::get_subscriber("debug", TestWriter::new).try_init();

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.application.port = 0;
        c
    };

    let application = Application::builder(configuration.clone())
        .with_pool(db_pool.clone())
        .build()
        .expect("Failed to build appplication.");
    let address = format!("http://127.0.0.1:{}", application.port());
    tokio::spawn(application.run_until_stopped());
    TestApp { address, db_pool }
}
