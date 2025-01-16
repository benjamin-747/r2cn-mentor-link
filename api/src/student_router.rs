use std::env;

use axum::{extract::State, routing::post, Json, Router};
use chrono::{Datelike, Utc};
use common::{date::get_last_month, errors::CommonError, model::CommonResult};
use entity::sea_orm_active_enums::TaskStatus;

use crate::{
    model::{
        student::{SearchStuTask, ValidateStudent, ValidateStudentRes},
        task::Task,
    },
    AppState,
};

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/student",
        Router::new()
            .route("/task", post(get_student_task))
            .route("/validate", post(validate_student))
            .route("/calculate-bonus", post(calculate_bonus)),
    )
}

async fn validate_student(
    _: State<AppState>,
    Json(json): Json<ValidateStudent>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
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
    let res = serde_json::from_str::<ValidateStudentRes>(&body);
    let res = match res {
        Ok(data) => CommonResult::success(Some(data.student_exist)),
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

async fn calculate_bonus(state: State<AppState>) -> Result<Json<CommonResult<()>>, CommonError> {
    let now = Utc::now();
    let last_month = get_last_month(now.naive_utc().into());

    let month_score = state
        .score_stg()
        .list_score_by_month(now.year(), now.month() as i32)
        .await
        .unwrap();
    let students: Vec<String> = month_score.iter().map(|x| x.github_login.clone()).collect();

    state.score_stg().calculate_bonus(month_score).await;
    let last_month_score = state
        .score_stg()
        .list_score_by_month(last_month.0, last_month.1)
        .await
        .unwrap();
    state
        .score_stg()
        .calculate_unactive_bonus(last_month_score, students)
        .await
        .unwrap();

    Ok(Json(CommonResult::success(None)))
}
