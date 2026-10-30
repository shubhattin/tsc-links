//! SeaORM entity for `expense_sources` (PostgreSQL).
//!
//! Mirrors `schema/schema.ts`. `key` is a short lowercase id (`vercel`,
//! `neon`, …) and the primary key; invoices point at it.

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(
  table_name = "expense_sources",
  model_attrs(derive(serde::Serialize, serde::Deserialize))
)]
pub struct Model {
  /// Lowercase source id (`[a-z0-9_-]{1,32}`), e.g. `vercel`.
  #[sea_orm(primary_key, auto_increment = false)]
  pub key: String,
  /// Display name, e.g. `Vercel`.
  pub name: String,
  #[sea_orm(column_type = "Text", nullable)]
  pub description: Option<String>,
  #[sea_orm(column_name = "createdAt")]
  pub created_at: DateTimeWithTimeZone,
  #[sea_orm(column_name = "updatedAt")]
  pub updated_at: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}
