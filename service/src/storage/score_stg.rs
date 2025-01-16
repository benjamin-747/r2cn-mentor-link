use std::sync::Arc;

use chrono::{Datelike, Utc};
use entity::prelude::Score;
use entity::score::{self};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set,
};

#[derive(Clone)]
pub struct ScoreStorage {
    connection: Arc<DatabaseConnection>,
}

impl ScoreStorage {
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub async fn new(connection: Arc<DatabaseConnection>) -> Self {
        ScoreStorage { connection }
    }

    pub async fn get_score(
        &self,
        year: i32,
        month: i32,
        login: String,
    ) -> Result<Option<score::Model>, anyhow::Error> {
        let record = score::Entity::find()
            .filter(score::Column::GithubLogin.eq(login))
            .filter(score::Column::Year.eq(year))
            .filter(score::Column::Month.eq(month))
            .one(self.get_connection())
            .await?;
        Ok(record)
    }

    pub async fn get_latest_score_by_login(
        &self,
        login: String,
    ) -> Result<Option<score::Model>, anyhow::Error> {
        let record = score::Entity::find()
            .filter(score::Column::GithubLogin.eq(login))
            .order_by_desc(score::Column::Year)
            .order_by_desc(score::Column::Month)
            .one(self.get_connection())
            .await?;
        Ok(record)
    }

    pub async fn list_score_by_month(
        &self,
        year: i32,
        month: i32,
    ) -> Result<Vec<score::Model>, anyhow::Error> {
        let records = score::Entity::find()
            .filter(score::Column::Year.eq(year))
            .filter(score::Column::Month.eq(month))
            .all(self.get_connection())
            .await?;
        Ok(records)
    }

    pub async fn insert_score(
        &self,
        active_model: score::ActiveModel,
    ) -> Result<score::Model, anyhow::Error> {
        let score = active_model.insert(self.get_connection()).await?;
        Ok(score)
    }

    pub async fn update_score(
        &self,
        active_model: score::ActiveModel,
    ) -> Result<score::Model, anyhow::Error> {
        let model = active_model.update(self.get_connection()).await?;
        Ok(model)
    }

    pub async fn calculate_bonus(&self, models: Vec<score::Model>) {
        for model in models {
            let score: ScoreRes = model.clone().into();
            let bonus = score.calculate_bonus();
            if bonus.0 > 0 {
                let mut a_model: score::ActiveModel = model.into();
                a_model.consumption_score = Set(bonus.0);
                a_model.exchanged = Set(Some(bonus.1));
                a_model.update_at = Set(Utc::now().naive_utc());
                a_model.update(self.get_connection()).await.unwrap();
            }
        }
    }

    pub async fn calculate_unactive_bonus(
        &self,
        models: Vec<score::Model>,
        filted: Vec<String>,
    ) -> Result<(), anyhow::Error> {
        let mut save_models = vec![];
        for model in models {
            if !filted.contains(&model.github_login) {
                let score: ScoreRes = model.clone().into();
                let bonus = score.calculate_bonus();
                if bonus.0 > 0 {
                    let new_score = score::ActiveModel {
                        id: NotSet,
                        github_login: Set(score.github_login.clone()),
                        github_id: Set(score.github_id),
                        year: Set(Utc::now().year()),
                        month: Set(Utc::now().month() as i32),
                        carryover_score: Set(score.score_balance()),
                        new_score: Set(0),
                        consumption_score: Set(bonus.0),
                        exchanged: Set(Some(bonus.1)),
                        create_at: Set(chrono::Utc::now().naive_utc()),
                        update_at: Set(chrono::Utc::now().naive_utc()),
                    };
                    save_models.push(new_score);
                }
            }
        }
        Score::insert_many(save_models)
            .exec(self.get_connection())
            .await?;
        Ok(())
    }
}

pub struct ScoreRes {
    pub id: i32,
    pub github_login: String,
    pub github_id: i64,
    pub year: i32,
    pub month: i32,
    pub carryover_score: i32,
    pub new_score: i32,
    pub consumption_score: i32,
    pub exchanged: Option<i32>,
}

impl From<score::Model> for ScoreRes {
    fn from(value: score::Model) -> Self {
        Self {
            id: value.id,
            github_login: value.github_login,
            github_id: value.github_id,
            year: value.year,
            month: value.month,
            carryover_score: value.carryover_score,
            new_score: value.new_score,
            consumption_score: value.consumption_score,
            exchanged: value.exchanged,
        }
    }
}

impl ScoreRes {
    pub fn score_balance(&self) -> i32 {
        self.carryover_score + self.new_score - self.consumption_score
    }

    pub fn calculate_bonus(&self) -> (i32, i32) {
        if self.score_balance() >= 100 {
            return (100, 8000);
        } else if self.score_balance() >= 80 {
            return (80, 6000);
        } else if self.score_balance() >= 60 {
            return (60, 4000);
        } else if self.score_balance() >= 40 {
            return (40, 2000);
        } else if self.score_balance() >= 20 {
            return (20, 1000);
        }
        (0, 0)
    }
}
