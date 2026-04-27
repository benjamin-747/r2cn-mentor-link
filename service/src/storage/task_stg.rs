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
        issue_id: i64,
        issue_title: String,
        score: i32,
    ) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.score = Set(score);
        task.issue_title = Set(issue_title);
        task.update_at = Set(Utc::now().naive_utc());
        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn search_task_with_issue_id(
        &self,
        issue_id: i64,
    ) -> Result<Option<task::Model>, anyhow::Error> {
        let task = task::Entity::find()
            .filter(task::Column::IssueId.eq(issue_id))
            .one(self.get_connection())
            .await?;
        Ok(task)
    }

    pub async fn sum_all_task_score(&self) -> Result<i64, anyhow::Error> {
        let tasks = task::Entity::find().all(self.get_connection()).await?;
        Ok(tasks.into_iter().map(|t| i64::from(t.score)).sum())
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
            .order_by_asc(task::Column::StudentId)
            .all(self.get_connection())
            .await?;
        Ok(task)
    }

    pub async fn search_task_with_status(
        &self,
        repo_id: i64,
        mentor_login: String,
        status: Vec<TaskStatus>,
    ) -> Result<Vec<task::Model>, anyhow::Error> {
        let tasks: Vec<task::Model> = task::Entity::find()
            .filter(task::Column::RepoId.eq(repo_id))
            .filter(task::Column::MentorLogin.eq(mentor_login))
            .filter(task::Column::TaskStatus.is_in(status))
            .all(self.get_connection())
            .await?;
        Ok(tasks)
    }

    pub async fn search_student_task(
        &self,
        student_id: String,
        status: Vec<TaskStatus>,
    ) -> Result<Option<task::Model>, anyhow::Error> {
        let tasks = task::Entity::find()
            .filter(task::Column::StudentId.eq(student_id))
            .filter(task::Column::TaskStatus.is_in(status))
            .one(self.get_connection())
            .await?;
        Ok(tasks)
    }

    pub async fn get_student_tasks_with_status_in_month(
        &self,
        student_id: &str,
        status: Vec<TaskStatus>,
        year: i32,
        month: i32,
    ) -> Result<Vec<task::Model>, anyhow::Error> {
        let tasks = task::Entity::find()
            .filter(task::Column::StudentId.eq(student_id))
            .filter(task::Column::TaskStatus.is_in(status))
            .filter(task::Column::FinishYear.eq(year))
            .filter(task::Column::FinishMonth.eq(month))
            .all(self.get_connection())
            .await?;

        Ok(tasks)
    }

    pub async fn request_assign(
        &self,
        issue_id: i64,
        student_id: String,
    ) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.student_id = Set(Some(student_id));
        task.task_status = Set(TaskStatus::RequestAssign);
        task.update_at = Set(Utc::now().naive_utc());

        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn release_task(&self, issue_id: i64) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.student_id = Set(None);
        task.task_status = Set(TaskStatus::Open);
        task.update_at = Set(Utc::now().naive_utc());
        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn intern_approve(&self, issue_id: i64) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.task_status = Set(TaskStatus::Assigned);
        task.update_at = Set(Utc::now().naive_utc());
        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn request_complete(&self, issue_id: i64) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.task_status = Set(TaskStatus::RequestFinish);
        task.update_at = Set(Utc::now().naive_utc());
        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn intern_done(&self, issue_id: i64) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                issue_id
            )))?;
        let mut task: task::ActiveModel = task.into();
        task.task_status = Set(TaskStatus::Finished);
        task.finish_year = Set(Some(Utc::now().year()));
        task.finish_month = Set(Some(Utc::now().month() as i32));
        task.update_at = Set(Utc::now().naive_utc());
        Ok(task.update(self.get_connection()).await?)
    }

    pub async fn intern_close(&self, issue_id: i64) -> Result<task::Model, anyhow::Error> {
        let task = self
            .search_task_with_issue_id(issue_id)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Task not found for issue_id {}",
                issue_id
            )))?;
        if task.task_status != TaskStatus::Finished {
            let task: task::ActiveModel = task.clone().into();
            task.delete(self.get_connection()).await?;
        }
        Ok(task)
    }
}
