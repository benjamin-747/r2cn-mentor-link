use std::env;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use common::{errors::CommonError, model::CommonResult};
use entity::sea_orm_active_enums::TaskStatus;


use crate::{
    model::task::{NewTask, SearchTask, Task, ValidateStudent, ValidateStudentRes},
    AppState,
};

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/task",
        Router::new()
            .route("/new", post(new_task))
            .route("/issue/{:github_issue_id}", get(get_task))
            .route("/search", post(search_with_status))
            .route("/student/validate", post(validate_student)),
    )
}

async fn new_task(
    state: State<AppState>,
    Json(json): Json<NewTask>,
) -> Result<Json<CommonResult<Task>>, CommonError> {
    let active_model = json.into();
    let res = state.task_stg().new_task(active_model).await.unwrap();
    Ok(Json(CommonResult::success(Some(res.into()))))
}

async fn get_task(
    state: State<AppState>,
    Path(github_issue_id): Path<i64>,
) -> Result<Json<CommonResult<Task>>, CommonError> {
    let res = state
        .task_stg()
        .search_task_with_issue_id(github_issue_id)
        .await
        .unwrap();

    let res: CommonResult<Task> = match res {
        Some(model) => CommonResult::success(Some(model.into())),
        None => CommonResult::failed("Task Not Found"),
    };
    Ok(Json(res))
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

async fn search_with_status(
    state: State<AppState>,
    Json(json): Json<SearchTask>,
) -> Result<Json<CommonResult<Vec<Task>>>, CommonError> {
    let res = state
        .task_stg()
        .search_task_with_status(json.github_repo_id, processing_task_status())
        .await;
    let res = match res {
        Ok(model) => {
            let data = model.into_iter().map(|model| model.into()).collect();
            CommonResult::success(Some(data))
        }
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
