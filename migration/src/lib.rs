pub use sea_orm_migration::prelude::*;

mod m20261028000000_initial_schema;
mod m20261028000001_expense_tracking;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20261028000000_initial_schema::Migration),
      Box::new(m20261028000001_expense_tracking::Migration),
    ]
  }
}
