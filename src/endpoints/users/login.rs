use actix_web::{post, web, HttpResponse, Responder};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::{engine, Engine};
use serde::Deserialize;
use serde_json::json;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;

use crate::AppState;
use crate::{badRequest, internalServerError, unwrap_or_esalate};

#[derive(Deserialize)]
struct LoginRequest {
	username: String,
	password: String,
}

#[post("/users/login")]
async fn login_endpoint(
	form: web::Form<LoginRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let entered_password = &form.password;
	let stored_password_result = get_stored_password(&form.username, &app_state.dbpool).await;
	let stored_password = unwrap_or_esalate!(stored_password_result);

	let check_password_result = check_password(entered_password, &stored_password);
	let is_correct_password = unwrap_or_esalate!(check_password_result);
	if !is_correct_password {
		return badRequest!("Password incorrect");
	}
	let session_token_result = create_session_token(&form.username, &app_state.dbpool).await;
	let session_token = unwrap_or_esalate!(session_token_result);
	let body = json!({"session_token": session_token});
	HttpResponse::Ok().body(body.to_string())
}

async fn get_stored_password(username: &str, db_pool: &PgPool) -> Result<String, HttpResponse> {
	let result = sqlx::query!(
		"SELECT password_hash FROM users WHERE username=$1;",
		username
	)
	.fetch_one(db_pool)
	.await;
	match result {
		Ok(res) => Ok(res.password_hash),
		Err(sqlx::Error::RowNotFound) => Err(badRequest!("User not found")),
		Err(e) => Err(internalServerError!("Error retrieving user data: {e}")),
	}
}

fn check_password(entered_password: &str, stored_password: &str) -> Result<bool, HttpResponse> {
	let hash = match PasswordHash::new(stored_password) {
		Ok(hash) => hash,
		Err(e) => return Err(internalServerError!("Invalid stored password hash: {e}")),
	};
	let password_bytes = entered_password.as_bytes();
	let is_correct_password = Argon2::default()
		.verify_password(password_bytes, &hash)
		.is_ok();
	Ok(is_correct_password)
}

async fn create_session_token(username: &str, db_pool: &PgPool) -> Result<String, HttpResponse> {
	let mut token = [0u8; 16];
	OsRng.fill_bytes(&mut token);
	let result = sqlx::query!(
		"INSERT INTO session_tokens(username, token) VALUES ($1, $2);",
		username,
		&token
	)
	.execute(db_pool)
	.await;
	check_errors(result)?;
	let base64_encoder = engine::general_purpose::URL_SAFE;
	Ok(base64_encoder.encode(token))
}

fn check_errors(result: Result<PgQueryResult, sqlx::Error>) -> Result<(), HttpResponse> {
	match result {
		Ok(res) => success(res),
		Err(e) => Err(internalServerError!("Error saving session token: {e}")),
	}
}

fn success(res: PgQueryResult) -> Result<(), HttpResponse> {
	match res.rows_affected() {
		1 => Ok(()),
		rows => Err(internalServerError!("{rows} rows affected")),
	}
}
