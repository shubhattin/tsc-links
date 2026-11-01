//! Adds `active` (boolean, default true) to `expense_sources`. Sources with
//! bills can't be deleted (`ON DELETE RESTRICT`), so retired vendors are
//! marked inactive and tucked away in the UI instead.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ExpenseSources {
  Table,
  Active,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(ExpenseSources::Table)
          .add_column_if_not_exists(
            ColumnDef::new(ExpenseSources::Active)
              .boolean()
              .not_null()
              .default(true),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(ExpenseSources::Table)
          .drop_column(ExpenseSources::Active)
          .to_owned(),
      )
      .await
  }
}
