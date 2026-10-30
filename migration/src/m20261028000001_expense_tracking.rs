//! Expense tracking tables, mirroring `schema/schema.ts`.
//!
//! - `expense_sources`: `key` (varchar(32) PK, e.g. `vercel`), display `name`,
//!   optional `description`, createdAt/updatedAt (timestamptz).
//! - `expenses`: bigserial `id` PK, `sourceKey` FK → `expense_sources.key`
//!   (`ON DELETE RESTRICT` — a source with entries can't be removed),
//!   `date` (Postgres `date`, day precision), optional `label` pill,
//!   `usdCents` (nullable bigint), `inrPaise` (bigint, required),
//!   optional `invoiceLink` plus raw `invoiceBlob` (bytea) with its
//!   `invoiceMime`/`invoiceName`, createdBy/createdAt/updatedAt.
//!
//! Money is integer minor units (paise/cents) — no float rounding.
//! Column names keep the drizzle camelCase spelling.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ExpenseSources {
  Table,
  Key,
  Name,
  Description,
  #[sea_orm(iden = "createdAt")]
  CreatedAt,
  #[sea_orm(iden = "updatedAt")]
  UpdatedAt,
}

#[derive(DeriveIden)]
enum Expenses {
  Table,
  Id,
  #[sea_orm(iden = "sourceKey")]
  SourceKey,
  Date,
  Label,
  #[sea_orm(iden = "usdCents")]
  UsdCents,
  #[sea_orm(iden = "inrPaise")]
  InrPaise,
  #[sea_orm(iden = "invoiceLink")]
  InvoiceLink,
  #[sea_orm(iden = "invoiceBlob")]
  InvoiceBlob,
  #[sea_orm(iden = "invoiceMime")]
  InvoiceMime,
  #[sea_orm(iden = "invoiceName")]
  InvoiceName,
  #[sea_orm(iden = "createdBy")]
  CreatedBy,
  #[sea_orm(iden = "createdAt")]
  CreatedAt,
  #[sea_orm(iden = "updatedAt")]
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(ExpenseSources::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(ExpenseSources::Key)
              .string_len(32)
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(ExpenseSources::Name)
              .string_len(64)
              .not_null(),
          )
          .col(ColumnDef::new(ExpenseSources::Description).text().null())
          .col(
            ColumnDef::new(ExpenseSources::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .col(
            ColumnDef::new(ExpenseSources::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(Expenses::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Expenses::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(Expenses::SourceKey)
              .string_len(32)
              .not_null(),
          )
          .col(ColumnDef::new(Expenses::Date).date().not_null())
          .col(ColumnDef::new(Expenses::Label).string_len(120).null())
          .col(ColumnDef::new(Expenses::UsdCents).big_integer().null())
          .col(
            ColumnDef::new(Expenses::InrPaise)
              .big_integer()
              .not_null(),
          )
          .col(ColumnDef::new(Expenses::InvoiceLink).text().null())
          .col(
            ColumnDef::new(Expenses::InvoiceBlob)
              .binary()
              .null(),
          )
          .col(
            ColumnDef::new(Expenses::InvoiceMime)
              .string_len(64)
              .null(),
          )
          .col(
            ColumnDef::new(Expenses::InvoiceName)
              .string_len(255)
              .null(),
          )
          .col(ColumnDef::new(Expenses::CreatedBy).text().null())
          .col(
            ColumnDef::new(Expenses::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .col(
            ColumnDef::new(Expenses::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .foreign_key(
            ForeignKey::create()
              .from(Expenses::Table, Expenses::SourceKey)
              .to(ExpenseSources::Table, ExpenseSources::Key)
              .on_delete(ForeignKeyAction::Restrict),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("expenses_sourceKey_date_idx")
          .table(Expenses::Table)
          .col(Expenses::SourceKey)
          .col(Expenses::Date)
          .if_not_exists()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("expenses_date_idx")
          .table(Expenses::Table)
          .col(Expenses::Date)
          .if_not_exists()
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Expenses::Table).if_exists().to_owned())
      .await?;
    manager
      .drop_table(
        Table::drop()
          .table(ExpenseSources::Table)
          .if_exists()
          .to_owned(),
      )
      .await
  }
}
