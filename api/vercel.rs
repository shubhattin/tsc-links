use tower::ServiceBuilder;
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;

#[tokio::main]
async fn main() -> Result<(), Error> {
  let state = tsc_links::app::init_state()
    .await
    .map_err(|e| -> Error { Box::new(std::io::Error::other(e)) })?;
  let router = tsc_links::app::router(state);

  let app = ServiceBuilder::new()
    .layer(VercelLayer::new())
    .service(router);
  vercel_runtime::run(app).await
}
