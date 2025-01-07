use std::env;

use axum::{extract::State, routing::post, Json, Router};
use common::{errors::CommonError, model::CommonResult};
use entity::sea_orm_active_enums::TaskStatus;

use crate::{
    model::task::{ValidateStudent, ValidateStudentRes, ValidateTask},
    AppState,
};

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/task/validate", post(validate_new_tasks))
        .route("/task/student/validate", post(validate_student))
}

async fn validate_student(
    _: State<AppState>,
    Json(json): Json<ValidateStudent>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
    //call ospp api check status
    let client = reqwest::Client::new();
    let api_host = env::var("OSPP_API_ENDPOINT").unwrap();
    let res = client
        .get(format!("{}/api/r2cnStudent/{}", api_host, json.git_email))
        .send()
        .await
        .unwrap();
    let body = res.text().await.unwrap();
    tracing::debug!("response body:{:?}", body);
    let res = serde_json::from_str::<ValidateStudentRes>(&body);
    let res = match res {
        Ok(data) => CommonResult::success(Some(data.student_exist)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn validate_new_tasks(
    state: State<AppState>,
    Json(json): Json<ValidateTask>,
) -> Result<Json<CommonResult<String>>, CommonError> {
    let res = state
        .task_stg()
        .search_task_with_status(json.github_repo_id, processing_task_status())
        .await;
    let res = match res {
        Ok(_) => CommonResult::success(None),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

pub fn processing_task_status() -> Vec<TaskStatus> {
    vec![
        TaskStatus::Open,
        TaskStatus::RequestAssign,
        TaskStatus::Assigned,
        TaskStatus::RequestFinish,
    ]
}
