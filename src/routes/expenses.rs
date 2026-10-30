//! Expense tracking API (all routes behind [`crate::auth::require_admin`]).
//!
//! | Method | Path | Handler |
//! |--------|------|---------|
//! | `GET` | `/api/expense-sources` | [`list_sources`] |
//! | `POST` | `/api/expense-sources` | [`create_source`] |
//! | `PATCH` | `/api/expense-sources/{key}` | [`update_source`] |
//! | `DELETE` | `/api/expense-sources/{key}` | [`delete_source`] |
//! | `GET` | `/api/expenses[?month=YYYY-MM]` | [`list_expenses`] |
//! | `POST` | `/api/expenses` | [`create_expense`] |
//! | `PATCH` | `/api/expenses/{id}` | [`update_expense`] |
//! | `DELETE` | `/api/expenses/{id}` | [`delete_expense`] |
//! | `PUT` | `/api/expenses/{id}/invoice` | [`upload_invoice`] |
//! | `GET` | `/api/expenses/{id}/invoice` | [`download_invoice`] |
//! | `DELETE` | `/api/expenses/{id}/invoice` | [`delete_invoice`] |
//!
//! Money is integer minor units (paise/cents) on the wire. Dates are
//! `YYYY-MM-DD` (`date` in Postgres, displayed `dd/MM/yyyy` in the UI).
//! Invoices are either an external `invoiceLink` or raw `invoiceBlob` bytes
//! (images re-encoded to webp on upload, PDFs stored as-is).

use std::collections::HashMap;

use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::NaiveDate;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::auth::{AuthUser, ErrorBody, error_response};
use crate::entities::{expense, expense_source};
use crate::state::AppState;

const LABEL_MAX_LEN: usize = 120;
const NAME_MAX_LEN: usize = 64;
const DESC_MAX_LEN: usize = 2000;
/// Raw invoice uploads are capped (Vercel function payloads are small anyway).
const INVOICE_MAX_BYTES: usize = 8 * 1024 * 1024;
/// Uploaded images are downscaled to this bounding box before webp encoding.
const INVOICE_IMAGE_MAX_DIM: u32 = 2048;

/// `PUT` invoice accepts these; images become webp, PDFs are stored raw.
const IMAGE_MIMES: &[&str] = &["image/png", "image/jpeg", "image/webp"];

/* ---------------- sources ---------------- */

/// Expense source row (seeded vendors + custom ones).
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ExpenseSourceDto {
  /// Lowercase source id, e.g. `vercel`.
  pub key: String,
  /// Display name, e.g. `Vercel`.
  pub name: String,
  pub description: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

impl From<expense_source::Model> for ExpenseSourceDto {
  fn from(m: expense_source::Model) -> Self {
    Self {
      key: m.key,
      name: m.name,
      description: m.description,
      created_at: m.created_at.to_rfc3339(),
      updated_at: m.updated_at.to_rfc3339(),
    }
  }
}

/// `GET /api/expense-sources` response (alphabetical by name).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExpenseSourcesListResponse {
  pub sources: Vec<ExpenseSourceDto>,
}

/// `POST /api/expense-sources` body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSourceBody {
  /// Lowercase `[a-z0-9_-]{1,32}`.
  pub key: String,
  /// Display name (max 64 chars).
  pub name: String,
  pub description: Option<String>,
}

/// `PATCH /api/expense-sources/{key}` body — omitted fields are unchanged,
/// empty/null description clears it.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSourceBody {
  pub name: Option<String>,
  pub description: Option<String>,
}

fn normalize_source_key(raw: &str) -> Result<String, String> {
  let key = raw.trim().to_lowercase();
  if key.is_empty() || key.len() > 32 {
    return Err("key must be 1-32 characters".to_string());
  }
  if !key
    .bytes()
    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
  {
    return Err("key must be lowercase letters, digits, '-' or '_'".to_string());
  }
  Ok(key)
}

fn normalize_name(raw: &str) -> Result<String, String> {
  let name = raw.trim();
  if name.is_empty() {
    return Err("name is required".to_string());
  }
  if name.chars().count() > NAME_MAX_LEN {
    return Err(format!("name must be at most {NAME_MAX_LEN} characters"));
  }
  Ok(name.to_string())
}

fn normalize_description(raw: Option<&str>) -> Result<Option<String>, String> {
  let Some(desc) = raw.map(str::trim).filter(|d| !d.is_empty()) else {
    return Ok(None);
  };
  if desc.chars().count() > DESC_MAX_LEN {
    return Err(format!(
      "description must be at most {DESC_MAX_LEN} characters"
    ));
  }
  Ok(Some(desc.to_string()))
}

