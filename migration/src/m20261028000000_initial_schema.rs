//! Initial schema, mirroring `schema/schema.ts` (`schema/migrations/0000_initial.sql`).
//!
//! - `links`: id (**nanoid** text PK, app-generated), slug (unique varchar(64)),
//!   url (text), title (nullable varchar(255)), enabled (bool, default true),
//!   createdBy (nullable text), createdAt/updatedAt (timestamptz).
//!
//! Column names keep the drizzle camelCase spelling (`createdAt`, …) so SeaORM
//! stays compatible with drizzle-generated migrations.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Links {
  Table,
  Id,
  Slug,
  Url,
  Title,
  Enabled,
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
          .table(Links::Table)
          .if_not_exists()
          .col(
            // nanoid — generated in application code (no DB default).
            ColumnDef::new(Links::Id).text().not_null().primary_key(),
          )
          .col(ColumnDef::new(Links::Slug).string_len(64).not_null())
          .col(ColumnDef::new(Links::Url).text().not_null())
          .col(ColumnDef::new(Links::Title).string_len(255).null())
          .col(
            ColumnDef::new(Links::Enabled)
              .boolean()
              .not_null()
              .default(true),
          )
          .col(ColumnDef::new(Links::CreatedBy).text().null())
          .col(
            ColumnDef::new(Links::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .col(
            ColumnDef::new(Links::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .to_owned(),
      )
      .await?;

    // drizzle emits `CONSTRAINT "links_slug_unique" UNIQUE("slug")`; a unique
    // index with the same name is equivalent for lookups and conflicts.
    manager
      .create_index(
        Index::create()
          .name("links_slug_unique")
          .table(Links::Table)
          .col(Links::Slug)
          .unique()
          .if_not_exists()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("links_createdAt_idx")
          .table(Links::Table)
          .col(Links::CreatedAt)
          .if_not_exists()
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Links::Table).if_exists().to_owned())
      .await
  }
}
