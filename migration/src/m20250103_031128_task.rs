use extension::postgres::Type;
use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Task::TaskStatus)
                    .values(TaskStatusVariants::iter()).to_owned()
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .if_not_exists()
                    .col(pk_auto(Task::Id))
                    .col(big_integer(Task::GithubRepoId))
                    .col(integer_null(Task::Points))
                    .col(enumeration(
                        Task::TaskStatus,
                        Alias::new("task_status"),
                        TaskStatusVariants::iter(),
                    ))
                    .col(big_integer_null(Task::StudentGithubId))
                    .col(big_integer(Task::MentorGithubId))
                    .col(date_time(Task::CreateAt))
                    .col(date_time(Task::UpdateAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Task::Table).to_owned())
            .await
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Task {
    Table,
    Id,
    GithubRepoId,
    Points,
    TaskStatus,
    StudentGithubId,
    MentorGithubId,
    CreateAt,
    UpdateAt,
}

#[derive(Iden, EnumIter)]
pub enum TaskStatusVariants {
    #[iden = "Open"]
    Open,
    #[iden = "Invalid"]
    Invalid,
    #[iden = "RequestAssign"]
    RequestAssign,
    #[iden = "Assigned"]
    Assigned,
    #[iden = "RequestFinish"]
    RequestFinish,
    #[iden = "Finished"]
    Finished,
}
