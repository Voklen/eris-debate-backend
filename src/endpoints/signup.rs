use crate::hashing_helper::hash_string;
use crate::AppState;
use crate::{badRequest, internalServerError};
use actix_web::{post, web, HttpResponse, Responder};
use log::warn;
use serde::Deserialize;
use serde_json::json;
use sqlx::postgres::PgQueryResult;

#[derive(Deserialize)]
struct SignupRequest {
	email: String,
	username: String,
	password: String,
}

#[post("/signup")]
async fn signup_endpoint(
	request: web::Json<SignupRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let password_hash = match hash_string(&request.password) {
		Ok(res) => res,
		Err(e) => return e,
	};
	let result = sqlx::query!(
		"INSERT INTO users(email, username, password_hash) VALUES ($1, $2, $3);",
		request.email,
		request.username,
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
			if db_error.is_unique_violation() {
				match db_error.constraint() {
					Some("users_email_key") => {
						return badRequest!("An account with that email already exists")
					}
					Some("users_username_key") => {
						return badRequest!("An account with that username already exists")
					}
					_ => {}
				};
			};
			warn!("Database error on creating account: {}", db_error.message());
			internalServerError!("Error creating account")
		}
		Err(e) => {
			warn!("Error creating account: {e}");
			internalServerError!("Error creating account")
		}
	}
}

fn success(res: PgQueryResult) -> HttpResponse {
	let body = json!({
		"token": 0
	});
	let rows = res.rows_affected();
	if rows != 1 {
		warn!("Unexpected number of rows affected: {rows}");
		// Return success to user but log unexpected rows affected
	};
	HttpResponse::Ok().body(body.to_string())
}
