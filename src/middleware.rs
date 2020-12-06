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

pub struct ForwardAuth {
    default_nickname: String,
}

impl ForwardAuth {
    pub fn new(default_nickname: &str) -> Self {
        ForwardAuth {
            default_nickname: default_nickname.to_string(),
        }
    }
}

pub struct ForwardAuthInfo {
    pub nickname: String,
}

#[async_trait]
impl Middleware<State> for ForwardAuth {
    async fn handle(&self, mut request: Request, next: Next<'_, State>) -> Result {
        let nickname = match request.cookie("_forward_auth_name") {
            Some(cookie) => cookie.value().to_string(),
            None => self.default_nickname.clone(),
        };
        request.set_ext(ForwardAuthInfo { nickname });
        Ok(next.run(request).await)
    }

    fn name(&self) -> &str {
        "ForwardAuth"
    }
}
