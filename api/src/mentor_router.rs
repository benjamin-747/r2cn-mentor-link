use axum::{Json, Router, extract::State, routing::post};
use common::{errors::CommonError, model::CommonResult};
use service::storage::mentor_stg::MentorRes;

use crate::{
    AppState,
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
    let active_model = json.into();
    let res = state.mentor_stg().new_mentor(active_model).await.unwrap();
    Ok(Json(CommonResult::success(Some(res.into()))))
}

async fn change_mentor_status(
    state: State<AppState>,
    Json(json): Json<UpdateMentorStatusRequest>,
) -> Result<Json<CommonResult<MentorRes>>, CommonError> {
    let res = state
        .mentor_stg()
        .change_mentor_status(&json.login, json.status)
        .await;

    let res = match res {
        Ok(model) => CommonResult::success(Some(model.into())),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}
