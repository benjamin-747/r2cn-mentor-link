pub use sea_orm_migration::prelude::*;

mod m20241210_082543_huawei_meeting;
mod m20241212_090613_conference;
mod m20250103_031128_task;
mod m20250114_074736_score;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241210_082543_huawei_meeting::Migration),
            Box::new(m20241212_090613_conference::Migration),
            Box::new(m20250103_031128_task::Migration),
            Box::new(m20250114_074736_score::Migration),
        ]
    }
}
