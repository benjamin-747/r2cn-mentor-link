use anyhow::Result;
use axum::extract::State;
use entity::sea_orm_active_enums::TaskStatus;

use crate::AppState;

pub async fn get_student_processing_task(
    state: State<AppState>,
    student_id: String,
) -> Result<Option<entity::task::Model>> {
    state
        .task_stg()
        .search_student_task(student_id, TaskStatus::processing_task_status())
        .await
}