/// Link routes with OpenAPI path registration.
pub fn openapi_router() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes!(list_sources, create_source))
    .routes(routes!(update_source, delete_source))
    .routes(routes!(list_expenses, create_expense))
    .routes(routes!(update_expense, delete_expense))
    .routes(routes!(upload_invoice, download_invoice, delete_invoice))
}

/// `GET /api/expense-sources` — every source, alphabetical by name.
#[utoipa::path(
    get,
    path = "/api/expense-sources",
    operation_id = "sources.list",
    tag = "expenses",
    security(("session_cookie" = [])),
    responses(
        (status = 200, description = "All expense sources", body = ExpenseSourcesListResponse),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
    )
)]
pub async fn list_sources(State(state): State<AppState>) -> Response {
  match expense_source::Entity::find()
    .order_by_asc(expense_source::Column::Name)
    .all(&state.db)
    .await
  {
    Ok(rows) => Json(ExpenseSourcesListResponse {
      sources: rows.into_iter().map(ExpenseSourceDto::from).collect(),
    })
    .into_response(),
    Err(e) => internal_error(e),
  }
}

/// `POST /api/expense-sources` — register a custom source.
#[utoipa::path(
    post,
    path = "/api/expense-sources",
    operation_id = "sources.create",
    tag = "expenses",
    security(("session_cookie" = [])),
    request_body = CreateSourceBody,
    responses(
        (status = 201, description = "Source created", body = ExpenseSourceDto),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 409, description = "Key already in use", body = ErrorBody),
    )
)]
pub async fn create_source(
  State(state): State<AppState>,
  Json(body): Json<CreateSourceBody>,
) -> Response {
  let key = match normalize_source_key(&body.key) {
    Ok(k) => k,
    Err(msg) => return bad_request(msg),
  };
  let name = match normalize_name(&body.name) {
    Ok(n) => n,
    Err(msg) => return bad_request(msg),
  };
  let description = match normalize_description(body.description.as_deref()) {
    Ok(d) => d,
    Err(msg) => return bad_request(msg),
  };
  let now = chrono::Utc::now().fixed_offset();
  let model = expense_source::ActiveModel {
    key: Set(key.clone()),
    name: Set(name),
    description: Set(description),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  match expense_source::Entity::insert(model).exec(&state.db).await {
    Ok(_) => match expense_source::Entity::find_by_id(&key).one(&state.db).await {
      Ok(Some(row)) => (StatusCode::CREATED, Json(ExpenseSourceDto::from(row))).into_response(),
      Ok(None) => internal_error(DbErr::RecordNotFound("source vanished".to_string())),
      Err(e) => internal_error(e),
    },
    Err(e) if is_unique_violation(&e) => error_response(
      StatusCode::CONFLICT,
      format!("source '{key}' already exists"),
    ),
    Err(e) => internal_error(e),
  }
}

/// `PATCH /api/expense-sources/{key}` — rename / edit description.
#[utoipa::path(
    patch,
    path = "/api/expense-sources/{key}",
    operation_id = "sources.update",
    tag = "expenses",
    security(("session_cookie" = [])),
    params(("key" = String, Path, description = "Source key")),
    request_body = UpdateSourceBody,
    responses(
        (status = 200, description = "Source updated", body = ExpenseSourceDto),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 404, description = "Source not found", body = ErrorBody),
    )
)]
pub async fn update_source(
  State(state): State<AppState>,
  Path(key): Path<String>,
  Json(body): Json<UpdateSourceBody>,
) -> Response {
  let row = match expense_source::Entity::find_by_id(key.trim()).one(&state.db).await {
    Ok(Some(row)) => row,
    Ok(None) => return error_response(StatusCode::NOT_FOUND, "source not found"),
    Err(e) => return internal_error(e),
  };
  let mut model: expense_source::ActiveModel = row.into();
  if let Some(raw) = body.name.as_deref() {
    match normalize_name(raw) {
      Ok(name) => model.name = Set(name),
      Err(msg) => return bad_request(msg),
    }
  }
  if body.description.is_some() {
    match normalize_description(body.description.as_deref()) {
      Ok(description) => model.description = Set(description),
      Err(msg) => return bad_request(msg),
    }
  }
  model.updated_at = Set(chrono::Utc::now().fixed_offset());
  match model.update(&state.db).await {
    Ok(row) => Json(ExpenseSourceDto::from(row)).into_response(),
    Err(e) => internal_error(e),
  }
}

