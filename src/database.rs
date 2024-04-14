use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

use crate::AppState;

pub async fn init_app_state() -> AppState {
	dotenv().ok();
	let dbpool = get_pool().await;
	AppState { dbpool }
}

async fn get_pool() -> Pool<Postgres> {
	let url = env::var("DATABASE_URL").unwrap();
	PgPoolOptions::new()
		.max_connections(5)
		.connect(&url)
		.await
		.unwrap()
}
