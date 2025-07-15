use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use chrono::{Datelike, Utc};
use common::{errors::CommonError, model::CommonResult};
use entity::{monthly_score, sea_orm_active_enums::TaskStatus, task};
use sea_orm::{Set, TryIntoModel};
use service::model::score::ScoreDto;

use crate::{
    AppState,
    model::{
        score::NewScore,
        task::{CommandRequest, NewTask, SearchTask, Task, UpdateScoreRequest},
    },
};

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/task",
        Router::new()
            .route("/new", post(new_task))
            .route("/update-score", post(update_task_score))
            .route("/issue/{:github_issue_id}", get(get_task))
            .route("/search", post(search_with_status))
            .route("/request-assign", post(request_assign))
            .route("/intern-approve", post(intern_approve))
            .route("/release", post(release_task))
            .route("/request-complete", post(request_complete))
            .route("/intern-done", post(intern_done))
            .route("/intern-close", post(intern_close)),
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

async fn update_task_score(
    state: State<AppState>,
    Json(json): Json<UpdateScoreRequest>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
    let res = state
        .task_stg()
        .update_score(json.github_issue_id, json.score)
        .await;
    let res = match res {
        Ok(_) => CommonResult::success(Some(true)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
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

async fn search_with_status(
    state: State<AppState>,
    Json(json): Json<SearchTask>,
) -> Result<Json<CommonResult<Vec<Task>>>, CommonError> {
    let res = state
        .task_stg()
        .search_task_with_status(
            json.github_repo_id,
            json.github_mentor_login,
            TaskStatus::processing_task_status(),
        )
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

async fn request_assign(
    state: State<AppState>,
    Json(json): Json<CommandRequest>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
    let res = state
        .task_stg()
        .request_assign(json.github_issue_id, json.student_login.unwrap())
        .await;

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
    let res = state.task_stg().intern_approve(json.github_issue_id).await;

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
    let res = state.task_stg().release_task(json.github_issue_id).await;

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
    let res = state
        .task_stg()
        .request_complete(json.github_issue_id)
        .await;

    let res = match res {
        Ok(_) => CommonResult::success(Some(true)),
        Err(err) => CommonResult::failed(&err.to_string()),
    };
    Ok(Json(res))
}

async fn intern_done(
    state: State<AppState>,
    Json(json): Json<CommandRequest>,
) -> Result<Json<CommonResult<task::Model>>, CommonError> {
    let task = state
        .task_stg()
        .intern_done(json.github_issue_id)
        .await
        .unwrap();
    let score_stg = state.score_stg();
    let date = Utc::now();
    let student_login = task.student_github_login.clone().unwrap();
    let current_score = score_stg
        .get_score(date.year(), date.month() as i32, &student_login)
        .await
        .unwrap();
    let balance = if let Some(score) = current_score {
        let sum_score = score.new_score + task.score;
        let mut a_model: monthly_score::ActiveModel = score.clone().into();
        a_model.new_score = Set(sum_score);
        score_stg.update_score(a_model.clone()).await.unwrap();
        let score_dto: ScoreDto = a_model.try_into_model().unwrap().into();
        score_dto.score_balance()
    } else {
        let student = state
            .student_stg()
            .get_student_by_login(&student_login)
            .await
            .unwrap();
        let mut new_score = NewScore {
            score: task.score,
            github_login: student_login.clone(),
            student_name: String::new(),
            carryover_score: 0,
        };
        if let Some(student) = student {
            new_score.student_name = student.student_name;
        };
        let last_score = score_stg
            .get_latest_score_by_login(&student_login)
            .await
            .unwrap();
        if let Some(last_score) = last_score {
            let last_score: ScoreDto = last_score.into();
            new_score.carryover_score = last_score.score_balance();
        }
        score_stg
            .insert_score(new_score.clone().into())
            .await
            .unwrap();
        new_score.carryover_score + new_score.score
    };
    let stu_stg = state.student_stg();
    let student = stu_stg
        .get_student_by_login(&task.mentor_github_login)
        .await
        .unwrap();

    if let Some(student) = student {
        tracing::info!(
            "{} new_score: {}, total_score:{}",
            student.email,
            task.score,
            balance
        )
        //     let mut email_context = tera::Context::new();
        //     email_context.insert("new_score", &task.score);
        //     email_context.insert("total_score", &balance);
        //     email_context.insert("task_link", "");
        //     email_context.insert("task_title", &task.github_issue_title);
        //     let sender = EmailSender::new(
        //         "score_count_email.html",
        //         "R2CN任务完成",
        //         email_context,
        //         &student.email,
        //     );
        //     sender.send();
    }
    Ok(Json(CommonResult::success(Some(task))))
}

async fn intern_close(
    state: State<AppState>,
    Json(json): Json<CommandRequest>,
) -> Result<Json<CommonResult<bool>>, CommonError> {
    state
        .task_stg()
        .intern_close(json.github_issue_id)
        .await
        .unwrap();
    Ok(Json(CommonResult::success(Some(true))))
}
