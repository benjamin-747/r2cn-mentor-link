use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MonthlyScore::Table)
                    .if_not_exists()
                    .col(pk_auto(MonthlyScore::Id))
                    .col(string(MonthlyScore::GithubLogin))
                    .col(string(MonthlyScore::StudentName))
                    .col(big_integer(MonthlyScore::GithubId))
                    .col(integer(MonthlyScore::Year))
                    .col(integer(MonthlyScore::Month))
                    .col(integer(MonthlyScore::CarryoverScore))
                    .col(integer(MonthlyScore::NewScore))
                    .col(integer(MonthlyScore::ConsumptionScore))
                    .col(integer(MonthlyScore::Exchanged))
                    .col(date_time(MonthlyScore::CreateAt))
                    .col(date_time(MonthlyScore::UpdateAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-monthly_score_year_month")
                    .table(MonthlyScore::Table)
                    .col(MonthlyScore::Year)
                    .col(MonthlyScore::Month)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MonthlyScore::Table).to_owned())
            .await
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum MonthlyScore {
    Table,
    Id,
    GithubLogin,
    GithubId,
    StudentName,
    Year,
    Month,
    CarryoverScore,
    NewScore,
    ConsumptionScore,
    Exchanged,
    CreateAt,
    UpdateAt,
}
