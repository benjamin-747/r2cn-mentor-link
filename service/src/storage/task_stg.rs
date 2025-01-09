use std::sync::Arc;

use entity::{sea_orm_active_enums::TaskStatus, task};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

#[derive(Clone)]
pub struct TaskStorage {
    connection: Arc<DatabaseConnection>,
}

impl TaskStorage {
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub async fn new(connection: Arc<DatabaseConnection>) -> Self {
        TaskStorage { connection }
    }

    pub async fn new_task(&self, active_model: task::ActiveModel) -> Result<task::Model, anyhow::Error> {
        let task = active_model.insert(self.get_connection()).await?;
        Ok(task)
    }

    pub async fn search_task_with_issue_id(
        &self,
        github_issue_id: i64,
    ) -> Result<Option<task::Model>, anyhow::Error> {
        let task = task::Entity::find()
            .filter(task::Column::GithubIssueId.eq(github_issue_id))
            .one(self.get_connection())
            .await?;
        Ok(task)
    }

    pub async fn search_task_with_status(
        &self,
        github_repo_id: i64,
        status: Vec<TaskStatus>,
    ) -> Result<Vec<task::Model>, anyhow::Error> {
        let tasks: Vec<task::Model> = task::Entity::find()
            .filter(task::Column::GithubRepoId.eq(github_repo_id))
            .filter(task::Column::TaskStatus.is_in(status))
            .all(self.get_connection())
            .await?;
        Ok(tasks)
    }
}
