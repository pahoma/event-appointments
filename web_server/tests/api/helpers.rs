use std::net::TcpListener;
use sqlx::{Connection, PgConnection, PgPool, Executor};
use uuid::Uuid;
use shared::configuration::{DatabaseSettings, get_configuration};
use web_server::startup::run;
use once_cell::sync::Lazy;
use tracing::metadata::LevelFilter;
use shared::email_client::EmailClient;
use shared::qr_client::QRClient;
use web_server::telemetry::{setup_subscriber, init_subscriber};

static APP_TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = LevelFilter::DEBUG.to_string();
    let subscriber_name = "web_server_app_test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = setup_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = setup_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&APP_TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let pg_pool = configure_database(&configuration.database).await;

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

    let qr_client = QRClient::new(
        configuration.qr_client.api_url.clone(),
        configuration.qr_client.api_key.clone(),
        configuration.qr_client.base_url.clone(),
        configuration.qr_client.base_image_path.clone(),
        configuration.qr_client.timeout()
    );

    let server = run(listener, pg_pool.clone(), qr_client, email_client).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: pg_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {

    // Create database

    let mut connection = PgConnection::connect_with(&config.connection_options_base())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.connection_options_with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./../migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}