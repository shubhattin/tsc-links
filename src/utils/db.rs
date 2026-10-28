use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::env;

pub type DbPool = DatabaseConnection;

/// Connect to Postgres (Rustls-backed when using `postgresql://…` URLs, e.g. Neon).
pub async fn connect(database_url: &str) -> Result<DbPool, DbErr> {
  match env::var("DATABASE_SCHEMA")
    .ok()
    .filter(|s| !s.trim().is_empty())
  {
    Some(schema) => {
      let connect_options = ConnectOptions::new(database_url)
        .set_schema_search_path(schema)
        .to_owned();
      Database::connect(connect_options).await
    }
    None => Database::connect(database_url).await,
  }
}