/// `DELETE /api/expense-sources/{key}` — refused (409) while entries exist.
#[utoipa::path(
    delete,
    path = "/api/expense-sources/{key}",
    operation_id = "sources.delete",
    tag = "expenses",
    security(("session_cookie" = [])),
    params(("key" = String, Path, description = "Source key")),
    responses(
        (status = 204, description = "Source deleted"),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 404, description = "Source not found", body = ErrorBody),
        (status = 409, description = "Source still has entries", body = ErrorBody),
    )
)]
pub async fn delete_source(State(state): State<AppState>, Path(key): Path<String>) -> Response {
  let key = key.trim();
  if expense_source::Entity::find_by_id(key)
    .one(&state.db)
    .await
    .map(|r| r.is_none())
    .unwrap_or(false)
  {
    return error_response(StatusCode::NOT_FOUND, "source not found");
  }
  match source_has_entries(&state, key).await {
    Ok(true) => {
      return error_response(
        StatusCode::CONFLICT,
        "source still has expense entries and cannot be deleted",
      );
    }
    Ok(false) => {}
    Err(e) => return internal_error(e),
  }
  match expense_source::Entity::delete_by_id(key).exec(&state.db).await {
    Ok(_) => StatusCode::NO_CONTENT.into_response(),
    Err(e) if is_fk_violation(&e) => error_response(
      StatusCode::CONFLICT,
      "source still has expense entries and cannot be deleted",
    ),
    Err(e) => internal_error(e),
  }
}

/// `true` when deleting `key` would orphan entries. Checked up front so the
/// common case never depends on driver error codes; the delete below keeps
/// the `is_fk_violation` fallback for the (tiny) race window.
async fn source_has_entries(state: &AppState, key: &str) -> Result<bool, DbErr> {
  Ok(
    expense::Entity::find()
      .filter(expense::Column::SourceKey.eq(key))
      .count(&state.db)
      .await?
      > 0,
  )
}

/* ---------------- expenses ---------------- */

/// Expense row for API responses (`date` is `YYYY-MM-DD`; blobs never leave
/// the server except through the dedicated download endpoint).
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ExpenseDto {
  pub id: i64,
  pub source_key: String,
  pub source_name: String,
  /// Charge date (`YYYY-MM-DD`).
  pub date: String,
  /// Optional pill label.
  pub label: Option<String>,
  /// Optional charged USD amount, in cents.
  pub usd_cents: Option<i64>,
  /// Final charged INR amount, in paise.
  pub inr_paise: i64,
  pub invoice_link: Option<String>,
  /// Whether a blob invoice is attached (fetch it from the download endpoint).
  pub has_invoice: bool,
  pub invoice_name: Option<String>,
  pub created_by: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

/// `GET /api/expenses` response (newest date first).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExpensesListResponse {
  pub expenses: Vec<ExpenseDto>,
}

/// `GET /api/expenses?month=YYYY-MM` filter.
#[derive(Debug, Deserialize)]
pub struct ExpensesQuery {
  pub month: Option<String>,
}

/// `POST /api/expenses` body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateExpenseBody {
  pub source_key: String,
  /// Charge date (`YYYY-MM-DD`).
  pub date: String,
  /// Optional pill label (max 120 chars).
  pub label: Option<String>,
  /// Optional charged USD amount, in cents.
  pub usd_cents: Option<i64>,
  /// Final charged INR amount, in paise (required).
  pub inr_paise: i64,
  /// Optional external invoice URL.
  pub invoice_link: Option<String>,
}

/// `PATCH /api/expenses/{id}` body — omitted fields are unchanged; empty/null
/// label/link and null `usd_cents` clear the field.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateExpenseBody {
  pub source_key: Option<String>,
  pub date: Option<String>,
  pub label: Option<String>,
  // Explicit `null` must mean "clear", not "unchanged" — serde collapses
  // `Option<Option<T>>` nulls to `None` by default, hence the custom
  // deserializer (missing → `None`, null → `Some(None)`).
  #[serde(default, deserialize_with = "deserialize_optional_field")]
  pub usd_cents: Option<Option<i64>>,
  pub inr_paise: Option<i64>,
  #[serde(default, deserialize_with = "deserialize_optional_field")]
  pub invoice_link: Option<Option<String>>,
}

