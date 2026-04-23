use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .rename_column(
                        Alias::new("github_issue_number"),
                        Alias::new("issue_number"),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .rename_column(Alias::new("github_repo_id"), Alias::new("repo_id"))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .rename_column(Alias::new("github_issue_id"), Alias::new("issue_id"))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .rename_column(Alias::new("student_github_login"), Alias::new("student_id"))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .rename_column(
                        Alias::new("mentor_github_login"),
                        Alias::new("mentor_login"),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .rename_column(Alias::new("github_issue_title"), Alias::new("issue_title"))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .rename_column(Alias::new("github_issue_link"), Alias::new("issue_link"))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("scm_provider"))
                            .string()
                            .not_null()
                            .default("github"),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_column_if_not_exists(ColumnDef::new(Alias::new("external_ref")).string())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Mentor::Table)
                    .rename_column(Alias::new("github_login"), Alias::new("login"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(MonthlyScore::Table)
                    .rename_column(Alias::new("github_login"), Alias::new("student_id"))
                    .to_owned(),
            )
            .await?;

        // manager
        //     .drop_table(Table::drop().table(Conference::Table).to_owned())
        //     .await?;
        manager
            .drop_table(Table::drop().table(Student::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx-task_issue_id").to_owned())
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-task_issue_id")
                    .unique()
                    .table(Task::Table)
                    .col(Alias::new("issue_id"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Task {
    Table,
}

#[derive(DeriveIden)]
enum Student {
    Table,
}

#[derive(DeriveIden)]
enum Mentor {
    Table,
}

#[derive(DeriveIden)]
enum MonthlyScore {
    Table,
}
