use anyhow::{Context, Result};
use argh::FromArgs;
use axum::extract::State;
use axum::{
    Router,
    middleware::from_extractor,
    routing::{get, post},
};
use axum_messages::{Level, MessagesManagerLayer};
use openssl_probe;
use sqlx::MySqlPool;
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
#[derive(FromArgs, Debug)]
struct Config {
    /// listen address
    #[argh(option, default = "\"[::1]:8080\".to_string()")]
    listen: String,

    /// database dsn
    #[argh(
        option,
        default = "\"mysql://administration:foosinn123@127.0.0.1/administration\".to_string()"
    )]
    dsn: String,

    /// unifi console hostname
    #[argh(option)]
    unifi_hostname: Option<String>,

    /// unifi console username
    #[argh(option)]
    unifi_username: Option<String>,

    /// unifi console password
    #[argh(option)]
    unifi_password: Option<String>,

    /// scan
    #[argh(switch)]
    scan: bool,
}

#[derive(Clone)]
pub struct AppState {
    pool: MySqlPool,
}

type AxumAppState = State<AppState>;

type AppMessage = (Level, String);

#[tokio::main]
async fn main() -> Result<()> {
    openssl_probe::init_ssl_cert_env_vars();

    let config: Config = argh::from_env();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("RUST_LOG"))
        .init();

    if config.scan {
        return scan::scan(config).await;
    }

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

    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
