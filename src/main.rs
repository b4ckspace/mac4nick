use anyhow::{Context, Result};
use axum::extract::State;
use axum::{
    Router,
    middleware::from_extractor,
    routing::{get, post},
};
use axum_messages::{Level, MessagesManagerLayer};
use envconfig::Envconfig;
use sqlx::MySqlPool;
use std::time::Duration;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::{MemoryStore, SessionManagerLayer};

mod db;
mod forms;
mod helpers;
mod middleware;
mod routes;
mod scan;
mod templates;

/// Configuration
#[derive(Clone, Envconfig)]
struct Config {
    #[envconfig(from = "LISTEN", default = "[::1]:8080")]
    listen: String,

    #[envconfig(from = "DATABASE_DSN")]
    dsn: String,

    #[envconfig(from = "UNIFI_HOSTNAME")]
    unifi_hostname: String,

    #[envconfig(from = "UNIFI_USERNAME")]
    unifi_username: String,

    #[envconfig(from = "UNIFI_PASSWORD")]
    unifi_password: String,

    #[envconfig(from = "MQTT_SPACE_STATUS_TOPIC", default = "sensor/space/status")]
    mqtt_spacestatus_topic: String,

    #[envconfig(
        from = "MQTT_MEMBER_PRESENT_TOPIC",
        default = "sensor/space/member/present"
    )]
    mqtt_member_present_topic: String,

    #[envconfig(
        from = "MQTT_MEMBER_NAMES_TOPIC",
        default = "sensor/space/member/names"
    )]
    mqtt_member_names_topic: String,

    #[envconfig(
        from = "MQTT_MEMBER_DEVICE_COUNT_TOPIC",
        default = "sensor/space/member/deviceCount"
    )]
    mqtt_member_device_count_topic: String,

    #[envconfig(from = "MQTT_HOST")]
    mqtt_host: String,
}

#[derive(Clone)]
pub struct AppState {
    pool: MySqlPool,
}

type AxumAppState = State<AppState>;

type AppMessage = (Level, String);

#[tokio::main]
async fn main() -> Result<()> {
    // Safety: This is required to find the os certificates.
    unsafe {
        openssl_probe::init_openssl_env_vars();
    }

    let config = Config::init_from_env().context("unable to parse environment")?;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("RUST_LOG"))
        .init();

    let job = tokio::spawn(async {
        let config = Config::init_from_env()
            .context("unable to parse environment")
            .unwrap();
        let scanner = scan::Scanner::new(&config);
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(err) = scanner.scan().await {
                tracing::error!("unable to scan for devices: {}", err);
            };
        }
    });

    let pool = MySqlPool::connect(&config.dsn)
        .await
        .context("unable to open database connection")?;
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

    let app_state = AppState { pool };
    let app = Router::new()
        .route("/healthz", get(routes::healthz))
        .route("/", get(routes::index))
        .route("/change", post(routes::change))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state)
        .layer(MessagesManagerLayer)
        .layer(session_layer)
        .layer(from_extractor::<middleware::ForwardAuth>())
        .layer(TraceLayer::new_for_http());

    tracing::info!("listening on {}", config.listen);
    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    axum::serve(listener, app).await?;

    job.await.expect("lock");
    Ok(())
}