fn deserialize_optional_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
  D: serde::Deserializer<'de>,
  T: serde::Deserialize<'de>,
{
  Ok(Some(Option::deserialize(deserializer)?))
}

fn parse_date(raw: &str) -> Result<NaiveDate, String> {
  let raw = raw.trim();
  // chrono accepts non-padded `2026-1-5`; the wire format is strict.
  if raw.len() != 10 {
    return Err("date must be YYYY-MM-DD".to_string());
  }
  NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_err(|_| "date must be YYYY-MM-DD".to_string())
}

/// `YYYY-MM` → `[first-day, first-of-next-month)`.
fn parse_month(raw: &str) -> Result<(NaiveDate, NaiveDate), String> {
  let raw = raw.trim();
  let parts: Vec<&str> = raw.split('-').collect();
  if parts.len() != 2 || parts[0].len() != 4 || parts[1].len() != 2 {
    return Err("month must be YYYY-MM".to_string());
  }
  let year: i32 = parts[0].parse().map_err(|_| "month must be YYYY-MM".to_string())?;
  let month: u32 = parts[1].parse().map_err(|_| "month must be YYYY-MM".to_string())?;
  if !(1..=12).contains(&month) {
    return Err("month must be YYYY-MM".to_string());
  }
  let start = NaiveDate::from_ymd_opt(year, month, 1)
    .ok_or_else(|| "month must be YYYY-MM".to_string())?;
  let end = if month == 12 {
    NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("year+1 is valid")
  } else {
    NaiveDate::from_ymd_opt(year, month + 1, 1).expect("next month is valid")
  };
  Ok((start, end))
}

fn normalize_label(raw: Option<&str>) -> Result<Option<String>, String> {
  let Some(label) = raw.map(str::trim).filter(|l| !l.is_empty()) else {
    return Ok(None);
  };
  if label.chars().count() > LABEL_MAX_LEN {
    return Err(format!("label must be at most {LABEL_MAX_LEN} characters"));
  }
  Ok(Some(label.to_string()))
}

fn normalize_money(raw: i64, field: &str) -> Result<i64, String> {
  if raw < 0 {
    return Err(format!("{field} cannot be negative"));
  }
  // 9e15 paise/cents is absurd but keeps the door open; the real guard is
  // against nonsense negative values.
  Ok(raw)
}

fn normalize_invoice_link(raw: Option<&str>) -> Result<Option<String>, String> {
  let Some(link) = raw.map(str::trim).filter(|l| !l.is_empty()) else {
    return Ok(None);
  };
  let parsed = url::Url::parse(link).map_err(|_| "invoice link must be a valid URL".to_string())?;
  if !matches!(parsed.scheme(), "http" | "https") {
    return Err("invoice link must start with http:// or https://".to_string());
  }
  Ok(Some(parsed.to_string()))
}

fn expense_dto(row: expense::Model, source_name: &str) -> ExpenseDto {
  ExpenseDto {
    id: row.id,
    source_key: row.source_key,
    source_name: source_name.to_string(),
    date: row.date.format("%Y-%m-%d").to_string(),
    label: row.label,
    usd_cents: row.usd_cents,
    inr_paise: row.inr_paise,
    invoice_link: row.invoice_link,
    has_invoice: row.invoice_blob.is_some(),
    invoice_name: row.invoice_name,
    created_by: row.created_by,
    created_at: row.created_at.to_rfc3339(),
    updated_at: row.updated_at.to_rfc3339(),
  }
}

async fn source_names(state: &AppState) -> Result<HashMap<String, String>, DbErr> {
  Ok(
    expense_source::Entity::find()
      .all(&state.db)
      .await?
      .into_iter()
      .map(|s| (s.key, s.name))
      .collect(),
  )
}

