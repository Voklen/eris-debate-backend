use actix_web::cookie::{Cookie, CookieBuilder};
use actix_web::{post, web, HttpResponse, Responder};
use base64::{engine, Engine};
use log::{error, warn};
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;

use crate::hashing_helper::session_token_hash;
use crate::{badRequest, internalServerError, unwrap_or_esalate};
use crate::{unauthorized, AppState};

#[post("/logout")]
async fn logout_endpoint(
	req: actix_web::HttpRequest,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let cookie = match req.cookie("session_token") {
		Some(cookie) => cookie,
		None => return unauthorized!("Unauthorized"),
	};
	let deletion_result = delete_session_token(cookie, &app_state.dbpool).await;
	unwrap_or_esalate!(deletion_result);
	let mut cookie = CookieBuilder::new("session_token", "")
		.secure(true)
		.same_site(actix_web::cookie::SameSite::None)
		.http_only(true)
		.finish();
	cookie.make_removal();
	HttpResponse::Ok().cookie(cookie).finish()
}

async fn delete_session_token(cookie: Cookie<'_>, db_pool: &PgPool) -> Result<(), HttpResponse> {
	let base64_decoder = engine::general_purpose::URL_SAFE;
	let cookie_string = cookie.value();
	let token = base64_decoder
		.decode(cookie_string)
		.map_err(|_| badRequest!("Session token is invalid base64"))?;
	let token_hash = session_token_hash(&token)?;
	let result = sqlx::query!(
		"DELETE FROM session_tokens WHERE token_hash=$1;",
		token_hash.to_string()
	)
	.execute(db_pool)
	.await;
	check_errors(result)
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
