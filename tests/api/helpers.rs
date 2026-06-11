use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing_subscriber::fmt::TestWriter;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::startup::{Application, get_connection_pool};
use zero2prod::telemetry;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    let _ = telemetry::get_subscriber("debug", TestWriter::new).try_init();

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build appplication.");
    let address = format!("http://127.0.0.1:{}", application.port());
    tokio::spawn(application.run_until_stopped());
    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
