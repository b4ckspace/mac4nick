use crate::db;
use crate::forms::ChangeForm;
use crate::middleware::ForwardAuthInfo;
use crate::session::Session;
use crate::templates::IndexTemplate;
use tide::Redirect;

pub async fn healthz(_request: crate::Request) -> tide::Result {
    Ok("ok".into())
}

pub async fn index(mut request: crate::Request) -> tide::Result {
    let forward_auth: &ForwardAuthInfo = request.ext().unwrap();
    let nickname = forward_auth.nickname.clone();
    let my = db::Device::for_user(&nickname)
        .fetch_all(&request.state().pool)
        .await?;
    let unassinged = db::AliveDevice::unassinged()
        .fetch_all(&request.state().pool)
        .await?;
    let messages = Session::from(&mut request).pop_messages();
    Ok(IndexTemplate::new(nickname, my, unassinged, messages).into())
}

pub async fn change(mut request: crate::Request) -> tide::Result {
    let form: ChangeForm = request.body_form().await?;
    let message = form.handle(&request).await;
    Session::from(&mut request).add_message(message);
    Ok(Redirect::see_other("/").into())
}
