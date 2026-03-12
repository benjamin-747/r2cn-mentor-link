use axum::{Json, Router, extract::State, routing::post};
use common::{errors::CommonError, model::CommonResult};

use crate::email::EmailSender;
use crate::{AppState, model::email::EmailAnnouncement};

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/email",
        Router::new().route("/announcement", post(announcement)),
    )
}

async fn announcement(
    state: State<AppState>,
    Json(json): Json<EmailAnnouncement>,
) -> Result<Json<CommonResult<()>>, CommonError> {
    let res = EmailSender::notice_all_email(state, &json.temp_id).await;
    let res = match res {
        Ok(_) => CommonResult::success(Some(())),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}
