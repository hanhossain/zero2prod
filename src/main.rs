use tracing_subscriber::util::SubscriberInitExt;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    telemetry::get_subscriber("info", std::io::stdout).init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::builder(configuration).build()?;
    application.run_until_stopped().await
}
