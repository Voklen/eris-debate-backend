use actix_web::{get, http::header::Header, web, HttpRequest, HttpResponse, Responder};
use actix_web_httpauth::headers::authorization::{Authorization, Basic};
use base64::{engine, Engine};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::{internalServerError, unauthorized, unwrap_or_esalate, AppState};

#[derive(Deserialize)]
struct ArgumentsRequest {
	// title: String,
}

#[get("/arguments")]
async fn arguments_endpoint(
	req: HttpRequest,
	title_req: web::Query<ArgumentsRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	// let auth = match Authorization::<Basic>::parse(&req) {
	// 	Ok(res) => res,
	// 	Err(e) => return unauthorized!("Authorization header error: {e}"),
	// };
	// let email = auth.as_ref().user_id();
	// let token = auth.as_ref().password();
	// let authorization_result = check_authorization(email, token, &app_state.dbpool).await;
	// let is_authorized = unwrap_or_esalate!(authorization_result);
	// if !is_authorized {
	// 	return unauthorized!("Incorrect username or key");
	// }
	// let title = &title_req.title;
	let short_title = format!("Hello from Actix Rust with Postgresql!");
	let body = json!([{"body": short_title}]);
	HttpResponse::Ok().body(body.to_string())
}

/// Returns `Ok(true)` if authorized, `Ok(false)` if unauthorized and `Err(_)` on error
async fn check_authorization(
	username: &str,
	session_token: Option<&str>,
	db_pool: &PgPool,
) -> Result<bool, HttpResponse> {
	let token = session_token.ok_or(unauthorized!("No session token provided"))?;
	let base64_decoder = engine::general_purpose::URL_SAFE;
	let token_bytes = base64_decoder
		.decode(token)
		.or_else(|e| Err(unauthorized!("Session token decode error: {e}")))?;
	let result = sqlx::query!(
		r#"SELECT COUNT(1) AS "count!" FROM session_tokens WHERE username=$1 AND token=$2;"#,
		username,
		token_bytes
	)
	.fetch_one(db_pool)
	.await;
	match result {
		Ok(res) => Ok(res.count >= 1),
		Err(e) => Err(internalServerError!("Error retrieving user data: {e}")),
	}
}