/// `GET /api/expenses[?month=YYYY-MM]` — entries, newest date first.
#[utoipa::path(
    get,
    path = "/api/expenses",
    operation_id = "expenses.list",
    tag = "expenses",
    security(("session_cookie" = [])),
    params(("month" = Option<String>, Query, description = "Filter to a calendar month (YYYY-MM)")),
    responses(
        (status = 200, description = "Expense entries", body = ExpensesListResponse),
        (status = 400, description = "Bad month filter", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
    )
)]
pub async fn list_expenses(
  State(state): State<AppState>,
  Query(query): Query<ExpensesQuery>,
) -> Response {
  let mut finder = expense::Entity::find()
    .order_by_desc(expense::Column::Date)
    .order_by_desc(expense::Column::Id);
  if let Some(month) = query.month.as_deref() {
    match parse_month(month) {
      Ok((start, end)) => {
        finder = finder
          .filter(expense::Column::Date.gte(start))
          .filter(expense::Column::Date.lt(end));
      }
      Err(msg) => return bad_request(msg),
    }
  }
  let (rows, names) = match tokio::join!(finder.all(&state.db), source_names(&state)) {
    (Ok(rows), Ok(names)) => (rows, names),
    _ => {
      return error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "database error",
      );
    }
  };
  Json(ExpensesListResponse {
    expenses: rows
      .into_iter()
      .map(|row| {
        let name = names
          .get(&row.source_key)
          .map(String::as_str)
          .unwrap_or(&row.source_key)
          .to_string();
        expense_dto(row, &name)
      })
      .collect(),
  })
  .into_response()
}

/// `POST /api/expenses` — record a charged cost.
#[utoipa::path(
    post,
    path = "/api/expenses",
    operation_id = "expenses.create",
    tag = "expenses",
    security(("session_cookie" = [])),
    request_body = CreateExpenseBody,
    responses(
        (status = 201, description = "Expense recorded", body = ExpenseDto),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
    )
)]
pub async fn create_expense(
  State(state): State<AppState>,
  auth: AuthUser,
  Json(body): Json<CreateExpenseBody>,
) -> Response {
  let source_key = match normalize_source_key(&body.source_key) {
    Ok(k) => k,
    Err(msg) => return bad_request(msg),
  };
  let date = match parse_date(&body.date) {
    Ok(d) => d,
    Err(msg) => return bad_request(msg),
  };
  let label = match normalize_label(body.label.as_deref()) {
    Ok(l) => l,
    Err(msg) => return bad_request(msg),
  };
  let usd_cents = match body.usd_cents.map(|c| normalize_money(c, "usd_cents")).transpose() {
    Ok(c) => c,
    Err(msg) => return bad_request(msg),
  };
  let inr_paise = match normalize_money(body.inr_paise, "inr_paise") {
    Ok(p) => p,
    Err(msg) => return bad_request(msg),
  };
  let invoice_link = match normalize_invoice_link(body.invoice_link.as_deref()) {
    Ok(l) => l,
    Err(msg) => return bad_request(msg),
  };

  let source_name = match expense_source::Entity::find_by_id(&source_key)
    .one(&state.db)
    .await
  {
    Ok(Some(source)) => source.name,
    Ok(None) => {
      return error_response(
        StatusCode::BAD_REQUEST,
        format!("unknown expense source '{source_key}'"),
      );
    }
    Err(e) => return internal_error(e),
  };

  let now = chrono::Utc::now().fixed_offset();
  let model = expense::ActiveModel {
    source_key: Set(source_key),
    date: Set(date),
    label: Set(label),
    usd_cents: Set(usd_cents),
    inr_paise: Set(inr_paise),
    invoice_link: Set(invoice_link),
    created_by: Set(Some(auth.user_id.clone())),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  match expense::Entity::insert(model).exec(&state.db).await {
    Ok(inserted) => {
      match expense::Entity::find_by_id(inserted.last_insert_id).one(&state.db).await {
        Ok(Some(row)) => (StatusCode::CREATED, Json(expense_dto(row, &source_name))).into_response(),
        Ok(None) => internal_error(DbErr::RecordNotFound("expense vanished".to_string())),
        Err(e) => internal_error(e),
      }
    }
    Err(e) => internal_error(e),
  }
}

/// `PATCH /api/expenses/{id}` — edit an entry.
#[utoipa::path(
    patch,
    path = "/api/expenses/{id}",
    operation_id = "expenses.update",
    tag = "expenses",
    security(("session_cookie" = [])),
    params(("id" = i64, Path, description = "Expense id")),
    request_body = UpdateExpenseBody,
    responses(
        (status = 200, description = "Expense updated", body = ExpenseDto),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 404, description = "Expense not found", body = ErrorBody),
    )
)]
pub async fn update_expense(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Json(body): Json<UpdateExpenseBody>,
) -> Response {
  let row = match expense::Entity::find_by_id(id).one(&state.db).await {
    Ok(Some(row)) => row,
    Ok(None) => return error_response(StatusCode::NOT_FOUND, "expense not found"),
    Err(e) => return internal_error(e),
  };
  let mut model: expense::ActiveModel = row.into();

  if let Some(raw) = body.source_key.as_deref() {
    let source_key = match normalize_source_key(raw) {
      Ok(k) => k,
      Err(msg) => return bad_request(msg),
    };
    match expense_source::Entity::find_by_id(&source_key).one(&state.db).await {
      Ok(Some(_)) => model.source_key = Set(source_key),
      Ok(None) => {
        return error_response(
          StatusCode::BAD_REQUEST,
          format!("unknown expense source '{source_key}'"),
        );
      }
      Err(e) => return internal_error(e),
    }
  }
  if let Some(raw) = body.date.as_deref() {
    match parse_date(raw) {
      Ok(date) => model.date = Set(date),
      Err(msg) => return bad_request(msg),
    }
  }
  if body.label.is_some() {
    match normalize_label(body.label.as_deref()) {
      Ok(label) => model.label = Set(label),
      Err(msg) => return bad_request(msg),
    }
  }
  if let Some(usd) = body.usd_cents {
    match usd.map(|c| normalize_money(c, "usd_cents")).transpose() {
      Ok(usd_cents) => model.usd_cents = Set(usd_cents),
      Err(msg) => return bad_request(msg),
    }
  }
  if let Some(inr) = body.inr_paise {
    match normalize_money(inr, "inr_paise") {
      Ok(inr_paise) => model.inr_paise = Set(inr_paise),
      Err(msg) => return bad_request(msg),
    }
  }
  if let Some(link) = body.invoice_link {
    match normalize_invoice_link(link.as_deref()) {
      Ok(invoice_link) => model.invoice_link = Set(invoice_link),
      Err(msg) => return bad_request(msg),
    }
  }
  model.updated_at = Set(chrono::Utc::now().fixed_offset());

  let row = match model.update(&state.db).await {
    Ok(row) => row,
    Err(e) => return internal_error(e),
  };
  let name = expense_source::Entity::find_by_id(&row.source_key)
    .one(&state.db)
    .await
    .ok()
    .flatten()
    .map(|s| s.name)
    .unwrap_or_else(|| row.source_key.clone());
  Json(expense_dto(row, &name)).into_response()
}

