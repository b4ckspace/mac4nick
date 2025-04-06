use anyhow::{Context, Result, anyhow};
use sqlx::MySqlPool;
use std::convert::TryFrom;
use std::net::Ipv4Addr;

#[derive(sqlx::FromRow, Debug)]
pub struct Device {
    pub id: Option<i32>,
    pub macaddr: String,
    pub nickname: String,
    pub descr: String,
    pub privacy: PrivacyLevel,
    pub present: bool,
}

impl Device {
    pub async fn create(self, state: &crate::AppState) -> Result<()> {
        if let Some(_) = self.id {
            return Err(anyhow!("device has already been created"));
        }
        sqlx::query(
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
        .bind(self.privacy)
        .execute(&state.pool)
        .await
        .context("unable to create device entry")
        .and(Ok(()))
    }

    pub async fn for_user(state: &crate::AppState, user: &str) -> Result<Vec<Device>> {
        sqlx::query_as(
            "
SELECT DISTINCT
  mtn.*,
  IF(al.iplong, TRUE, FALSE) present
FROM
  mac_to_nick mtn
LEFT OUTER JOIN
  alive_hosts al
ON
  mtn.macaddr = al.macaddr
  AND al.erfda > NOW() - INTERVAL 30 MINUTE
WHERE
  nickname LIKE ?
ORDER BY
  al.erfda DESC
",
        )
        .bind(user)
        .fetch_all(&state.pool)
        .await
        .context("unable to select by user")
    }

    pub async fn for_mac(state: &crate::AppState, macaddr: &str) -> Result<Device> {
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
        .fetch_one(&state.pool)
        .await
        .context("unable to select by mac")
    }

    pub async fn update(self, state: &crate::AppState) -> Result<Device> {
        let id = match self.id {
            Some(id) => id,
            None => return Err(anyhow!("selected device has no id")),
        };
        sqlx::query(
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
        .bind(id)
        .execute(&state.pool)
        .await
        .context("unable to update")
        .and(Ok(self))
    }

    pub async fn delete(self, state: &crate::AppState) -> Result<()> {
        let id = match self.id {
            Some(id) => id,
            None => return Err(anyhow!("selected device has no id")),
        };
        sqlx::query(
            "
DELETE FROM
  mac_to_nick
WHERE
  id = ?
LIMIT 1
",
        )
        .bind(id)
        .execute(&state.pool)
        .await
        .context("unable to delete")
        .and(Ok(()))
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct PrivacyLevelRow {
    privacy: PrivacyLevel,
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
    pub fn selected(&self, level: PrivacyLevel) -> &'static str {
        if *self as u8 == level as u8 {
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

impl AliveDevice {
    pub async fn unassinged(state: &crate::AppState) -> Result<Vec<AliveDevice>> {
        sqlx::query_as(
            "
SELECT DISTINCT
  al.macaddr macaddr,
  al.iplong iplong,
  mtn.nickname
FROM
  alive_hosts al
LEFT OUTER JOIN
  mac_to_nick mtn
ON
  al.macaddr = mtn.macaddr
WHERE
  mtn.nickname IS NULL
  AND al.erfda > NOW() - INTERVAL 30 MINUTE
ORDER BY
  al.erfda DESC
;
",
        )
        .fetch_all(&state.pool)
        .await
        .context("unable to load alive devices")
    }

    pub async fn loggable(&self, pool: &MySqlPool) -> Result<bool> {
        let privacy: Result<Option<PrivacyLevelRow>> =
            sqlx::query_as("SELECT privacy FROM mac_to_nick WHERE macaddr = ? LIMIT 1")
                .bind(&self.macaddr)
                .fetch_optional(pool)
                .await
                .context("unable to lookup device");
        let should_log = match privacy?.ok_or(anyhow!("device not found"))?.privacy {
            PrivacyLevel::DontLog => false,
            _ => true,
        };
        return Ok(should_log);
    }

    pub async fn log(pool: &MySqlPool, macaddr: &str, ip: &Ipv4Addr) -> Result<Self> {
        let device = AliveDevice {
            macaddr: macaddr.to_string(),
            iplong: ip.to_bits() as i32,
        };

        if !device.loggable(pool).await? {
            return Err(anyhow!("device should not be logged"));
        }

        sqlx::query("INSERT INTO alive_hosts (macaddr, iplong, erfda) VALUES (?, ?, NOW())")
            .bind(&device.macaddr)
            .bind(&device.iplong.to_string())
            .execute(pool)
            .await
            .context("unable to insert into db")?;
        Ok(device)
    }

    pub fn ip(&self) -> Ipv4Addr {
        Ipv4Addr::from([
            (self.iplong >> 24 & 0xff) as u8,
            (self.iplong >> 16 & 0xff) as u8,
            (self.iplong >> 8 & 0xff) as u8,
            (self.iplong & 0xff) as u8,
        ])
    }
}
