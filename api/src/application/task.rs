use anyhow::{Result, anyhow};
use axum::extract::State;
use chrono::{Datelike, Utc};
use entity::{monthly_score, sea_orm_active_enums::TaskStatus, task};
use sea_orm::{Set, TryIntoModel};
use service::model::score::ScoreDto;

use crate::{AppState, email::EmailSender, model::score::NewScore};

pub async fn update_task_score(
    state: &AppState,
    issue_id: i64,
    issue_title: String,
    score: i32,
) -> Result<bool> {
    state
        .task_stg()
        .update_score(issue_id, issue_title, score)
        .await?;
    Ok(true)
}

pub async fn get_task_by_issue_id(state: &AppState, issue_id: i64) -> Result<Option<task::Model>> {
    state.task_stg().search_task_with_issue_id(issue_id).await
}

pub async fn search_processing_tasks(
    state: &AppState,
    repo_id: i64,
    mentor_login: String,
) -> Result<Vec<task::Model>> {
    state
        .task_stg()
        .search_task_with_status(repo_id, mentor_login, TaskStatus::processing_task_status())
        .await
}

pub async fn request_assign(state: &AppState, issue_id: i64, student_id: String) -> Result<bool> {
    state
        .task_stg()
        .request_assign(issue_id, student_id)
        .await?;
    Ok(true)
}

pub async fn intern_approve(state: State<AppState>, issue_id: i64) -> Result<bool> {
    let task = state.task_stg().intern_approve(issue_id).await?;
    tokio::spawn(async move { EmailSender::assigned_email(state, task).await });
    Ok(true)
}

pub async fn release_task(state: State<AppState>, issue_id: i64) -> Result<bool> {
    let task = state
        .task_stg()
        .search_task_with_issue_id(issue_id)
        .await?
        .ok_or(anyhow!("Task not found for issue_id {}", issue_id))?;
    let moved_state = state.clone();
    tokio::spawn(async move { EmailSender::failed_email(moved_state, task).await });
    state.task_stg().release_task(issue_id).await?;
    Ok(true)
}

pub async fn request_complete(state: &AppState, issue_id: i64) -> Result<bool> {
    state.task_stg().request_complete(issue_id).await?;
    Ok(true)
}

pub async fn intern_done(state: State<AppState>, issue_id: i64) -> Result<task::Model> {
    let task = state.task_stg().intern_done(issue_id).await?;
    let score_stg = state.score_stg();
    let date = Utc::now();
    let student_id = task
        .student_id
        .clone()
        .ok_or(anyhow!("Student id missing for issue_id {}", issue_id))?;
    let current_score = score_stg
        .get_score(date.year(), date.month() as i32, &student_id)
        .await?;
    let balance = if let Some(score) = current_score {
        let sum_score = score.new_score + task.score;
        let mut a_model: monthly_score::ActiveModel = score.clone().into();
        a_model.new_score = Set(sum_score);
        score_stg.update_score(a_model.clone()).await?;
        let score_dto: ScoreDto = a_model.try_into_model()?.into();
        score_dto.score_balance()
    } else {
        let student = state
            .student_stg()
            .get_student_by_student_id(&student_id)
            .await?;
        let mut new_score = NewScore {
            score: task.score,
            student_id: student_id.clone(),
            student_name: String::new(),
            carryover_score: 0,
        };
        if let Some(student) = student {
            new_score.student_name = student.student_name;
        };
        let last_score = score_stg
            .get_latest_score_by_student_id(&student_id)
            .await?;
        if let Some(last_score) = last_score {
            let last_score: ScoreDto = last_score.into();
            new_score.carryover_score = last_score.score_balance();
        }
        score_stg.insert_score(new_score.clone().into()).await?;
        new_score.carryover_score + new_score.score
    };

    let moved_task = task.clone();
    tokio::spawn(async move { EmailSender::complete_email(state, moved_task, balance).await });
    Ok(task)
}

pub async fn intern_close(state: State<AppState>, issue_id: i64) -> Result<bool> {
    let task = state.task_stg().intern_close(issue_id).await?;
    tokio::spawn(async move { EmailSender::failed_email(state, task).await });
    Ok(true)
}
