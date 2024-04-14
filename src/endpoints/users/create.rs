use crate::internalServerError;
use actix_web::{post, web, HttpResponse, Responder};
use argon2::{
	password_hash::{rand_core::OsRng, SaltString},
	Argon2, PasswordHasher,
};
use serde::Deserialize;
use sqlx::postgres::PgQueryResult;

use crate::AppState;

#[derive(Deserialize)]
struct CreateUserRequest {
	username: String,
	password: String,
}

#[post("/users/create")]
async fn create_user_endpoint(
	form: web::Form<CreateUserRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let argon2 = Argon2::default();
	let salt = SaltString::generate(&mut OsRng);
	let password_bytes = form.password.as_bytes();

	let password_hash = match argon2.hash_password(password_bytes, &salt) {
		Ok(hash) => hash.serialize(),
		Err(e) => return internalServerError!("Error hashing password: {e}"),
	};
	let result = sqlx::query!(
		"INSERT INTO users(username, password_hash) VALUES ($1, $2);",
		form.username,
		password_hash.as_str()
	)
	.execute(&app_state.dbpool)
	.await;
	check_errors(result)
}

fn check_errors(result: Result<PgQueryResult, sqlx::Error>) -> HttpResponse {
	match result {
		Ok(res) => success(res),
		Err(sqlx::Error::Database(db_error)) => {
			let unique_violation_error_code = Some(std::borrow::Cow::Borrowed("23505"));
			if db_error.code() == unique_violation_error_code {
				return internalServerError!("User already exists");
			};
			internalServerError!("Database error: {}", db_error.message())
		}
		Err(e) => internalServerError!("Error inserting query: {e}"),
	}
}

fn success(res: PgQueryResult) -> HttpResponse {
	match res.rows_affected() {
		1 => HttpResponse::Ok().into(),
		rows => {
			internalServerError!("{rows} rows affected")
		}
	}
}
