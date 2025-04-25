use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::CookieJar;

pub(crate) struct ForwardAuth(pub String);

impl<S> FromRequestParts<S> for ForwardAuth
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        if let Some(nickname) = jar.get("_forward_auth_name") {
            return Ok(ForwardAuth(nickname.value().to_owned()));
        }

        Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
    }
}
