use crate::db;
use crate::forms;
use crate::templates;
use crate::AppSession;
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

    let mut session = AppSession::from(&mut request);
    let messages = session.pop_messages();
    session.commit(&mut request);

    Ok(templates::IndexTemplate::new(my, unassinged, messages).into())
}

pub async fn register(mut request: crate::Request) -> tide::Result {
    let form: forms::RegisterForm = request.body_form().await?;
    let message = form.handle(&request).await;
    AppSession::from(&mut request)
        .add_message(message)
        .commit(&mut request);
    Ok(Redirect::see_other("/").into())
}

pub async fn update(mut request: crate::Request) -> tide::Result {
    let form: forms::UpdateForm = request.body_form().await?;
    let message = form.handle(&request).await;
    AppSession::from(&mut request)
        .add_message(message)
        .commit(&mut request);
    Ok(Redirect::see_other("/").into())
}
