//! SeaORM entity for `expenses` (PostgreSQL).
//!
//! Mirrors `schema/schema.ts`: column names keep the drizzle camelCase
//! spelling. One row is one charged cost; a source can have many rows per
//! month (the UI aggregates them).
//!
//! Money is integer minor units (`inrPaise`, `billedCents`) — no float math.
//! Invoices live either behind `invoiceLink` (external URL) or inline as
//! `invoiceBlob` (bytea, images re-encoded to webp on upload).

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(
  table_name = "expenses",
  model_attrs(derive(serde::Serialize, serde::Deserialize))
)]
pub struct Model {
  /// bigserial — database-generated, never set in app code.
  #[sea_orm(primary_key, auto_increment = true)]
  pub id: i64,
  #[sea_orm(column_name = "sourceKey")]
  pub source_key: String,
  /// Day precision (`YYYY-MM-DD` on the wire).
  pub date: Date,
  /// Optional pill label, e.g. `Pro plan`.
  pub label: Option<String>,
  /// Optional short explanatory note.
  #[sea_orm(column_type = "Text", nullable)]
  pub note: Option<String>,
  /// Optional vendor-billed amount, in minor units of `billed_currency`.
  #[sea_orm(column_name = "billedCents", nullable)]
  pub billed_cents: Option<i64>,
  /// ISO 4217 code of `billed_cents`: `USD` (default) or `EUR`.
  #[sea_orm(column_name = "billedCurrency")]
  pub billed_currency: String,
  #[sea_orm(column_name = "inrPaise")]
  pub inr_paise: i64,
  #[sea_orm(column_name = "invoiceLink", column_type = "Text", nullable)]
  pub invoice_link: Option<String>,
  #[sea_orm(column_name = "invoiceBlob", column_type = "Blob", nullable)]
  pub invoice_blob: Option<Vec<u8>>,
  #[sea_orm(column_name = "invoiceMime", nullable)]
  pub invoice_mime: Option<String>,
  #[sea_orm(column_name = "invoiceName", nullable)]
  pub invoice_name: Option<String>,
  #[sea_orm(column_name = "createdBy", column_type = "Text", nullable)]
  pub created_by: Option<String>,
  #[sea_orm(column_name = "createdAt")]
  pub created_at: DateTimeWithTimeZone,
  #[sea_orm(column_name = "updatedAt")]
  pub updated_at: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}
