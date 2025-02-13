use std::sync::Arc;

use chrono::{Datelike, Utc};
use entity::monthly_score::{self};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set,
};

use crate::model::score::ScoreDto;

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
    ) -> Result<Option<monthly_score::Model>, anyhow::Error> {
        let record = monthly_score::Entity::find()
            .filter(monthly_score::Column::GithubLogin.eq(login))
            .filter(monthly_score::Column::Year.eq(year))
            .filter(monthly_score::Column::Month.eq(month))
            .one(self.get_connection())
            .await?;
        Ok(record)
    }

    pub async fn get_latest_score_by_login(
        &self,
        login: String,
    ) -> Result<Option<monthly_score::Model>, anyhow::Error> {
        let record = monthly_score::Entity::find()
            .filter(monthly_score::Column::GithubLogin.eq(login))
            .order_by_desc(monthly_score::Column::Year)
            .order_by_desc(monthly_score::Column::Month)
            .one(self.get_connection())
            .await?;
        Ok(record)
    }

    pub async fn list_score_by_month(
        &self,
        year: i32,
        month: i32,
    ) -> Result<Vec<monthly_score::Model>, anyhow::Error> {
        let records = monthly_score::Entity::find()
            .filter(monthly_score::Column::Year.eq(year))
            .filter(monthly_score::Column::Month.eq(month))
            .all(self.get_connection())
            .await?;
        Ok(records)
    }

    pub async fn insert_score(
        &self,
        active_model: monthly_score::ActiveModel,
    ) -> Result<monthly_score::Model, anyhow::Error> {
        let score = active_model.insert(self.get_connection()).await?;
        Ok(score)
    }

    pub async fn update_score(
        &self,
        active_model: monthly_score::ActiveModel,
    ) -> Result<monthly_score::Model, anyhow::Error> {
        let model = active_model.update(self.get_connection()).await?;
        Ok(model)
    }

    pub async fn insert_or_update_carryover_score(
        &self,
        last_month: ScoreDto,
    ) -> Result<(), anyhow::Error> {
        let now = Utc::now().naive_utc();
        let year = now.year();
        let month = now.month() as i32;
        let github_login = last_month.github_login.clone();

        let balance = last_month.score_balance();
        let current_month = self.get_score(year, month, github_login).await.unwrap();
        if let Some(current_month) = current_month {
            let mut a_model: monthly_score::ActiveModel = current_month.into();
            a_model.carryover_score = Set(balance);
            a_model.update_at = Set(now);
            self.update_score(a_model).await.unwrap();
        } else {
            let new_score = monthly_score::ActiveModel {
                id: NotSet,
                github_login: Set(last_month.github_login),
                student_name: Set(last_month.student_name),
                github_id: Set(last_month.github_id),
                year: Set(now.year()),
                month: Set(now.month() as i32),
                carryover_score: Set(balance),
                new_score: Set(0),
                consumption_score: Set(0),
                exchanged: Set(0),
                create_at: Set(now),
                update_at: Set(now),
            };
            self.insert_score(new_score).await.unwrap();
        }
        Ok(())
    }
}
