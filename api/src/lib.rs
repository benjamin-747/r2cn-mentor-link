mod confernece_router;
mod model;
mod task_router;

use std::env;

use axum::Router;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use service::{storage::task_stg::TaskStorage, Context};
use tower_cookies::CookieManagerLayer;

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");
    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");

    Migrator::up(&conn, None).await.unwrap();
    let context = Context::new(conn.into()).await;
    let state = AppState { context };

    let api_router = Router::new()
        .merge(confernece_router::routers())
        .merge(task_router::routers());

    let app = Router::new()
        .nest("/api/v1/", api_router)
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    context: Context,
}

impl AppState {
    fn task_stg(&self) -> TaskStorage {
        self.context.services.task_stg.clone()
    }
}

pub fn main() {
    let result = start();
    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
