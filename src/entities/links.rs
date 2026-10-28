//! SeaORM entity for `links` (PostgreSQL).
//!
//! Mirrors `schema/schema.ts`: column names keep the drizzle camelCase
//! spelling (`createdAt`, `createdBy`).
//!
//! `id` is a **nanoid** string (`text` in Postgres) — generate with
//! [`crate::utils::id::new_id`]. `slug` is the public, editable path segment.

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(
  table_name = "links",
  model_attrs(derive(serde::Serialize, serde::Deserialize))
)]
pub struct Model {
  /// nanoid — set in app code via `crate::utils::id::new_id()` (no DB default).
  #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
  pub id: String,
  #[sea_orm(unique)]
  pub slug: String,
  #[sea_orm(column_type = "Text")]
  pub url: String,
  pub title: Option<String>,
  pub enabled: bool,
  #[sea_orm(column_name = "createdBy", column_type = "Text", nullable)]
  pub created_by: Option<String>,
  #[sea_orm(column_name = "createdAt")]
  pub created_at: DateTimeWithTimeZone,
  #[sea_orm(column_name = "updatedAt")]
  pub updated_at: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}
