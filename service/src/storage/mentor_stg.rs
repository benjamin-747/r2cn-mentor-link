use std::sync::Arc;

use chrono::NaiveDateTime;
use entity::mentor;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub enum MentorStatus {
    Active,
    #[default]
    Inactive,
}

impl From<String> for MentorStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "active" => MentorStatus::Active,
            "inactive" => MentorStatus::Inactive,
            _ => MentorStatus::Inactive, // default
        }
    }
}

impl From<MentorStatus> for String {
    fn from(v: MentorStatus) -> Self {
        match v {
            MentorStatus::Active => "active".to_string(),
            MentorStatus::Inactive => "inactive".to_string(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct MentorRes {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub github_login: String,
    pub status: MentorStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<mentor::Model> for MentorRes {
    fn from(value: mentor::Model) -> Self {
        MentorRes {
            id: value.id,
            name: value.name,
            email: value.email,
            github_login: value.github_login,
            status: MentorStatus::from(value.status),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct MentorStorage {
    connection: Arc<DatabaseConnection>,
}

impl MentorStorage {
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub async fn new(connection: Arc<DatabaseConnection>) -> Self {
        MentorStorage { connection }
    }

    pub async fn get_mentor_by_login(
        &self,
        login: &str,
    ) -> Result<Option<mentor::Model>, anyhow::Error> {
        let record = mentor::Entity::find()
            .filter(mentor::Column::GithubLogin.eq(login))
            .one(self.get_connection())
            .await?;
        Ok(record)
    }

    pub async fn get_mentors_by_logins(
        &self,
        logins: Vec<String>,
    ) -> Result<Vec<mentor::Model>, anyhow::Error> {
        let mentors = mentor::Entity::find()
            .filter(mentor::Column::GithubLogin.is_in(logins))
            .all(self.get_connection())
            .await?;

        Ok(mentors)
    }

    pub async fn new_mentor(
        &self,
        active_model: mentor::ActiveModel,
    ) -> Result<mentor::Model, anyhow::Error> {
        let login = active_model.github_login.clone().unwrap();

        if self.get_mentor_by_login(&login).await?.is_some() {
            return Err(anyhow::anyhow!("mentor already exists: {}", login));
        }

        let mentor = active_model.insert(self.get_connection()).await?;

        Ok(mentor)
    }

    pub async fn change_mentor_status(
        &self,
        login: &str,
        status: MentorStatus,
    ) -> Result<mentor::Model, anyhow::Error> {
        let model = self.get_mentor_by_login(login).await?.ok_or_else(|| {
            DbErr::RecordNotFound(format!("Mentor not found for github_login {}", login))
        })?;

        let mut active: mentor::ActiveModel = model.into();
        active.status = Set(status.into());

        let updated = active.update(self.get_connection()).await?;
        Ok(updated)
    }
}
