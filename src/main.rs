use actix_cors::Cors;
use actix_web::dev::RequestHead;
use actix_web::http::header::HeaderValue;
use actix_web::{web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
use std::env;

use crate::arguments::arguments_endpoint;
use crate::create_user::create_user_endpoint;
use crate::login::login_endpoint;

mod errors;

#[path = "endpoints/arguments.rs"]
mod arguments;
#[path = "endpoints/users/create.rs"]
mod create_user;
#[path = "endpoints/users/login.rs"]
mod login;

#[derive(Clone)]
struct AppState {
	dbpool: PgPool,
}

const PORT: u16 = 9000;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let app_state = init_app_state().await;
	let server = HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(app_state.clone()))
			.wrap(get_cors())
			.service(arguments_endpoint)
			.service(create_user_endpoint)
			.service(login_endpoint)
	})
	.bind(("0.0.0.0", PORT))?
	.run();
	println!("Server initialised on port {PORT}!");
	server.await
}

async fn init_app_state() -> AppState {
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
	sqlx::query!(
		"
		CREATE TABLE IF NOT EXISTS users(
			id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
			username TEXT NOT NULL UNIQUE,
			password_hash TEXT NOT NULL
		);
		"
	)
	.execute(dbpool)
	.await?;
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
	.execute(dbpool)
	.await?;
	Ok(())
}

fn get_cors() -> Cors {
	Cors::default()
		.allowed_origin("http://localhost:3000")
		.allowed_methods(["GET", "POST"])
}
