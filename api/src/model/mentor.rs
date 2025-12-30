use entity::mentor;
use sea_orm::ActiveValue::{NotSet, Set};
use serde::{Deserialize, Serialize};
use service::storage::mentor_stg::MentorStatus;

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateMentorStatusRequest {
    pub login: String,
    pub status: MentorStatus,
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewMentor {
    pub name: String,
    pub email: String,
    pub github_login: String,
    pub status: String,
}

impl From<NewMentor> for mentor::ActiveModel {
    fn from(value: NewMentor) -> Self {
        Self {
            id: NotSet,
            name: Set(value.name),
            email: Set(value.email),
            github_login: Set(value.github_login),
            status: Set(MentorStatus::Active.into()),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        }
    }
}
