use anyhow::{anyhow, Result};
use sqlx::database::HasArguments;
use std::convert::TryFrom;
use std::net::IpAddr;

type QueryAs<'q, T> =
    sqlx::query::QueryAs<'q, sqlx::MySql, T, <sqlx::MySql as HasArguments<'q>>::Arguments>;
type Query<'q> = sqlx::query::Query<'q, sqlx::MySql, <sqlx::MySql as HasArguments<'q>>::Arguments>;

#[derive(sqlx::FromRow, Debug)]
pub struct Device {
    pub id: Option<i32>,
    pub macaddr: String,
    pub nickname: String,
    pub descr: String,
    pub privacy: PrivacyLevel,
    pub present: bool,
}

impl<'q> Device {
    pub fn create(&'q self) -> Result<Query<'q>> {
        if let Some(_) = self.id {
            return Err(anyhow!("device has already been created"));
        }
        Ok(sqlx::query(
            "
INSERT
INTO mac_to_nick
(macaddr, nickname, descr, privacy, created)
VALUES
(?, ?, ?, ?, NOW())
",
        )
        .bind(&self.macaddr)
        .bind(&self.nickname)
        .bind(&self.descr)
        .bind(self.privacy))
    }

    pub fn for_user(user: &'q str) -> QueryAs<'q, Self> {
        sqlx::query_as(
            "
SELECT DISTINCT
  mtn.*,
  IF(al.iplong, TRUE, FALSE) present
FROM
  mac_to_nick mtn
LEFT JOIN
  alive_hosts al
ON
  mtn.macaddr = al.macaddr
  AND al.erfda > NOW() - INTERVAL 24 DAY
WHERE
  nickname = ?
ORDER BY
  al.erfda DESC
",
        )
        .bind(user)
    }

    pub fn for_mac(macaddr: &'q str) -> QueryAs<'q, Self> {
        dbg!(&macaddr);
        sqlx::query_as(
            "
SELECT DISTINCT
  *,
  FALSE present
FROM
  mac_to_nick
WHERE
  macaddr = ?
",
        )
        .bind(macaddr)
    }

    pub fn update(&'q self) -> Result<Query<'q>> {
        let id = match self.id {
            Some(id) => id,
            None => return Err(anyhow!("selected device has no id")),
        };
        Ok(sqlx::query(
            "
UPDATE
  mac_to_nick
SET
  privacy = ?,
  descr = ?
WHERE
  id = ?
",
        )
        .bind(self.privacy as u8)
        .bind(&self.descr)
        .bind(id))
    }
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

impl TryFrom<i8> for PrivacyLevel {
    type Error = &'static str;

    fn try_from(i: i8) -> Result<Self, Self::Error> {
        let level = match i {
            0 => PrivacyLevel::ShowUserAndDevice,
            1 => PrivacyLevel::ShowUser,
            2 => PrivacyLevel::ShowAnonymous,
            3 => PrivacyLevel::HideUser,
            4 => PrivacyLevel::DontLog,
            _ => return Err("invalid privacy level"),
        };
        Ok(level)
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
  alive_hosts al
NATURAL LEFT JOIN
  mac_to_nick mtn
WHERE
  mtn.nickname IS NULL
  AND al.erfda > NOW() - INTERVAL 24 DAY
ORDER BY
  al.erfda DESC
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
