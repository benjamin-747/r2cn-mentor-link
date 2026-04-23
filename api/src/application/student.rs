use anyhow::Result;
use axum::extract::State;
use entity::sea_orm_active_enums::TaskStatus;
use service::ospp::{ValidateStudent, ValidateStudentRes};

use crate::AppState;

pub async fn validate_student(
    state: &AppState,
    payload: ValidateStudent,
) -> Result<ValidateStudentRes> {
    state
        .student_stg()
        .validate_student_by_login(&payload.login)
        .await
}

pub async fn get_student_processing_task(
    state: State<AppState>,
    student_id: String,
) -> Result<Option<entity::task::Model>> {
    state
        .task_stg()
        .search_student_task(student_id, TaskStatus::processing_task_status())
        .await
}
