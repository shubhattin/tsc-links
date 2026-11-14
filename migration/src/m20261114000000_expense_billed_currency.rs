//! Generalises the optional vendor-billed amount beyond USD: renames
//! `usdCents` → `billedCents` and adds `billedCurrency` (`varchar(3)`,
//! default `'USD'`, so existing rows stay USD). Supported: `USD`, `EUR`.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Expenses {
  Table,
  #[sea_orm(iden = "usdCents")]
  UsdCents,
  #[sea_orm(iden = "billedCents")]
  BilledCents,
  #[sea_orm(iden = "billedCurrency")]
  BilledCurrency,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Expenses::Table)
          .rename_column(Expenses::UsdCents, Expenses::BilledCents)
          .to_owned(),
      )
      .await?;
    manager
      .alter_table(
        Table::alter()
          .table(Expenses::Table)
          .add_column_if_not_exists(
            ColumnDef::new(Expenses::BilledCurrency)
              .string_len(3)
              .not_null()
              .default("USD"),
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
          .drop_column(Expenses::BilledCurrency)
          .to_owned(),
      )
      .await?;
    manager
      .alter_table(
        Table::alter()
          .table(Expenses::Table)
          .rename_column(Expenses::BilledCents, Expenses::UsdCents)
          .to_owned(),
      )
      .await
  }
}
