use argh::FromArgs;
use sqlx::MySqlPool;
use std::io;
mod db;
mod routes;
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
}

#[derive(Clone)]
pub struct State {
    pool: MySqlPool,
}

pub type Request = tide::Request<State>;

#[async_std::main]
async fn main() -> Result<(), io::Error> {
    let config: Config = argh::from_env();

    let pool = MySqlPool::connect(&config.dsn)
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;

    let mut app = tide::with_state(State { pool });
    app.at("/").get(routes::index);
    app.at("/register").post(routes::register);
    app.at("/healthz").get(routes::healthz);
    app.at("/static").serve_dir("static/")?;

    app.listen(config.listen).await
}