/// `DELETE /api/expenses/{id}` — delete an entry (its blob goes with it).
#[utoipa::path(
    delete,
    path = "/api/expenses/{id}",
    operation_id = "expenses.delete",
    tag = "expenses",
    security(("session_cookie" = [])),
    params(("id" = i64, Path, description = "Expense id")),
    responses(
        (status = 204, description = "Expense deleted"),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 404, description = "Expense not found", body = ErrorBody),
    )
)]
pub async fn delete_expense(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
  if expense::Entity::find_by_id(id)
    .one(&state.db)
    .await
    .map(|r| r.is_none())
    .unwrap_or(false)
  {
    return error_response(StatusCode::NOT_FOUND, "expense not found");
  }
  match expense::Entity::delete_by_id(id).exec(&state.db).await {
    Ok(_) => StatusCode::NO_CONTENT.into_response(),
    Err(e) => internal_error(e),
  }
}

/* ---------------- invoice blobs ---------------- */

async fn find_expense(state: &AppState, id: i64) -> Result<expense::Model, Response> {
  match expense::Entity::find_by_id(id).one(&state.db).await {
    Ok(Some(row)) => Ok(row),
    Ok(None) => Err(error_response(StatusCode::NOT_FOUND, "expense not found")),
    Err(e) => {
      eprintln!("database error: {e}");
      Err(error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "database error",
      ))
    }
  }
}

/// Decode an uploaded image and re-encode it as (lossless) webp, downscaled
/// to a bounded size. `image` 0.25 only ships a lossless webp encoder —
/// plenty for invoice screenshots at a fraction of PNG size.
fn image_to_webp(bytes: &[u8]) -> Result<Vec<u8>, String> {
  let img = image::load_from_memory(bytes).map_err(|_| "could not decode image".to_string())?;
  let img = if img.width() > INVOICE_IMAGE_MAX_DIM || img.height() > INVOICE_IMAGE_MAX_DIM {
    img.thumbnail(INVOICE_IMAGE_MAX_DIM, INVOICE_IMAGE_MAX_DIM)
  } else {
    img
  };
  let rgba = img.to_rgba8();
  let (width, height) = (rgba.width(), rgba.height());
  let mut out = Vec::new();
  image::codecs::webp::WebPEncoder::new_lossless(&mut out)
    .encode(&rgba, width, height, image::ExtendedColorType::Rgba8)
    .map_err(|e| format!("could not encode webp: {e}"))?;
  Ok(out)
}

