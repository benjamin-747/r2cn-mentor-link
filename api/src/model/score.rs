use chrono::{Datelike, Utc};
use entity::monthly_score;
use sea_orm::{ActiveValue::NotSet, Set};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewScore {
    pub score: i32,
    pub carryover_score: i32,
    pub github_id: i64,
    pub github_login: String,
    pub student_name: String,
}

impl From<NewScore> for monthly_score::ActiveModel {
    fn from(value: NewScore) -> Self {
        Self {
            id: NotSet,
            github_login: Set(value.github_login),
            student_name: Set(value.student_name),
            github_id: Set(value.github_id),
            year: Set(Utc::now().year()),
            month: Set(Utc::now().month() as i32),
            carryover_score: Set(value.carryover_score),
            new_score: Set(value.score),
            consumption_score: Set(0),
            exchanged: NotSet,
            create_at: Set(chrono::Utc::now().naive_utc()),
            update_at: Set(chrono::Utc::now().naive_utc()),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportExcel {
    pub year: i32,
    pub month: i32,
}
