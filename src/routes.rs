use crate::db;
use crate::templates;

pub async fn healthz(_request: crate::Request) -> tide::Result {
    Ok("ok".into())
}

pub async fn index(request: crate::Request) -> tide::Result {
    let my = db::Entry::for_user("foosinn")
        .fetch_all(&request.state().pool)
        .await
        .map_err(|err| dbg!(err))?;
    let unassinged = db::AliveDevice::unassinged()
        .fetch_all(&request.state().pool)
        .await
        .map_err(|err| dbg!(err))?;
    Ok(templates::IndexTemplate::new(my, unassinged).into())
}
