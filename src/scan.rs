use anyhow::{Context, Result};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Deserialize;
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::time::Duration;

use crate::db;

#[derive(Deserialize, Debug)]
struct UnifiStaEntry {
    ip: Option<String>,
    mac: String,
}

#[derive(Deserialize, Debug)]
struct UnifiStaResponse {
    data: Vec<UnifiStaEntry>,
}

#[derive(Debug)]
struct User {
    username: String,
    privacy: db::PrivacyLevel,
}

impl From<&db::Device> for User {
    fn from(device: &db::Device) -> Self {
        Self {
            username: device.get_public_username(),
            privacy: device.privacy,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Scanner {
    client: AsyncClient,
    config: crate::Config,
}

impl Scanner {
    pub(crate) fn new(config: &crate::Config) -> Self {
        let mut options = MqttOptions::new("mac4nick", config.mqtt_host.clone(), 1883);
        options.set_keep_alive(Duration::from_secs(5));
        options.set_clean_session(true);
        let (client, mut eventloop) = AsyncClient::new(options, 10);

        tokio::task::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Err(err) => {
                        tracing::error!("mqtt issue: {}", err)
                    }
                    _ => (),
                };
            }
        });
        Self {
            client,
            config: config.clone(),
        }
    }

    async fn publish(&self, topic: &str, data: impl std::fmt::Display) {
        if let Err(err) = self
            .client
            .publish(topic, QoS::AtLeastOnce, true, format!("{}", data))
            .await
        {
            tracing::error!("unable to push to mqtt: {}", err);
        }
    }

    pub(crate) async fn scan(&self) -> Result<()> {
        let pool = MySqlPool::connect(&self.config.dsn)
            .await
            .context("unable to open database connection")?;

        let hostname = self.config.unifi_hostname.clone();

        let http_client = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .danger_accept_invalid_certs(true)
            .build()?;
        http_client
            .post(format!("https://{}/api/login", hostname))
            .body(format!(
                "{{\"username\": \"{}\", \"password\": \"{}\"}}",
                self.config.unifi_username, self.config.unifi_password
            ))
            .send()
            .await?;

        let resp = http_client
            .get(format!("https://{}/api/s/default/stat/sta", hostname))
            .send()
            .await?;

        let mut member_known: HashMap<String, User> = HashMap::default();
        let mut device_count = 0_u64;

        let unifi_sta = resp.json::<UnifiStaResponse>().await?;
        for discovered in unifi_sta.data.iter().filter(|device| device.ip.is_some()) {
            let device = match db::Device::for_mac(&pool, &discovered.mac).await {
                Ok(device) => device,
                Err(err) => {
                    tracing::debug!("unable to find device {:?}: {:?}", discovered.mac, err);
                    continue;
                }
            };

            if device.privacy == db::PrivacyLevel::HideUser {
                continue;
            }

            if let Some(known) = member_known.get(&device.nickname) {
                if device.privacy < known.privacy {
                    member_known.insert(device.nickname.clone(), (&device).into());
                }
            } else {
                member_known.insert(device.nickname.clone(), (&device).into());
            }

            device_count += 1;
            if let Err(err) = device.log(&pool, &discovered.ip.as_ref().unwrap()).await {
                tracing::debug!("unable to log device {:?}: {:?}", discovered.mac, err);
            };
        }

        let spacestatus = if device_count > 0 { "open" } else { "closed" };
        let member_count = member_known.len();
        let member_names = member_known
            .values()
            .map(|u| u.username.clone())
            .collect::<Vec<String>>()
            .join(", ");

        self.publish(&self.config.mqtt_spacestatus_topic, spacestatus)
            .await;
        self.publish(&self.config.mqtt_member_device_count_topic, device_count)
            .await;
        self.publish(&self.config.mqtt_member_present_topic, member_count)
            .await;
        self.publish(&self.config.mqtt_member_names_topic, &member_names)
            .await;

        tracing::info!(
            "discovered {} devices, {} members",
            device_count,
            member_count
        );
        Ok(())
    }
}
