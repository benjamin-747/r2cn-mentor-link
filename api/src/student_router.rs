use axum::{extract::State, routing::post, Json, Router};
use common::{errors::CommonError, model::CommonResult};
use entity::sea_orm_active_enums::TaskStatus;
use service::ospp::{ValidateStudent, ValidateStudentRes};

use crate::{
    model::{student::SearchStuTask, task::Task},
    AppState,
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
    let res = service::ospp::validate_student(json.clone()).await;
    let res = match res {
        Ok(data) => {
            if data.success {
                state
                    .student_stg()
                    .insert_or_update_student(&json.login, data.clone())
                    .await
                    .unwrap();
            }
            CommonResult::success(Some(data))
        }
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn get_student_task(
    state: State<AppState>,
    Json(json): Json<SearchStuTask>,
) -> Result<Json<CommonResult<Task>>, CommonError> {
    let res = state
        .task_stg()
        .search_student_task(json.login, TaskStatus::processing_task_status())
        .await;
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
