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
}

#[derive(Clone)]
pub struct State {
    pool: MySqlPool,
}

pub type Request = tide::Request<State>;

#[async_std::main]
async fn main() -> Result<(), io::Error> {
    let config: Config = argh::from_env();

    let dsn = "mysql://administration:foosinn123@127.0.0.1/administration";
    let pool = MySqlPool::connect(dsn)
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;
    let mut conn = pool
        .acquire()
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;

    let mut app = tide::with_state(State { pool });
    app.at("/").get(routes::index);
    app.at("/healthz").get(routes::healthz);
    app.at("/static").serve_dir("static/")?;

    let entries: Vec<db::Entry> = sqlx::query_as("select * from mac_to_nick")
        .fetch_all(&mut conn)
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;

    entries.into_iter().for_each(|e| println!("{:?}", e));
    app.listen(config.listen).await
}
