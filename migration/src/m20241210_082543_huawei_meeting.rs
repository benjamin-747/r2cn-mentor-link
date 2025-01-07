use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HuaweiMeeting::Table)
                    .if_not_exists()
                    .col(pk_auto(HuaweiMeeting::Id))
                    .col(string(HuaweiMeeting::ConferenceID))
                    .col(string(HuaweiMeeting::Subject))
                    .col(string(HuaweiMeeting::StartTime))
                    .col(string(HuaweiMeeting::EndTime))
                    .col(string(HuaweiMeeting::ConferenceState))
                    .col(string(HuaweiMeeting::Language))
                    .col(integer(HuaweiMeeting::RecordType))
                    .col(integer(HuaweiMeeting::IsAutoRecord))
                    .col(string(HuaweiMeeting::ConfType))
                    .col(string(HuaweiMeeting::ChairJoinUri))
                    .col(string(HuaweiMeeting::GuestJoinUri))
                    .col(date_time(HuaweiMeeting::CreateAt))
                    .col(date_time(HuaweiMeeting::UpdateAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .unique()
                    .if_not_exists()
                    .name("idx-time")
                    .table(HuaweiMeeting::Table)
                    .col(HuaweiMeeting::StartTime)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HuaweiMeeting::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum HuaweiMeeting {
    Table,
    Id,
    ConferenceID,
    Subject,
    StartTime,
    EndTime,
    ConferenceState,
    Language,
    RecordType,
    IsAutoRecord,
    ConfType,
    ChairJoinUri,
    GuestJoinUri,
    CreateAt,
    UpdateAt,
}
