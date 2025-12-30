use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Mentor::Table)
                    .if_not_exists()
                    .col(pk_auto(Mentor::Id))
                    .col(string(Mentor::Name))
                    .col(string(Mentor::Email))
                    .col(string(Mentor::GithubLogin))
                    .col(string(Mentor::Status))
                    .col(date_time(Mentor::CreatedAt))
                    .col(date_time(Mentor::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_mentors_github_login_unique")
                    .table(Mentor::Table)
                    .col(Mentor::GithubLogin)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mentor::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Mentor {
    Table,
    Id,
    Name,
    Email,
    GithubLogin,
    Status,
    CreatedAt,
    UpdatedAt,
}
