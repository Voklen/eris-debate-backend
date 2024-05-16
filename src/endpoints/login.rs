use actix_web::cookie::CookieBuilder;
use actix_web::{post, web, HttpResponse, Responder};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::{engine, Engine};
use serde::Deserialize;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;

use crate::AppState;
use crate::{badRequest, internalServerError, unwrap_or_esalate};

#[derive(Deserialize)]
struct LoginRequest {
	email: String,
	password: String,
}

#[post("/login")]
async fn login_endpoint(
	form: web::Json<LoginRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let entered_password = &form.password;
	let id_and_password_result = get_id_and_password(&form.email, &app_state.dbpool).await;
	let (id, stored_password) = unwrap_or_esalate!(id_and_password_result);

	let check_password_result = check_password(entered_password, &stored_password);
	let is_correct_password = unwrap_or_esalate!(check_password_result);
	if !is_correct_password {
		return badRequest!("Incorrect password");
	}
	let session_token_result = create_session_token(id, &app_state.dbpool).await;
	let session_token = unwrap_or_esalate!(session_token_result);
	let cookie = CookieBuilder::new("session_token", session_token)
		.secure(true)
		.same_site(actix_web::cookie::SameSite::None)
		.http_only(true)
		.finish();
	HttpResponse::Ok().cookie(cookie).finish()
}

async fn get_id_and_password(email: &str, db_pool: &PgPool) -> Result<(i64, String), HttpResponse> {
	let result = sqlx::query!("SELECT id, password_hash FROM users WHERE email=$1;", email)
		.fetch_one(db_pool)
		.await;
	match result {
		Ok(res) => Ok((res.id, res.password_hash)),
		Err(sqlx::Error::RowNotFound) => {
			Err(badRequest!("An account with this email does not exist"))
		}
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

async fn create_session_token(id: i64, db_pool: &PgPool) -> Result<String, HttpResponse> {
	let mut token = [0u8; 16];
	OsRng.fill_bytes(&mut token);
	let result = sqlx::query!(
		"INSERT INTO session_tokens(id, token) VALUES ($1, $2);",
		id,
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
