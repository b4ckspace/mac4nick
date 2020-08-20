use crate::db;
use crate::forms;
use crate::templates;
use crate::Session;
use crate::USER;
use tide::Redirect;

pub async fn healthz(_request: crate::Request) -> tide::Result {
    Ok("ok".into())
}

pub async fn index(mut request: crate::Request) -> tide::Result {
    let my = db::Device::for_user(USER)
        .fetch_all(&request.state().pool)
        .await
        .map_err(|err| dbg!(err))?;
    let unassinged = db::AliveDevice::unassinged()
        .fetch_all(&request.state().pool)
        .await
        .map_err(|err| dbg!(err))?;
    let messages = Session::from(&mut request).pop_messages();
    Ok(templates::IndexTemplate::new(my, unassinged, messages).into())
}

pub async fn register(mut request: crate::Request) -> tide::Result {
    let form: forms::RegisterForm = request.body_form().await?;
    let message = form.handle(&request).await;
    Session::from(&mut request).add_message(message);
    Ok(Redirect::see_other("/").into())
}

pub async fn update(mut request: crate::Request) -> tide::Result {
    let form: forms::UpdateForm = request.body_form().await?;
    let message = form.handle(&request).await;
    Session::from(&mut request).add_message(message);
    Ok(Redirect::see_other("/").into())
}

pub async fn delete(mut request: crate::Request) -> tide::Result {
    let form: forms::DeleteForm = request.body_form().await?;
    let message = form.handle(&request).await;
    Session::from(&mut request).add_message(message);
    Ok(Redirect::see_other("/").into())
}