/// `PUT /api/expenses/{id}/invoice` — attach an invoice (`multipart` `file`
/// field: PNG/JPEG/WebP image → webp, or PDF as-is; max 8 MiB).
#[utoipa::path(
    put,
    path = "/api/expenses/{id}/invoice",
    operation_id = "expenses.putInvoice",
    tag = "expenses",
    security(("session_cookie" = [])),
    params(("id" = i64, Path, description = "Expense id")),
    request_body(content = Vec<u8>, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Invoice attached", body = ExpenseDto),
        (status = 400, description = "Invalid input", body = ErrorBody),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 404, description = "Expense not found", body = ErrorBody),
        (status = 413, description = "File too large", body = ErrorBody),
        (status = 415, description = "Unsupported file type", body = ErrorBody),
    )
)]
pub async fn upload_invoice(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  mut multipart: Multipart,
) -> Response {
  let row = match find_expense(&state, id).await {
    Ok(row) => row,
    Err(response) => return response,
  };

  let mut picked: Option<(String, String, Vec<u8>)> = None;
  while let Ok(Some(field)) = multipart.next_field().await {
    if field.name() != Some("file") {
      continue;
    }
    let mime = field.content_type().unwrap_or("").to_lowercase();
    let name = field
      .file_name()
      .map(|n| n.to_string())
      .unwrap_or_else(|| "invoice".to_string());
    match field.bytes().await {
      Ok(bytes) => {
        picked = Some((mime, name, bytes.to_vec()));
        break;
      }
      Err(_) => return bad_request("could not read uploaded file"),
    }
  }
  let Some((mime, name, bytes)) = picked else {
    return bad_request("missing multipart `file` field");
  };
  if bytes.len() > INVOICE_MAX_BYTES {
    return error_response(
      StatusCode::PAYLOAD_TOO_LARGE,
      "invoice must be at most 8 MiB",
    );
  }
  if bytes.is_empty() {
    return bad_request("uploaded file is empty");
  }

  let (store_mime, store_bytes, store_name) = if IMAGE_MIMES.contains(&mime.as_str()) {
    match image_to_webp(&bytes) {
      Ok(webp) => (
        "image/webp".to_string(),
        webp,
        webp_file_name(&name),
      ),
      Err(msg) => return bad_request(msg),
    }
  } else if mime == "application/pdf" {
    (mime, bytes, sanitize_file_name(&name))
  } else {
    return error_response(
      StatusCode::UNSUPPORTED_MEDIA_TYPE,
      "invoice must be a PNG/JPEG/WebP image or a PDF",
    );
  };

  let source_name = expense_source::Entity::find_by_id(&row.source_key)
    .one(&state.db)
    .await
    .ok()
    .flatten()
    .map(|s| s.name)
    .unwrap_or_else(|| row.source_key.clone());
  let mut model: expense::ActiveModel = row.into();
  model.invoice_blob = Set(Some(store_bytes));
  model.invoice_mime = Set(Some(store_mime));
  model.invoice_name = Set(Some(store_name));
  model.updated_at = Set(chrono::Utc::now().fixed_offset());
  match model.update(&state.db).await {
    Ok(row) => Json(expense_dto(row, &source_name)).into_response(),
    Err(e) => internal_error(e),
  }
}

/// `photo.PNG` → `photo.webp`; overlong names are truncated.
fn webp_file_name(name: &str) -> String {
  let base = sanitize_file_name(name);
  let stem = base.rsplit_once('.').map(|(s, _)| s).unwrap_or(&base);
  format!("{stem}.webp")
}

fn sanitize_file_name(name: &str) -> String {
  let clean: String = name
    .trim()
    .chars()
    .take(240)
    .map(|c| if c.is_control() || c == '/' || c == '\\' { '_' } else { c })
    .collect();
  if clean.is_empty() {
    "invoice".to_string()
  } else {
    clean
  }
}

