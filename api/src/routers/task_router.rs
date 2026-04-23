use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use common::{errors::CommonError, model::CommonResult};
use entity::task as task_entity;

use crate::{
    AppState,
    application::task,
    model::task::{CommandRequest, NewTask, SearchTask, Task, UpdateScoreRequest},
};

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/task",
        Router::new()
            .route("/new", post(new_task))
            .route("/update-score", post(update_task_score))
            .route("/issue/{:issue_id}", get(get_task))
            .route("/search", post(search_with_status))
            .route("/request-assign", post(request_assign))
            .route("/intern-approve", post(intern_approve))
            .route("/release", post(release_task))
            .route("/request-complete", post(request_complete))
            .route("/intern-done", post(intern_done))
            .route("/intern-close", post(intern_close)),
    )
}

async fn build_task_response(
    state: &AppState,
    model: task_entity::Model,
) -> Result<Task, CommonError> {
    let student_login = if let Some(student_id) = &model.student_id {
        state
            .student_stg()
            .get_student_login_by_student_id(student_id)
            .await
            .map_err(|err| CommonError::InvalidInput(err.to_string()))?
    } else {
        None
    };

    let mut task: Task = model.into();
    task.student_login = student_login;
    Ok(task)
}

async fn new_task(
    state: State<AppState>,
    Json(json): Json<NewTask>,
) -> Result<Json<CommonResult<Task>>, CommonError> {
    let active_model = json.into();
    let res = state.task_stg().new_task(active_model).await.unwrap();
    let res = build_task_response(&state, res).await?;
    Ok(Json(CommonResult::success(Some(res))))
}

async fn update_task_score(
    state: State<AppState>,
    Json(json): Json<UpdateScoreRequest>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
    let res = task::update_task_score(&state, json.issue_id, json.issue_title, json.score).await;
    let res = match res {
        Ok(_) => CommonResult::success(Some(true)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn get_task(
    state: State<AppState>,
    Path(issue_id): Path<i64>,
) -> Result<Json<CommonResult<Task>>, CommonError> {
    let res = task::get_task_by_issue_id(&state, issue_id)
        .await
        .map_err(|err| CommonError::InvalidInput(err.to_string()))?;

    let res: CommonResult<Task> = match res {
        Some(model) => CommonResult::success(Some(build_task_response(&state, model).await?)),
        None => CommonResult::failed("Task Not Found"),
    };
    Ok(Json(res))
}

async fn search_with_status(
    state: State<AppState>,
    Json(json): Json<SearchTask>,
) -> Result<Json<CommonResult<Vec<Task>>>, CommonError> {
    let res = task::search_processing_tasks(&state, json.repo_id, json.mentor_login).await;
    let res = match res {
        Ok(model) => {
            let mut data = Vec::with_capacity(model.len());
            for task_model in model {
                data.push(build_task_response(&state, task_model).await?);
            }
            CommonResult::success(Some(data))
        }
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn request_assign(
    state: State<AppState>,
    Json(json): Json<CommandRequest>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
    let student_id = if let Some(student_id) = json.student_id {
        student_id
    } else if let Some(student_login) = json.student_login {
        state
            .student_stg()
            .get_student_id_by_login(&student_login)
            .await
            .map_err(|err| CommonError::InvalidInput(err.to_string()))?
            .ok_or_else(|| CommonError::InvalidInput("Student not found".to_string()))?
    } else {
        return Ok(Json(CommonResult::failed(
            "student_id or student_login is required",
        )));
    };

    let res = task::request_assign(&state, json.issue_id, student_id).await;

    let res = match res {
        Ok(_) => CommonResult::success(Some(true)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn intern_approve(
    state: State<AppState>,
    Json(json): Json<CommandRequest>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
    let res = task::intern_approve(state, json.issue_id).await;
    let res = match res {
        Ok(_) => CommonResult::success(Some(true)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn release_task(
    state: State<AppState>,
    Json(json): Json<CommandRequest>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
    let res = task::release_task(state, json.issue_id).await;
    let res = match res {
        Ok(_) => CommonResult::success(Some(true)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn request_complete(
    state: State<AppState>,
    Json(json): Json<CommandRequest>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
    let res = task::request_complete(&state, json.issue_id).await;

    let res = match res {
        Ok(_) => CommonResult::success(Some(true)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn intern_done(
    state: State<AppState>,
    Json(json): Json<CommandRequest>,
) -> Result<Json<CommonResult<task_entity::Model>>, CommonError> {
    let task_model = task::intern_done(state, json.issue_id)
        .await
        .map_err(|err| CommonError::InvalidInput(err.to_string()))?;
    Ok(Json(CommonResult::success(Some(task_model))))
}

async fn intern_close(
    state: State<AppState>,
    Json(json): Json<CommandRequest>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
    task::intern_close(state, json.issue_id)
        .await
        .map_err(|err| CommonError::InvalidInput(err.to_string()))?;
    Ok(Json(CommonResult::success(Some(true))))
}
