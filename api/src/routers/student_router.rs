use axum::{Json, Router, extract::State, routing::post};
use common::{errors::CommonError, model::CommonResult};
use service::ospp::{ValidateStudent, ValidateStudentRes};

use crate::{
    AppState,
    application::student,
    model::{student::SearchStuTask, task::Task},
};

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/student",
        Router::new()
            .route("/task", post(get_student_task))
            .route("/validate", post(validate_student)),
    )
}

async fn validate_student(
    state: State<AppState>,
    Json(json): Json<ValidateStudent>,
) -> Result<Json<CommonResult<ValidateStudentRes>>, CommonError> {
    let res = student::validate_student(&state, json).await;
    let res = match res {
        Ok(data) => CommonResult::success(Some(data)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn get_student_task(
    state: State<AppState>,
    Json(json): Json<SearchStuTask>,
) -> Result<Json<CommonResult<Task>>, CommonError> {
    let res = student::get_student_processing_task(state, json.student_id).await;
    let res = match res {
        Ok(model) => {
            if let Some(model) = model {
                CommonResult::success(Some(model.into()))
            } else {
                CommonResult::success(None)
            }
        }
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}
