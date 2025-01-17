use std::env;

use axum::{extract::State, routing::post, Json, Router};
use common::{errors::CommonError, model::CommonResult};
use entity::sea_orm_active_enums::TaskStatus;

use crate::{
    model::{
        student::{OsppValidateStudentRes, SearchStuTask, ValidateStudent, ValidateStudentRes},
        task::Task,
    },
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
    _: State<AppState>,
    Json(json): Json<ValidateStudent>,
) -> Result<Json<CommonResult<ValidateStudentRes>>, CommonError> {
    //call ospp api check status
    let client = reqwest::Client::new();
    let api_host = env::var("OSPP_API_ENDPOINT").unwrap();
    let res = client
        .get(format!("{}/api/r2cnStudent/{}", api_host, json.login))
        .send()
        .await
        .unwrap();
    let body = res.text().await.unwrap();
    tracing::debug!("response body:{:?}", body);
    let res = serde_json::from_str::<OsppValidateStudentRes>(&body);
    let res = match res {
        Ok(data) => CommonResult::success(Some(ValidateStudentRes {
            success: data.student_exist,
            student_name: data.su_student_name,
        })),
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
