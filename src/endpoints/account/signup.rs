use crate::internalServerError;
use actix_web::{post, web, HttpResponse, Responder};
use argon2::{
	password_hash::{rand_core::OsRng, SaltString},
	Argon2, PasswordHasher,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::postgres::PgQueryResult;

use crate::AppState;

#[derive(Deserialize)]
struct CreateUserRequest {
	email: String,
	password: String,
}

#[post("/account/signup")]
async fn signup_endpoint(
	request: web::Json<CreateUserRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let argon2 = Argon2::default();
	let salt = SaltString::generate(&mut OsRng);
	let password_bytes = request.password.as_bytes();

	let password_hash = match argon2.hash_password(password_bytes, &salt) {
		Ok(hash) => hash.serialize(),
		Err(e) => return internalServerError!("Error hashing password: {e}"),
	};
	let result = sqlx::query!(
		"INSERT INTO users(email, password_hash) VALUES ($1, $2);",
		request.email,
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
	let body = json!({
		"token": 0
	});
	match res.rows_affected() {
		1 => HttpResponse::Ok().body(body.to_string()),
		rows => {
			internalServerError!("{rows} rows affected")
		}
	}
}
