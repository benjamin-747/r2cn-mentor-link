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
    pub login: String,
    pub status: String,
}

impl From<NewMentor> for mentor::ActiveModel {
    fn from(value: NewMentor) -> Self {
        Self {
            id: NotSet,
            name: Set(value.name),
            email: Set(value.email),
            login: Set(value.login),
            status: Set(value.status),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        }
    }
}
