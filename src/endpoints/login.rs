use actix_web::cookie::CookieBuilder;
use actix_web::{post, web, HttpResponse, Responder};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64::{engine, Engine};
use log::{error, warn};
use serde::Deserialize;
use serde_json::json;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;

use crate::hashing_helper::{check_hashes, session_token_hash};
use crate::AppState;
use crate::{badRequest, internalServerError, unwrap_or_esalate};

#[derive(Deserialize)]
struct LoginRequest {
	email: String,
	password: String,
}

struct User {
	id: i64,
	username: String,
	password: String,
}

#[post("/login")]
async fn login_endpoint(
	form: web::Json<LoginRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let entered_password = &form.password;
	let id_and_password_result = get_id_and_password(&form.email, &app_state.dbpool).await;
	let stored_user = unwrap_or_esalate!(id_and_password_result);

	let check_password_result = check_hashes(entered_password.as_bytes(), &stored_user.password);
	let is_correct_password = unwrap_or_esalate!(check_password_result);
	if !is_correct_password {
		return badRequest!("Incorrect password");
	}
	let session_token_result = create_session_token(stored_user.id, &app_state.dbpool).await;
	let session_token = unwrap_or_esalate!(session_token_result);
	let cookie = CookieBuilder::new("session_token", session_token)
		.secure(true)
		.same_site(actix_web::cookie::SameSite::None)
		.http_only(true)
		.finish();
	let roles = get_roles(stored_user.id, &app_state.dbpool).await;
	let body = json!({
		"username": stored_user.username,
		"roles": roles,
	});
	HttpResponse::Ok().cookie(cookie).body(body.to_string())
}

async fn get_id_and_password(email: &str, db_pool: &PgPool) -> Result<User, HttpResponse> {
	let result = sqlx::query!(
		"SELECT id, username, password_hash FROM users WHERE email = $1",
		email
	)
	.fetch_one(db_pool)
	.await;
	match result {
		Ok(res) => Ok(User {
			id: res.id,
			username: res.username,
			password: res.password_hash,
		}),
		Err(sqlx::Error::RowNotFound) => {
			Err(badRequest!("An account with this email does not exist"))
		}
		Err(e) => {
			warn!("Unexpected error in login: {e}");
			Err(internalServerError!("Error retrieving user data"))
		}
	}
}

async fn create_session_token(id: i64, db_pool: &PgPool) -> Result<String, HttpResponse> {
	let mut token = [0u8; 16];
	OsRng.fill_bytes(&mut token);
	let token_hash = session_token_hash(&token)?;

	let result = sqlx::query!(
		"INSERT INTO session_tokens(id, token_hash) VALUES ($1, $2)",
		id,
		token_hash.to_string()
	)
	.execute(db_pool)
	.await;
	check_errors(result)?;
	let base64_encoder = engine::general_purpose::URL_SAFE;
	Ok(base64_encoder.encode(token))
}

async fn get_roles(id: i64, db_pool: &PgPool) -> Vec<String> {
	let result = sqlx::query!("SELECT role FROM roles WHERE id = $1", id)
		.fetch_all(db_pool)
		.await;
	match result {
		Ok(res) => res.into_iter().map(|r| r.role).collect(),
		Err(e) => {
			error!("Unexpected error in getting roles(id={id}): {e}");
			// Silently ignore this error, to keep everything working
			// while I work out what's going on
			Vec::new()
		}
	}
}

fn check_errors(result: Result<PgQueryResult, sqlx::Error>) -> Result<(), HttpResponse> {
	let res = match result {
		Ok(res) => Ok(res),
		Err(e) => {
			warn!("Error saving session token: {e}");
			Err(internalServerError!("Error saving session token"))
		}
	}?;
	let rows = res.rows_affected();
	if rows != 1 {
		error!("{rows} rows affected when saving session token")
	};
	Ok(())
}
