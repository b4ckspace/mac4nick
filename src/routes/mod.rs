use crate::templates;

pub async fn healthz(_request: crate::Request) -> tide::Result {
    Ok("ok".into())
}

pub async fn index(request: crate::Request) -> tide::Result {
    Ok(templates::IndexTemplate::new().into())
}
