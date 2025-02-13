use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Student::Table)
                    .if_not_exists()
                    .col(pk_auto(Student::Id))
                    .col(string(Student::GithubLogin))
                    .col(string(Student::StudentName))
                    .col(date_null(Student::ContractEndDate))
                    .col(date_time(Student::CreateAt))
                    .col(date_time(Student::UpdateAt))
                    .to_owned(),
            )
            .await?;
        manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-student_login")
                .unique()
                .table(Student::Table)
                .col(Student::GithubLogin)
                .to_owned(),
        )
        .await?;
        Ok(())  
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Student::Table).to_owned())
            .await
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Student {
    Table,
    Id,
    GithubLogin,
    StudentName,
    ContractEndDate,
    CreateAt,
    UpdateAt,
}
