use axum::Router;
use axum::routing::get;

use crate::AppState;

pub mod email_router;
pub mod score_router;
pub mod student_router;
pub mod task_router;

pub fn build_api_router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .merge(task_router::routers())
        .merge(student_router::routers())
        .merge(score_router::routers())
        .merge(email_router::routers())
}

async fn healthz() -> &'static str {
    "ok"
}
