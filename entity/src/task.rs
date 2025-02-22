//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.4

use super::sea_orm_active_enums::TaskStatus;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub owner: String,
    pub repo: String,
    pub github_issue_number: i32,
    pub github_repo_id: i64,
    #[sea_orm(unique)]
    pub github_issue_id: i64,
    pub score: i32,
    pub task_status: TaskStatus,
    pub finish_year: Option<i32>,
    pub finish_month: Option<i32>,
    pub student_github_login: Option<String>,
    pub student_name: Option<String>,
    pub mentor_github_login: String,
    pub create_at: DateTime,
    pub update_at: DateTime,
    pub github_issue_title: String,
    pub github_issue_link: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
