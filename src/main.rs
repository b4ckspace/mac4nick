use argh::FromArgs;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::io;
use tide::sessions::{MemoryStore, SessionMiddleware};
mod db;
mod forms;
mod routes;
mod templates;

pub const USER: &str = "foosinn";

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

pub type Message = (Level, String);

#[derive(Default, Deserialize, Serialize)]
pub struct AppSession {
    messages: Vec<Message>,
}

impl AppSession {
    pub fn add_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    pub fn pop_messages(&mut self) -> Vec<Message> {
        let mut messages: Vec<Message> = Vec::new();
        std::mem::swap(&mut messages, &mut self.messages);
        messages
    }

    pub fn commit(self, request: &mut Request) {
        request.session_mut().insert("app", self).unwrap()
    }
}

impl From<&mut Request> for AppSession {
    fn from(request: &mut Request) -> Self {
        request.session().get("app").unwrap_or_default()
    }
}

pub type Request = tide::Request<State>;

#[async_std::main]
async fn main() -> Result<(), io::Error> {
    let config: Config = argh::from_env();

    let pool = MySqlPool::connect(&config.dsn)
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;

    let session_store =
        SessionMiddleware::new(MemoryStore::new(), config.session_secret.as_bytes());

    let mut app = tide::with_state(State { pool });
    app.with(session_store);
    app.at("/").get(routes::index);
    app.at("/register").post(routes::register);
    app.at("/update").post(routes::update);
    app.at("/healthz").get(routes::healthz);
    app.at("/static").serve_dir("static/")?;
    app.listen(config.listen).await
}
