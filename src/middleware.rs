use crate::Request;
use crate::State;
use tide::log;
use tide::utils::async_trait;
use tide::{Middleware, Next, Redirect, Result};

#[derive(Default)]
pub struct ErrorHandler {}

#[async_trait]
impl Middleware<State> for ErrorHandler {
    async fn handle(&self, request: Request, next: Next<'_, State>) -> Result {
        let mut resp = next.run(request).await;
        if let Some(err) = resp.take_error() {
            log::error!("middleware caught error", { error: err.to_string() });
            return Ok(Redirect::see_other("/").into());
        }
        Ok(resp)
    }
    fn name(&self) -> &str {
        "ErrorHandler"
    }
}

#[derive(Default)]
pub struct ForwardAuth {}

#[derive(Default)]
pub struct ForwardAuthInfo {
    pub nickname: String,
}

#[async_trait]
impl Middleware<State> for ForwardAuth {
    async fn handle(&self, mut request: Request, next: Next<'_, State>) -> Result {
        let mut nickname = "Anonymous".to_string();
        if let Some(cookie) = request.cookie("_forward_auth_name") {
            nickname = cookie.value().to_string();
        }
        request.set_ext(ForwardAuthInfo { nickname });
        Ok(next.run(request).await)
    }
}
