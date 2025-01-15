use std::sync::Arc;

use sea_orm::DatabaseConnection;
use storage::{conference_stg::ConferenceStorage, huaweimeeting_stg::HuaweiMeetingStorage, score_stg::ScoreStorage, task_stg::TaskStorage};
pub mod storage;

#[derive(Clone)]
pub struct Context {
    pub services: Arc<Service>,
}

impl Context {
    pub async fn new(connection: Arc<DatabaseConnection>) -> Self {
        Context {
            services: Arc::new(Service::new(connection).await),
        }
    }

    pub fn conf_stg(&self) -> ConferenceStorage {
        self.services.conference_stg.clone()
    }

    pub fn huaweimeeting_stg(&self) -> HuaweiMeetingStorage {
        self.services.huaweimeeting_stg.clone()
    }

    pub fn task_stg(&self) -> TaskStorage {
        self.services.task_stg.clone()
    }

    pub fn score_stg(&self) -> ScoreStorage {
        self.services.score_stg.clone()
    }
}

#[derive(Clone)]
pub struct Service {
    pub conference_stg: ConferenceStorage,
    pub huaweimeeting_stg: HuaweiMeetingStorage,
    pub task_stg: TaskStorage,
    pub score_stg: ScoreStorage,
}


impl Service {
    async fn new(connection: Arc<DatabaseConnection>) -> Service {
        Service {
            conference_stg: ConferenceStorage::new(connection.clone()).await,
            huaweimeeting_stg: HuaweiMeetingStorage::new(connection.clone()).await,
            task_stg: TaskStorage::new(connection.clone()).await,
            score_stg: ScoreStorage::new(connection).await,
        }
    }
}


