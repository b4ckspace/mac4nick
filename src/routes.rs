use crate::db;
use crate::templates;
use tide::prelude::*;

const USER: &str = "foosinn";

pub async fn healthz(_request: crate::Request) -> tide::Result {
    Ok("ok".into())
}

pub async fn index(request: crate::Request) -> tide::Result {
    let my = db::Device::for_user(USER)
        .fetch_all(&request.state().pool)
        .await
        .map_err(|err| dbg!(err))?;
    let unassinged = db::AliveDevice::unassinged()
        .fetch_all(&request.state().pool)
        .await
        .map_err(|err| dbg!(err))?;
    Ok(templates::IndexTemplate::new(my, unassinged).into())
}

#[derive(Deserialize)]
struct RegisterForm {
    macaddr: String,
}

pub async fn register(mut request: crate::Request) -> tide::Result {
    let form: RegisterForm = request.body_form().await?;
    unimplemented!();
}
