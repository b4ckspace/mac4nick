use crate::AxumAppState;
use crate::db;
use crate::forms::ChangeForm;
use crate::helpers;
use crate::middleware::ForwardAuth;
use crate::templates::IndexTemplate;
use anyhow::Context;
use axum::{
    Form,
    extract::State,
    response::{Html, IntoResponse, Redirect, Result},
};
use axum_messages::Messages;

pub async fn healthz() -> impl IntoResponse {
    "ok"
}

pub async fn index(
    State(state): AxumAppState,
    messages: Messages,
    ForwardAuth(nickname): ForwardAuth,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let my = db::Device::for_user(&state, &nickname)
        .await
        .context("unable to fetch user from db")?;
    let unassinged = db::AliveDevice::unassinged(&state)
        .await
        .context("unable to find device")?;
    Ok::<Html<String>, helpers::AppError>(Html(
        IndexTemplate::new(
            nickname.to_string(),
            my,
            unassinged,
            messages
                .into_iter()
                .map(|msg| (msg.level, msg.message.to_string()))
                .collect(),
        )
        .to_string(),
    ))
}

pub async fn change(
    State(state): AxumAppState,
    messages: Messages,
    Form(form): Form<ChangeForm>,
) -> Result<impl IntoResponse, ()> {
    let message = form.handle(&state).await;
    messages.push(message.0, message.1, None);
    Ok(Redirect::to("/"))
}
