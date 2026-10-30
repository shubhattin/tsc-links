//! Adds optional `note` (text) to `expenses` — a short explanatory line
//! shown under the entry in the month view and editable in the dialogs.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Expenses {
  Table,
  Note,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Expenses::Table)
          .add_column_if_not_exists(
            ColumnDef::new(Expenses::Note).text().null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Expenses::Table)
          .drop_column(Expenses::Note)
          .to_owned(),
      )
      .await
  }
}
