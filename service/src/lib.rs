use std::sync::Arc;

use sea_orm::DatabaseConnection;
use storage::{
    conference_stg::ConferenceStorage, score_stg::ScoreStorage, student_stg::StudentStorage,
    task_stg::TaskStorage,
};

pub mod model;
pub mod ospp;
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

    pub fn task_stg(&self) -> TaskStorage {
        self.services.task_stg.clone()
    }

    pub fn score_stg(&self) -> ScoreStorage {
        self.services.score_stg.clone()
    }

    pub fn student_stg(&self) -> StudentStorage {
        self.services.student_stg.clone()
    }
}

#[derive(Clone)]
pub struct Service {
    pub conference_stg: ConferenceStorage,
    pub task_stg: TaskStorage,
    pub score_stg: ScoreStorage,
    pub student_stg: StudentStorage,
}

impl Service {
    async fn new(connection: Arc<DatabaseConnection>) -> Service {
        Service {
            conference_stg: ConferenceStorage::new(connection.clone()).await,
            task_stg: TaskStorage::new(connection.clone()).await,
            score_stg: ScoreStorage::new(connection.clone()).await,
            student_stg: StudentStorage::new(connection).await,
        }
    }
}
