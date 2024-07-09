use crate::{hashing_helper::session_token_hash, internalServerError, unauthorized};
use actix_web::{cookie::Cookie, HttpResponse};
use base64::{engine, Engine};
use log::warn;
use sqlx::PgPool;

pub fn decode_cookie(cookie: &Cookie<'_>) -> Vec<u8> {
	let base64_decoder = engine::general_purpose::URL_SAFE;
	let cookie_string = cookie.value();
	base64_decoder.decode(cookie_string).unwrap()
}

/// Verifies the session is valid in the database and returns the id of the corresponding user
pub async fn check_session(
	req: &actix_web::HttpRequest,
	db_pool: &PgPool,
) -> Result<i64, HttpResponse> {
	let cookie = match req.cookie("session_token") {
		Some(cookie) => cookie,
		None => return Err(unauthorized!("Unauthorized: No cookie given")),
	};
	check_cookie(cookie, db_pool).await
}

async fn check_cookie(cookie: Cookie<'_>, db_pool: &PgPool) -> Result<i64, HttpResponse> {
	let token = decode_cookie(&cookie);
	let token_hash = session_token_hash(&token)?;
	let result = sqlx::query!(
		"SELECT id FROM session_tokens WHERE token_hash = $1",
		token_hash.as_str()
	)
	.fetch_one(db_pool)
	.await;
	match result {
		Ok(res) => Ok(res.id),
		Err(sqlx::Error::RowNotFound) => {
			Err(unauthorized!("Session token does not exist or has expired"))
		}
		Err(e) => {
			warn!("Error retrieving cookie (cookie={}): {e}", cookie.value());
			Err(internalServerError!("Error retrieving cookie"))
		}
	}
}
