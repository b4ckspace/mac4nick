use argh::FromArgs;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::io;
use tide::sessions::{MemoryStore, SessionMiddleware};
mod db;
mod forms;
mod middleware;
mod routes;
mod session;
mod templates;

pub const USER: &str = "hansi";

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

    /// session secret
    #[argh(option, default = "\"thisisnotasecretthisisnotasecret\".into()")]
    session_secret: String,

    /// debug
    #[argh(switch)]
    log: bool,
}

#[derive(Clone)]
pub struct State {
    pool: MySqlPool,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    fn color(&self) -> &'static str {
        match self {
            Level::Info => "success",
            Level::Warn => "warning",
            Level::Error => "danger",
        }
    }
}

pub type Message = (Level, String);

pub type Request = tide::Request<State>;

#[async_std::main]
async fn main() -> Result<(), io::Error> {
    let config: Config = argh::from_env();
    if config.log {
        tide::log::start();
    }

    let pool = MySqlPool::connect(&config.dsn)
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;

    let session_store =
        SessionMiddleware::new(MemoryStore::new(), config.session_secret.as_bytes());

    let mut app = tide::with_state(State { pool });
    app.with(middleware::ErrorHandler::default());
    app.with(session_store);
    app.at("/").get(routes::index);
    app.at("/change").post(routes::change);
    app.at("/healthz").get(routes::healthz);
    app.at("/static").serve_dir("static/")?;
    app.listen(config.listen).await
}
