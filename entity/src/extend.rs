use crate::sea_orm_active_enums::TaskStatus;

impl TaskStatus {
    pub fn processing_task_status() -> Vec<TaskStatus> {
        vec![
            TaskStatus::Open,
            TaskStatus::RequestAssign,
            TaskStatus::Assigned,
            TaskStatus::RequestFinish,
        ]
    }

    pub fn finish_task_status() -> Vec<TaskStatus> {
        vec![TaskStatus::Finished]
    }
}
