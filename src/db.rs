use chrono::{DateTime, Utc};
use sqlx::database::HasArguments;
use std::net::IpAddr;

type QueryAs<'q, T> =
    sqlx::query::QueryAs<'q, sqlx::MySql, T, <sqlx::MySql as HasArguments<'q>>::Arguments>;

#[derive(sqlx::FromRow, Debug)]
pub struct Entry {
    pub id: i32,
    pub macaddr: String,
    pub nickname: String,
    pub descr: String,
    pub privacy: PrivacyLevel,
    pub created: DateTime<Utc>,
    pub present: bool,
}

#[derive(sqlx::Type, Debug, Clone, Copy)]
#[repr(i8)]
pub enum PrivacyLevel {
    ShowUserAndDevice = 0,
    ShowUser = 1,
    ShowAnonymous = 2,
    HideUser = 3,
    DontLog = 4,
}

impl<'q> Entry {
    pub fn all() -> QueryAs<'q, Self> {
        sqlx::query_as("SELECT * FROM mac_to_nick")
    }

    pub fn for_user(user: &'q str) -> QueryAs<'q, Self> {
        sqlx::query_as(
            "
SELECT
  mtn.*,
  IF(al.iplong != NULL, TRUE, FALSE) present
FROM
  mac_to_nick mtn,
  alive_hosts al
WHERE
  al.macaddr = mtn.macaddr
  AND nickname = ?
GROUP BY
  mtn.macaddr
",
        )
        .bind(user)
    }
}

impl PrivacyLevel {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn selected(&self, level: &PrivacyLevel) -> &'static str {
        if *self as u8 == *level as u8 {
            "selected"
        } else {
            ""
        }
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct AliveDevice {
    pub macaddr: String,
    pub iplong: i32,
}

impl<'q> AliveDevice {
    pub fn unassinged() -> QueryAs<'q, Self> {
        sqlx::query_as(
            "
SELECT DISTINCT
  al.macaddr macaddr,
  al.iplong iplong
FROM
  alive_hosts al,
  mac_to_nick mtn
WHERE
  mtn.nickname IS NULL
  AND al.erfda > NOW() - INTERVAL 24 DAY
",
        )
    }

    pub fn ip(&self) -> IpAddr {
        IpAddr::from([
            (self.iplong >> 24 & 0xff) as u8,
            (self.iplong >> 16 & 0xff) as u8,
            (self.iplong >> 8 & 0xff) as u8,
            (self.iplong & 0xff) as u8,
        ])
    }
}
