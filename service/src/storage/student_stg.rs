use std::sync::Arc;

use chrono::NaiveDate;
use entity::{account, openatom_student, user};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StudentProfile {
    pub student_id: String,
    pub user_id: String,
    pub account_id: Option<String>,
    pub student_name: String,
    pub email: String,
    pub contract_end_date: Option<NaiveDate>,
}

#[derive(Clone)]
pub struct StudentStorage {
    connection: Arc<DatabaseConnection>,
}

impl StudentStorage {
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub async fn new(connection: Arc<DatabaseConnection>) -> Self {
        StudentStorage { connection }
    }

    pub async fn get_active_students(&self) -> Result<Vec<StudentProfile>, anyhow::Error> {
        let students = openatom_student::Entity::find()
            .all(self.get_connection())
            .await?;
        let mut result = Vec::with_capacity(students.len());
        for student in students {
            let user = user::Entity::find_by_id(student.user_id.clone())
                .one(self.get_connection())
                .await?;
            if let Some(user) = user {
                result.push(StudentProfile {
                    student_id: student.id,
                    user_id: user.id.clone(),
                    account_id: self.lookup_account_id(&user.id).await?,
                    student_name: student.full_name.clone().or(user.name).unwrap_or_default(),
                    email: user.email,
                    contract_end_date: student.contract_end_at.map(|dt| dt.date()),
                });
            }
        }
        Ok(result)
    }

    pub async fn get_student_by_student_id(
        &self,
        student_id: &str,
    ) -> Result<Option<StudentProfile>, anyhow::Error> {
        let student = openatom_student::Entity::find_by_id(student_id.to_owned())
            .one(self.get_connection())
            .await?;
        if let Some(student) = student {
            let user = user::Entity::find_by_id(student.user_id.clone())
                .one(self.get_connection())
                .await?;
            if let Some(user) = user {
                return Ok(Some(StudentProfile {
                    student_id: student.id,
                    user_id: user.id.clone(),
                    account_id: self.lookup_account_id(&user.id).await?,
                    student_name: student.full_name.or(user.name).unwrap_or_default(),
                    email: user.email,
                    contract_end_date: student.contract_end_at.map(|dt| dt.date()),
                }));
            }
        }
        Ok(None)
    }

    pub async fn get_student_id_by_login(
        &self,
        login: &str,
    ) -> Result<Option<String>, anyhow::Error> {
        let account = account::Entity::find()
            .filter(account::Column::Login.eq(login))
            .one(self.get_connection())
            .await?;
        if let Some(account) = account {
            let student = openatom_student::Entity::find()
                .filter(openatom_student::Column::UserId.eq(account.user_id))
                .one(self.get_connection())
                .await?;
            return Ok(student.map(|s| s.id));
        }
        Ok(None)
    }

    pub async fn get_student_login_by_student_id(
        &self,
        student_id: &str,
    ) -> Result<Option<String>, anyhow::Error> {
        let student = openatom_student::Entity::find_by_id(student_id.to_owned())
            .one(self.get_connection())
            .await?;
        let Some(student) = student else {
            return Ok(None);
        };

        let account = account::Entity::find()
            .filter(account::Column::UserId.eq(student.user_id))
            .one(self.get_connection())
            .await?;
        Ok(account.and_then(|a| a.login))
    }

    async fn lookup_account_id(&self, user_id: &str) -> Result<Option<String>, anyhow::Error> {
        let account = account::Entity::find()
            .filter(account::Column::UserId.eq(user_id))
            .one(self.get_connection())
            .await?;
        Ok(account.map(|a| a.account_id))
    }
}
