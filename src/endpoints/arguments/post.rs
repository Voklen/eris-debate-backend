use actix_web::{cookie::Cookie, http::header::ContentType, post, web, HttpResponse, Responder};
use base64::{engine, Engine};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{internalServerError, unauthorized, unwrap_or_esalate, AppState};

#[derive(Deserialize)]
struct ArgumentsRequest {
	parent: i64,
	body: String,
}

#[derive(Serialize)]
struct TopArgument {
	id: i64,
	body: String,
}

#[post("/arguments")]
async fn post_arguments_endpoint(
	json: web::Json<ArgumentsRequest>,
	req: actix_web::HttpRequest,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let cookie = match req.cookie("session_token") {
		Some(cookie) => cookie,
		None => return unauthorized!("Unauthorized"),
	};
	let id_result = check_session(cookie, &app_state.dbpool).await;
	let id = unwrap_or_esalate!(id_result);
	let res = create_argument(id, json, &app_state.dbpool).await;
	match res {
		Ok(()) => {
			return HttpResponse::Ok()
				.content_type(ContentType::json())
				.body("")
		}
		Err(http_response) => http_response,
	}
}

/// Verifies the session is valid in the database and returns the id of the corresponding user
async fn check_session(cookie: Cookie<'_>, db_pool: &PgPool) -> Result<i64, HttpResponse> {
	let base64_decoder = engine::general_purpose::URL_SAFE;
	let decoded_cookie = base64_decoder.decode(cookie.value()).unwrap();
	let result = sqlx::query!(
		"SELECT id FROM session_tokens WHERE token = $1",
		decoded_cookie
	)
	.fetch_one(db_pool)
	.await;
	match result {
		Ok(res) => Ok(res.id),
		Err(sqlx::Error::RowNotFound) => {
			Err(unauthorized!("Session token does not exist or has expired"))
		}
		Err(e) => Err(internalServerError!("Error retrieving user data: {e}")),
	}
}

async fn create_argument(
	id: i64,
	request: web::Json<ArgumentsRequest>,
	db_pool: &PgPool,
) -> Result<(), HttpResponse> {
	let result = sqlx::query!(
		"INSERT INTO arguments (parent, body, created_by) VALUES ($1, $2, $3)",
		request.parent,
		request.body,
		id,
	)
	.execute(db_pool)
	.await;
	let res = match result {
		Ok(res) => Ok(res),
		Err(e) => Err(internalServerError!("Error retrieving user data: {e}")),
	}?;
	if res.rows_affected() != 1 {
		println!(
			"Unexpected number of rows affected: {}",
			res.rows_affected()
		);
		return Err(internalServerError!("Unexpected number of rows affected"));
	};
	Ok(())
}
