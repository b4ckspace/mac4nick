use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Debug)]
pub struct Entry {
    id: i32,
    macaddr: String,
    nickname: String,
    descr: String,
    privacy: i8,
    created: DateTime<Utc>,
}
