use crate::db;
use crate::Level;
use crate::Message;
use crate::USER;
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Deserialize)]
pub struct RegisterForm {
    macaddr: String,
    descr: String,
    privacy: i8,
}

impl RegisterForm {
    pub async fn handle(self, request: &crate::Request) -> Message {
        let privacy = match db::PrivacyLevel::try_from(self.privacy) {
            Ok(privacy) => privacy,
            Err(_) => return (Level::Error, "unable to parse privacy level".to_string()),
        };

        let dbresult = db::Device {
            id: None,
            macaddr: self.macaddr,
            nickname: USER.to_string(),
            descr: self.descr,
            privacy,
            present: false,
        };
        let dbresult = dbresult
            .create()
            .unwrap()
            .execute(&request.state().pool)
            .await;
        return match dbresult {
            Ok(_) => (Level::Info, "assinged device".to_string()),
            Err(_) => (Level::Error, "unable to create device".to_string()),
        };
    }
}

#[derive(Deserialize)]
pub struct UpdateForm {
    macaddr: String,
    descr: String,
    privacy: i8,
}

impl UpdateForm {
    pub async fn handle(self, request: &crate::Request) -> Message {
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
}