/// `GET /api/expenses/{id}/invoice` — download the attached invoice bytes.
#[utoipa::path(
    get,
    path = "/api/expenses/{id}/invoice",
    operation_id = "expenses.getInvoice",
    tag = "expenses",
    security(("session_cookie" = [])),
    params(("id" = i64, Path, description = "Expense id")),
    responses(
        (status = 200, description = "Invoice bytes"),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 404, description = "Expense or invoice not found", body = ErrorBody),
    )
)]
pub async fn download_invoice(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
  let row = match find_expense(&state, id).await {
    Ok(row) => row,
    Err(response) => return response,
  };
  match (row.invoice_blob.clone(), row.invoice_mime.clone()) {
    (Some(bytes), Some(mime)) => {
      let mut headers = HeaderMap::new();
      if let (Ok(content_type), Ok(disposition)) = (
        mime.parse::<header::HeaderValue>(),
        format!("inline; filename=\"{}\"", row.invoice_name.as_deref().unwrap_or("invoice"))
          .parse::<header::HeaderValue>(),
      ) {
        headers.insert(header::CONTENT_TYPE, content_type);
        headers.insert(header::CONTENT_DISPOSITION, disposition);
      }
      (headers, bytes).into_response()
    }
    _ => error_response(StatusCode::NOT_FOUND, "no invoice attached"),
  }
}

/// `DELETE /api/expenses/{id}/invoice` — remove the attached invoice blob.
#[utoipa::path(
    delete,
    path = "/api/expenses/{id}/invoice",
    operation_id = "expenses.deleteInvoice",
    tag = "expenses",
    security(("session_cookie" = [])),
    params(("id" = i64, Path, description = "Expense id")),
    responses(
        (status = 200, description = "Invoice removed", body = ExpenseDto),
        (status = 401, description = "Not authenticated", body = ErrorBody),
        (status = 403, description = "Not an admin", body = ErrorBody),
        (status = 404, description = "Expense not found", body = ErrorBody),
    )
)]
pub async fn delete_invoice(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
  let row = match find_expense(&state, id).await {
    Ok(row) => row,
    Err(response) => return response,
  };
  let source_name = expense_source::Entity::find_by_id(&row.source_key)
    .one(&state.db)
    .await
    .ok()
    .flatten()
    .map(|s| s.name)
    .unwrap_or_else(|| row.source_key.clone());
  let mut model: expense::ActiveModel = row.into();
  model.invoice_blob = Set(None);
  model.invoice_mime = Set(None);
  model.invoice_name = Set(None);
  model.updated_at = Set(chrono::Utc::now().fixed_offset());
  match model.update(&state.db).await {
    Ok(row) => Json(expense_dto(row, &source_name)).into_response(),
    Err(e) => internal_error(e),
  }
}

/* ---------------- helpers ---------------- */

fn bad_request(message: impl Into<String>) -> Response {
  error_response(StatusCode::BAD_REQUEST, message)
}

fn internal_error(e: DbErr) -> Response {
  eprintln!("database error: {e}");
  error_response(StatusCode::INTERNAL_SERVER_ERROR, "database error")
}

fn is_unique_violation(e: &DbErr) -> bool {
  matches!(
    e.sql_err(),
    Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
  )
}

/// FK-violation detection. SeaORM maps Postgres `23503` for us, but our
/// `ON DELETE RESTRICT` fires as `23001` (restrict_violation, raised by the
/// RI trigger), which SeaORM doesn't classify — hence the message fallback.
fn is_fk_violation(e: &DbErr) -> bool {
  if matches!(
    e.sql_err(),
    Some(sea_orm::SqlErr::ForeignKeyConstraintViolation(_))
  ) {
    return true;
  }
  e.to_string().contains("violates RESTRICT")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn source_key_normalization() {
    assert_eq!(normalize_source_key(" Vercel ").unwrap(), "vercel");
    assert!(normalize_source_key("").is_err());
    assert!(normalize_source_key("has space").is_err());
    assert!(normalize_source_key("UPPER").is_ok());
  }

  #[test]
  fn date_and_month_parsing() {
    assert!(parse_date("2026-10-05").is_ok());
    for bad in ["05/10/2026", "2026-13-01", "not-a-date", "2026-1-5"] {
      assert!(parse_date(bad).is_err(), "{bad} should be rejected");
    }
    let (start, end) = parse_month("2026-02").unwrap();
    assert_eq!(start.to_string(), "2026-02-01");
    assert_eq!(end.to_string(), "2026-03-01");
    let (_, end) = parse_month("2026-12").unwrap();
    assert_eq!(end.to_string(), "2027-01-01");
    assert!(parse_month("2026-13").is_err());
    assert!(parse_month("october").is_err());
  }

  #[test]
  fn money_and_label_validation() {
    assert!(normalize_money(-1, "inr_paise").is_err());
    assert_eq!(normalize_money(0, "inr_paise").unwrap(), 0);
    assert_eq!(normalize_label(Some("  ")).unwrap(), None);
    assert!(normalize_label(Some(&"x".repeat(121))).is_err());
  }
}
