use crate::AppMessage;
use crate::AppState;
use crate::db;
use axum_messages::Level;
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
enum Action {
    Register,
    Update,
    Delete,
}

#[derive(Deserialize, Clone)]
pub struct ChangeForm {
    action: Action,
    macaddr: String,
    descr: String,
    privacy: i8,
}

impl ChangeForm {
    pub async fn handle(self, state: &AppState, nickname: String) -> AppMessage {
        match self.action {
            Action::Register => self.register(state, nickname).await,
            Action::Update => self.update(state).await,
            Action::Delete => self.delete(state).await,
        }
    }

    pub async fn register(self, state: &AppState, nickname: String) -> AppMessage {
        let privacy = match db::PrivacyLevel::try_from(self.privacy) {
            Ok(privacy) => privacy,
            Err(_) => return (Level::Error, "unable to parse privacy level".to_string()),
        };
        let dbresult = db::Device {
            id: None,
            macaddr: self.macaddr,
            nickname: nickname.to_string(),
            descr: self.descr.clone(),
            privacy,
            present: false,
        }
        .create(&state.pool)
        .await;
        return match dbresult {
            Ok(_) => (
                Level::Info,
                format!("assinged device \"{}\" to {}", &self.descr, &nickname),
            ),
            Err(_) => (Level::Error, "unable to create device".to_string()),
        };
    }

    pub async fn update(self, state: &AppState) -> AppMessage {
        let mut device = match db::Device::for_mac(&state.pool, &self.macaddr).await {
            Ok(device) => device,
            Err(_) => {
                return (
                    Level::Error,
                    "unable to load device from database".to_string(),
                );
            }
        };
        device.privacy = match db::PrivacyLevel::try_from(self.privacy) {
            Ok(privacy) => privacy,
            Err(_) => return (Level::Error, "unable to parse privacy level".to_string()),
        };
        device.descr = self.descr;
        match device.update(&state.pool).await {
            Ok(device) => (Level::Info, format!("updated device \"{}\"", device.descr)),
            Err(_) => (Level::Error, "unable to update device".to_string()),
        }
    }

    pub async fn delete(self, state: &AppState) -> AppMessage {
        let device = match db::Device::for_mac(&state.pool, &self.macaddr).await {
            Ok(device) => device,
            Err(_) => {
                return (
                    Level::Error,
                    "unable to load device from database".to_string(),
                );
            }
        };
        let descr = device.descr.clone();
        match device.delete(&state.pool).await {
            Ok(_) => (
                Level::Info,
                format!("device \"{}\" has been deleted", descr),
            ),
            Err(_) => (Level::Error, "unable to delete device".to_string()),
        }
    }
}
