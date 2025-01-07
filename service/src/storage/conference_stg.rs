use std::sync::Arc;

use entity::conference;
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};

#[derive(Clone)]
pub struct ConferenceStorage {
    pub connection: Arc<DatabaseConnection>,
}

impl ConferenceStorage {
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub async fn new(connection: Arc<DatabaseConnection>) -> Self {
        ConferenceStorage { connection }
    }

    pub async fn save_conf(&self, mut model: conference::Model) -> Result<(), anyhow::Error> {
        model.create_at = chrono::Utc::now().naive_utc();
        model.update_at = chrono::Utc::now().naive_utc();
        model
            .into_active_model()
            .insert(self.get_connection())
            .await
            .unwrap();
        Ok(())
    }
}
