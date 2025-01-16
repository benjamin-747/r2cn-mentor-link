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
                    .values(TaskStatusVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .if_not_exists()
                    .col(pk_auto(Task::Id))
                    .col(string(Task::Owner))
                    .col(string(Task::Repo))
                    .col(integer(Task::GithubIssueNumber))
                    .col(big_integer(Task::GithubRepoId))
                    .col(big_integer(Task::GithubIssueId))
                    .col(integer(Task::Score))
                    .col(enumeration(
                        Task::TaskStatus,
                        Alias::new("task_status"),
                        TaskStatusVariants::iter(),
                    ))
                    .col(integer_null(Task::FinishYear))
                    .col(integer_null(Task::FinishMonth))
                    .col(string_null(Task::StudentGithubLogin))
                    .col(string(Task::MentorGithubLogin))
                    .col(date_time(Task::CreateAt))
                    .col(date_time(Task::UpdateAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-task_issue_id")
                    .unique()
                    .table(Task::Table)
                    .col(Task::GithubIssueId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx-task_issue_id").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Task::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().if_exists().name(Task::TaskStatus).to_owned())
            .await?;
        Ok(())
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Task {
    Table,
    Id,
    Owner,
    Repo,
    GithubIssueNumber,
    GithubIssueId,
    GithubRepoId,
    Score,
    TaskStatus,
    FinishYear,
    FinishMonth,
    StudentGithubLogin,
    MentorGithubLogin,
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
