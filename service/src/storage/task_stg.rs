use std::sync::Arc;

use chrono::{Datelike, Utc};
use entity::{sea_orm_active_enums::TaskStatus, task};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    Set,
};

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

    pub async fn new_task(
        &self,
        active_model: task::ActiveModel,
    ) -> Result<task::Model, anyhow::Error> {
        let task = active_model.insert(self.get_connection()).await?;
        Ok(task)
    }

    pub async fn update_score(
        &self,
        github_issue_id: i64,
        github_issue_title: String,
        score: i32,
    ) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(github_issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                github_issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.score = Set(score);
        task.github_issue_title = Set(github_issue_title);
        task.update_at = Set(Utc::now().naive_utc());
        Ok(task.update(self.get_connection()).await?)
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

    pub async fn search_finished_task_with_date(
        &self,
        finish_year: i32,
        finish_month: i32,
    ) -> Result<Vec<task::Model>, anyhow::Error> {
        let task = task::Entity::find()
            .filter(task::Column::FinishYear.eq(finish_year))
            .filter(task::Column::FinishMonth.eq(finish_month))
            .filter(task::Column::TaskStatus.eq(TaskStatus::Finished))
            .order_by_asc(task::Column::StudentGithubLogin)
            .all(self.get_connection())
            .await?;
        Ok(task)
    }

    pub async fn search_task_with_status(
        &self,
        github_repo_id: i64,
        github_mentor_login: String,
        status: Vec<TaskStatus>,
    ) -> Result<Vec<task::Model>, anyhow::Error> {
        let tasks: Vec<task::Model> = task::Entity::find()
            .filter(task::Column::GithubRepoId.eq(github_repo_id))
            .filter(task::Column::MentorGithubLogin.eq(github_mentor_login))
            .filter(task::Column::TaskStatus.is_in(status))
            .all(self.get_connection())
            .await?;
        Ok(tasks)
    }

    pub async fn search_student_task(
        &self,
        login: String,
        status: Vec<TaskStatus>,
    ) -> Result<Option<task::Model>, anyhow::Error> {
        let tasks = task::Entity::find()
            .filter(task::Column::StudentGithubLogin.eq(login))
            .filter(task::Column::TaskStatus.is_in(status))
            .one(self.get_connection())
            .await?;
        Ok(tasks)
    }

    pub async fn get_student_tasks_with_status_in_month(
        &self,
        login: &str,
        status: Vec<TaskStatus>,
        year: i32,
        month: i32,
    ) -> Result<Vec<task::Model>, anyhow::Error> {
        let tasks = task::Entity::find()
            .filter(task::Column::StudentGithubLogin.eq(login))
            .filter(task::Column::TaskStatus.is_in(status))
            .filter(task::Column::FinishYear.eq(year))
            .filter(task::Column::FinishMonth.eq(month))
            .all(self.get_connection())
            .await?;

        Ok(tasks)
    }

    pub async fn request_assign(
        &self,
        github_issue_id: i64,
        login: String,
    ) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(github_issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                github_issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.student_github_login = Set(Some(login));
        task.task_status = Set(TaskStatus::RequestAssign);
        task.update_at = Set(Utc::now().naive_utc());

        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn release_task(&self, github_issue_id: i64) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(github_issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                github_issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.student_github_login = Set(None);
        task.task_status = Set(TaskStatus::Open);
        task.update_at = Set(Utc::now().naive_utc());
        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn intern_approve(&self, github_issue_id: i64) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(github_issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                github_issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.task_status = Set(TaskStatus::Assigned);
        task.update_at = Set(Utc::now().naive_utc());
        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn request_complete(
        &self,
        github_issue_id: i64,
    ) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(github_issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                github_issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.task_status = Set(TaskStatus::RequestFinish);
        task.update_at = Set(Utc::now().naive_utc());
        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn intern_done(&self, github_issue_id: i64) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(github_issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                github_issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.task_status = Set(TaskStatus::Finished);
        task.finish_year = Set(Some(Utc::now().year()));
        task.finish_month = Set(Some(Utc::now().month() as i32));
        task.update_at = Set(Utc::now().naive_utc());
        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn intern_close(&self, github_issue_id: i64) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(github_issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                github_issue_id
            )))?;
        if task.task_status != TaskStatus::Finished {
            let task: task::ActiveModel = task.clone().into();
            task.delete(self.get_connection()).await?;
        }
        Ok(task)
    }
}
