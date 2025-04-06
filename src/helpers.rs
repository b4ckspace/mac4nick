use axum::response::IntoResponse;
use axum::response::Response;

pub(crate) struct AppError(anyhow::Error);

// Convert from anyhow::Error to your wrapper
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError(err)
    }
}

// Implement IntoResponse for your wrapper
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::event!(tracing::Level::ERROR, "unexpectec error {:?}", self.0);
        (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        )
            .into_response()
    }
}
