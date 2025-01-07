use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppAuth {
    pub access_token: String,
    pub expire_time: i32,
    pub refresh_token: String,
    pub user: User,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub user_id: String,
    pub uclogin_account: String,
    pub name: String,
}