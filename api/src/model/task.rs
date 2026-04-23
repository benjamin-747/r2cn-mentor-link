use entity::{sea_orm_active_enums::TaskStatus, task};
use sea_orm::{ActiveValue::NotSet, Set};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackendMeta {
    #[serde(default)]
    pub scm_provider: Option<String>,
    #[serde(default)]
    pub external_ref: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewTask {
    pub owner: String,
    pub repo: String,
    pub issue_number: i32,
    pub repo_id: i64,
    pub issue_id: i64,
    pub score: i32,
    pub mentor_login: String,
    pub issue_title: String,
    pub issue_link: String,
    #[serde(flatten)]
    pub backend_meta: BackendMeta,
}

impl From<NewTask> for task::ActiveModel {
    fn from(value: NewTask) -> Self {
        Self {
            id: NotSet,
            owner: Set(value.owner),
            repo: Set(value.repo),
            issue_number: Set(value.issue_number),
            repo_id: Set(value.repo_id),
            issue_id: Set(value.issue_id),
            score: Set(value.score),
            task_status: Set(TaskStatus::Open),
            finish_year: NotSet,
            finish_month: NotSet,
            mentor_login: Set(value.mentor_login),
            student_id: NotSet,
            create_at: Set(chrono::Utc::now().naive_utc()),
            update_at: Set(chrono::Utc::now().naive_utc()),
            issue_title: Set(value.issue_title),
            issue_link: Set(value.issue_link),
            scm_provider: Set(value
                .backend_meta
                .scm_provider
                .unwrap_or_else(|| "github".to_string())),
            external_ref: Set(value.backend_meta.external_ref),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub owner: String,
    pub repo: String,
    pub issue_number: i32,
    pub repo_id: i64,
    pub issue_id: i64,
    pub score: i32,
    pub task_status: TaskStatus,
    pub student_id: Option<String>,
    pub student_login: Option<String>,
    pub mentor_login: String,
}

impl From<task::Model> for Task {
    fn from(value: task::Model) -> Self {
        Self {
            id: value.id,
            owner: value.owner,
            repo: value.repo,
            issue_number: value.issue_number,
            repo_id: value.repo_id,
            issue_id: value.issue_id,
            score: value.score,
            task_status: value.task_status,
            student_id: value.student_id,
            student_login: None,
            mentor_login: value.mentor_login,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchTask {
    pub repo_id: i64,
    pub mentor_login: String,
    #[serde(flatten)]
    pub backend_meta: BackendMeta,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandRequest {
    pub issue_id: i64,
    pub student_id: Option<String>,
    pub student_login: Option<String>,
    #[serde(flatten)]
    pub backend_meta: BackendMeta,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateScoreRequest {
    pub issue_id: i64,
    pub issue_title: String,
    pub score: i32,
    #[serde(flatten)]
    pub backend_meta: BackendMeta,
}
