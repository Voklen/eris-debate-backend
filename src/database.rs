use argon2::password_hash::SaltString;
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
	let url = env::var("DATABASE_URL").expect("env variable DATABASE_URL should be set");
	let pepper =
		env::var("SESSION_TOKEN_PEPPER").expect("env variable SESSION_TOKEN_PEPPER should be set");
	SaltString::from_b64(&pepper).expect("env variable SESSION_TOKEN_PEPPER cannot be parsed");
	PgPoolOptions::new()
		.max_connections(5)
		.connect(&url)
		.await
		.unwrap()
}
