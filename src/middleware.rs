use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

pub(crate) struct ForwardAuth(pub String);

impl<S> FromRequestParts<S> for ForwardAuth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(nickname) = parts
            .headers
            .get("X-Remote-User")
            .and_then(|value| value.to_str().ok())
        {
            return Ok(Self(nickname.to_owned()));
        };

        Err(StatusCode::UNAUTHORIZED)
    }
}
