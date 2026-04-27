use axum::{Json, Router, extract::State, routing::post};
use common::{errors::CommonError, model::CommonResult};

use crate::{
    AppState,
    application::student,
    model::{student::SearchStuTask, task::Task},
};

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/student",
        Router::new().route("/task", post(get_student_task)),
    )
}

async fn get_student_task(
    state: State<AppState>,
    Json(json): Json<SearchStuTask>,
) -> Result<Json<CommonResult<Task>>, CommonError> {
    let student_id = state
        .student_stg()
        .get_student_id_by_login(&json.login)
        .await
        .map_err(|err| CommonError::InvalidInput(err.to_string()))?;
    let Some(student_id) = student_id else {
        return Ok(Json(CommonResult::failed("Student not found")));
    };

    let res = student::get_student_processing_task(state, student_id).await;
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
