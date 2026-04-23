use axum::{Json, Router, extract::State, routing::post};
use common::{errors::CommonError, model::CommonResult};
use service::storage::mentor_stg::MentorRes;

use crate::{
    AppState,
    application::mentor,
    model::mentor::{NewMentor, UpdateMentorStatusRequest},
};

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/mentor",
        Router::new()
            .route("/new-mentor", post(new_mentor))
            .route("/status", post(change_mentor_status)),
    )
}

async fn new_mentor(
    state: State<AppState>,
    Json(json): Json<NewMentor>,
) -> Result<Json<CommonResult<MentorRes>>, CommonError> {
    let res = mentor::new_mentor(&state, json).await;
    let res = match res {
        Ok(model) => CommonResult::success(Some(model)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn change_mentor_status(
    state: State<AppState>,
    Json(json): Json<UpdateMentorStatusRequest>,
) -> Result<Json<CommonResult<MentorRes>>, CommonError> {
    let res = mentor::change_mentor_status(&state, json).await;

    let res = match res {
        Ok(model) => CommonResult::success(Some(model)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}
