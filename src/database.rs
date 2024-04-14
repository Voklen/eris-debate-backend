use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
use std::env;

use crate::{throw, AppState};

pub async fn init_app_state() -> AppState {
	let dbpool = get_pool().await;
	init_db(&dbpool)
		.await
		.unwrap_or_else(|e| throw!("Error initializing database: {e}"));
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

async fn init_db(dbpool: &PgPool) -> sqlx::Result<()> {
	tokio::try_join!(
		sqlx::query!(
			"
		CREATE TABLE IF NOT EXISTS users(
			id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
			username TEXT NOT NULL UNIQUE,
			password_hash TEXT NOT NULL
		);
		"
		)
		.execute(dbpool),
		sqlx::query!(
			"
		CREATE TABLE IF NOT EXISTS session_tokens(
			username TEXT NOT NULL REFERENCES users(username)
				ON DELETE CASCADE
				ON UPDATE CASCADE,
			token BYTEA NOT NULL
		);
		"
		)
		.execute(dbpool),
		sqlx::query!(
			"
		CREATE TABLE IF NOT EXISTS arguments(
			id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
			parent BIGINT REFERENCES arguments(id),
			body TEXT NOT NULL
		);
		"
		)
		.execute(dbpool),
		sqlx::query!(
			"
		CREATE TABLE IF NOT EXISTS topics(
			name TEXT NOT NULL UNIQUE,
			id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
			for_argument BIGINT NOT NULL REFERENCES arguments(id),
			against_argument BIGINT NOT NULL REFERENCES arguments(id)
		);
		"
		)
		.execute(dbpool),
	)?;
	Ok(())
}
