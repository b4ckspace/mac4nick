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
