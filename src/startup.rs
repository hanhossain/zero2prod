use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
use actix_web_flash_messages::FlashMessagesFramework;
use actix_web_flash_messages::storage::CookieMessageStore;
use secrecy::{ExposeSecret, SecretString};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Pool, Postgres};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> Pool<Postgres> {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub struct ApplicationBaseUrl(pub String);

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: SecretString,
    redis_uri: SecretString,
) -> Result<Server, anyhow::Error> {
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;

    let server = HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .wrap(TracingLogger::default())
            .route("/", web::get().to(routes::home))
            .route("/admin/dashboard", web::get().to(routes::admin_dashboard))
            .route("/admin/logout", web::post().to(routes::log_out))
            .route(
                "/admin/password",
                web::get().to(routes::change_password_form),
            )
            .route("/admin/password", web::post().to(routes::change_password))
            .route("/health_check", web::get().to(routes::health_check))
            .route("/login", web::get().to(routes::login_form))
            .route("/login", web::post().to(routes::login))
            .route("/newsletters", web::post().to(routes::publish_newsletter))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .route("/subscriptions/confirm", web::get().to(routes::confirm))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(hmac_secret.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub struct ApplicationBuilder {
    configuration: Settings,
    connection_pool: Option<PgPool>,
}

impl ApplicationBuilder {
    pub fn with_pool(mut self, pool: PgPool) -> Self {
        self.connection_pool = Some(pool);
        self
    }

    pub async fn build(self) -> Result<Application, anyhow::Error> {
        let connection_pool = self
            .connection_pool
            .unwrap_or_else(|| get_connection_pool(&self.configuration.database));

        let sender_email = self
            .configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = self.configuration.email_client.timeout();
        let email_client = EmailClient::new(
            self.configuration.email_client.base_url,
            sender_email,
            timeout,
        );

        let address = format!(
            "{}:{}",
            self.configuration.application.host, self.configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            self.configuration.application.base_url,
            self.configuration.application.hmac_secret,
            self.configuration.redis_uri,
        )
        .await?;
        Ok(Application { port, server })
    }
}

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub fn builder(configuration: Settings) -> ApplicationBuilder {
        ApplicationBuilder {
            configuration,
            connection_pool: None,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
