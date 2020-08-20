use crate::db;
use crate::Level;
use crate::Message;
use crate::USER;
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Register,
    Update,
    Delete,
}

#[derive(Deserialize)]
pub struct ChangeForm {
    action: Action,
    macaddr: String,
    descr: String,
    privacy: i8,
}

impl ChangeForm {
    pub async fn handle(self, request: &crate::Request) -> Message {
        match self.action {
            Action::Register => self.register(request).await,
            Action::Update => self.update(request).await,
            Action::Delete => self.delete(request).await,
        }
    }

    pub async fn register(self, request: &crate::Request) -> Message {
        let privacy = match db::PrivacyLevel::try_from(self.privacy) {
            Ok(privacy) => privacy,
            Err(_) => return (Level::Error, "unable to parse privacy level".to_string()),
        };
        let dbresult = db::Device {
            id: None,
            macaddr: self.macaddr,
            nickname: USER.to_string(),
            descr: self.descr.clone(),
            privacy,
            present: false,
        };
        let dbresult = dbresult
            .create()
            .unwrap()
            .execute(&request.state().pool)
            .await;
        return match dbresult {
            Ok(_) => (
                Level::Info,
                format!("assinged device \"{}\" to {}", self.descr, USER),
            ),
            Err(_) => (Level::Error, "unable to create device".to_string()),
        };
    }

    pub async fn update(self, request: &crate::Request) -> Message {
        let mut device = match db::Device::for_mac(&self.macaddr)
            .fetch_one(&request.state().pool)
            .await
        {
            Ok(device) => device,
            Err(_) => {
                return (
                    Level::Error,
                    "unable to load device from database".to_string(),
                )
            }
        };
        device.privacy = match db::PrivacyLevel::try_from(self.privacy) {
            Ok(privacy) => privacy,
            Err(_) => return (Level::Error, "unable to parse privacy level".to_string()),
        };
        device.descr = self.descr;
        match device
            .update()
            .unwrap()
            .execute(&request.state().pool)
            .await
        {
            Ok(_) => (Level::Info, "updated device".to_string()),
            Err(_) => (Level::Error, "unable to update device".to_string()),
        }
    }

    pub async fn delete(self, request: &crate::Request) -> Message {
        let device = match db::Device::for_mac(&self.macaddr)
            .fetch_one(&request.state().pool)
            .await
        {
            Ok(device) => device,
            Err(_) => {
                return (
                    Level::Error,
                    "unable to load device from database".to_string(),
                )
            }
        };
        match device
            .delete()
            .unwrap()
            .execute(&request.state().pool)
            .await
        {
            Ok(_) => (Level::Info, "delete device".to_string()),
            Err(_) => (Level::Error, "unable to delete device".to_string()),
        }
    }
}
