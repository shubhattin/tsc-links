use std::env;
use tokio::net::TcpListener;

type DynError = Box<dyn std::error::Error + Send + Sync>;

/// Local dev port (frontend/proxy.ts forwards `/api/*` and `/{slug}` here).
const PORT: u16 = 5779;

#[tokio::main]
async fn main() -> Result<(), DynError> {
  let state = tsc_links::app::init_state().await?;
  let app = tsc_links::app::router(state);

  let host = env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0".into());
  let port = env::var("PORT").unwrap_or_else(|_| PORT.to_string());
  let addr = format!("{host}:{port}");
  let listener = TcpListener::bind(&addr).await?;
  eprintln!("listening on http://{}", listener.local_addr()?);

  axum::serve(listener, app).await?;
  Ok(())
}
