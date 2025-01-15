use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Score::Table)
                    .if_not_exists()
                    .col(pk_auto(Score::Id))
                    .col(string(Score::GithubLogin))
                    .col(big_integer(Score::GithubId))
                    .col(integer(Score::Year))
                    .col(integer(Score::Month))
                    .col(integer(Score::CarryoverScore))
                    .col(integer(Score::NewScore))
                    .col(integer(Score::ConsumptionScore))
                    .col(integer_null(Score::Exchanged))
                    .col(date_time(Score::CreateAt))
                    .col(date_time(Score::UpdateAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-score_year_month")
                    .unique()
                    .table(Score::Table)
                    .col(Score::Year)
                    .col(Score::Month)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Score::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Score {
    Table,
    Id,
    GithubLogin,
    GithubId,
    Year,
    Month,
    CarryoverScore,
    NewScore,
    ConsumptionScore,
    Exchanged,
    CreateAt,
    UpdateAt,
}
