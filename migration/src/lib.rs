pub use sea_orm_migration::prelude::*;

mod m20250103_031128_task;
mod m20250114_074736_monthly_score;
mod m20250212_084656_create_student;
mod m20250222_082628_alter_task;
mod m20251226_023303_create_mentor;
mod m20260421_081200_rename_columns;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250103_031128_task::Migration),
            Box::new(m20250114_074736_monthly_score::Migration),
            Box::new(m20250212_084656_create_student::Migration),
            Box::new(m20250222_082628_alter_task::Migration),
            Box::new(m20251226_023303_create_mentor::Migration),
            Box::new(m20260421_081200_rename_columns::Migration),
        ]
    }
}
