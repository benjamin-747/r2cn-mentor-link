use entity::{sea_orm_active_enums::TaskStatus, task};
use sea_orm::{ActiveValue::NotSet, Set};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewTask {
    pub github_repo_id: i64,
    pub github_issue_id: i64,
    pub score: i32,
    pub mentor_github_id: i64,
}

impl From<NewTask> for task::ActiveModel {
    fn from(value: NewTask) -> Self {
        Self {
            id: NotSet,
            github_repo_id: Set(value.github_repo_id),
            github_issue_id: Set(value.github_issue_id),
            score: Set(value.score),
            task_status: Set(TaskStatus::Open),
            mentor_github_id: Set(value.mentor_github_id),
            student_github_id: NotSet,
            create_at: Set(chrono::Utc::now().naive_utc()),
            update_at: Set(chrono::Utc::now().naive_utc()),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub github_repo_id: i64,
    pub github_issue_id: i64,
    pub score: i32,
    pub task_status: TaskStatus,
    pub student_github_id: Option<i64>,
    pub mentor_github_id: i64,
}

impl From<task::Model> for Task {
    fn from(value: task::Model) -> Self {
        Self {
            id: value.id,
            github_repo_id: value.github_repo_id,
            github_issue_id: value.github_issue_id,
            score: value.score,
            task_status: value.task_status,
            student_github_id: value.student_github_id,
            mentor_github_id: value.mentor_github_id,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchTask {
    pub github_repo_id: i64,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidateStudent {
    pub git_email: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidateStudentRes {
    pub code: i32,
    pub err_code: i32,
    #[serde(alias = "studentExist")]
    pub student_exist: bool,
    pub message: String,
}
