use std::sync::Arc;

use chrono::NaiveDateTime;
use entity::{account, openatom_mentor, user};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct MentorRes {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub login: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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

    pub async fn get_approved_mentors(&self) -> Result<Vec<MentorRes>, anyhow::Error> {
        let records = openatom_mentor::Entity::find()
            .filter(openatom_mentor::Column::MentorStatus.eq("approved"))
            .all(self.get_connection())
            .await?;
        let mut result = Vec::with_capacity(records.len());
        for record in records {
            if let Some(profile) = self.resolve_mentor_profile(record).await? {
                result.push(profile);
            }
        }
        Ok(result)
    }

    pub async fn get_mentor_by_login(
        &self,
        login: &str,
    ) -> Result<Option<MentorRes>, anyhow::Error> {
        let account = account::Entity::find()
            .filter(account::Column::Login.eq(login))
            .one(self.get_connection())
            .await?;
        let Some(account) = account else {
            return Ok(None);
        };
        let mentor = openatom_mentor::Entity::find()
            .filter(openatom_mentor::Column::UserId.eq(account.user_id))
            .one(self.get_connection())
            .await?;
        let Some(mentor) = mentor else {
            return Ok(None);
        };
        self.resolve_mentor_profile(mentor).await
    }

    pub async fn get_mentors_by_logins(
        &self,
        logins: Vec<String>,
    ) -> Result<Vec<MentorRes>, anyhow::Error> {
        let mut mentors = Vec::new();
        for login in logins {
            if let Some(mentor) = self.get_mentor_by_login(&login).await? {
                mentors.push(mentor);
            }
        }
        Ok(mentors)
    }

    async fn resolve_mentor_profile(
        &self,
        mentor: openatom_mentor::Model,
    ) -> Result<Option<MentorRes>, anyhow::Error> {
        let user = user::Entity::find_by_id(mentor.user_id.clone())
            .one(self.get_connection())
            .await?;
        let Some(user) = user else {
            return Ok(None);
        };
        let account = account::Entity::find()
            .filter(account::Column::UserId.eq(mentor.user_id))
            .one(self.get_connection())
            .await?;
        let login = account.and_then(|a| a.login).unwrap_or_default();
        let name = user.name.unwrap_or_else(|| login.clone());

        Ok(Some(MentorRes {
            id: mentor.id,
            user_id: user.id,
            name,
            email: user.email,
            login,
            created_at: mentor.created_at,
            updated_at: mentor.updated_at,
        }))
    }
}
