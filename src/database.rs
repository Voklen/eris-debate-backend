use argon2::password_hash::SaltString;
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::{general_helper::get_env, AppState};

pub async fn init_app_state() -> AppState {
	dotenv().ok();
	let dbpool = get_pool().await;
	AppState { dbpool }
}

async fn get_pool() -> Pool<Postgres> {
	let url = get_env("DATABASE_URL");

	let peppr_env_var = "SESSION_TOKEN_PEPPER";
	let pepper = get_env(peppr_env_var);

	SaltString::from_b64(&pepper).expect(&format!("env variable {peppr_env_var} cannot be parsed"));
	PgPoolOptions::new()
		.max_connections(5)
		.connect(&url)
		.await
		.unwrap()
}
