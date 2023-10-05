mod error;

use std::net::TcpListener;
use web_server::startup::run;
use shared::configuration::get_configuration;
use web_server::telemetry::{setup_subscriber, init_subscriber};
use tracing::level_filters::LevelFilter;
use shared::email_client::EmailClient;
use shared::qr_client::QRClient;

pub fn convert_error<E: ToString>(err: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let telemetry_subscriber = setup_subscriber(
        "web_server".into(),
        LevelFilter::INFO.to_string(),
        std::io::stdout
    );
    init_subscriber(telemetry_subscriber);


    let configuration = get_configuration().map_err(convert_error)?;
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let listener = TcpListener::bind(&address)?;
    let db_connection_pool = shared::db::initialize().await.map_err(convert_error)?;
    let qr_client = QRClient::new(
        configuration.qr_client.api_url.clone(),
        configuration.qr_client.api_key.clone(),
        configuration.qr_client.base_url.clone(),
        configuration.qr_client.base_image_path.clone(),
        configuration.qr_client.timeout()
    );

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    run(listener, db_connection_pool, qr_client, email_client)
        .map_err(convert_error)?
        .await
        .map_err(convert_error)?;

    Ok(())
}