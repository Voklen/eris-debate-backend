use crate::email_helper::send_email;
use crate::general_helper::get_env;
use crate::hashing_helper::hash_string;
use crate::{badRequest, internalServerError};
use crate::{unwrap_or_esalate, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use log::{error, warn};
use rand::distributions::{Alphanumeric, DistString};
use serde::Deserialize;
use sqlx::error::DatabaseError;
use sqlx::PgPool;

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
		"INSERT INTO users(email, username, password_hash) VALUES ($1, $2, $3) RETURNING id",
		request.email,
		request.username,
		password_hash.as_str(),
	)
	.fetch_one(&app_state.dbpool)
	.await;

	let res = match result {
		Ok(res) => res,
		Err(sqlx::Error::Database(db_error)) => return check_errors(db_error),
		Err(e) => {
			warn!("Error creating account: {e}");
			return internalServerError!("Error creating account");
		}
	};
	let email_result = send_verification_email(res.id, &request.email, &app_state.dbpool).await;
	unwrap_or_esalate!(email_result);
	HttpResponse::Ok().finish()
}

async fn send_verification_email(
	id: i64,
	email: &str,
	dbpool: &PgPool,
) -> Result<(), HttpResponse> {
	// Generate a random token with `thread_rng` which is cryptographically secure
	let verification_token = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

	let result = sqlx::query!(
		"INSERT INTO email_verification_tokens(id, token) VALUES ($1, $2)",
		id,
		verification_token,
	)
	.execute(dbpool)
	.await;

	match result {
		Ok(_) => {} //TODO check number of rows affected
		Err(e) => {
			error!("Error saving verification token: {e}");
			return Err(internalServerError!("Error verifying email"));
		}
	};

	let frontend_url = get_env("FRONTEND_URL");

	let success = send_email(
		email,
		"Confirm account",
		format!(
			"
Your verification token is: {verification_token}
Please click this link to verify your email: {frontend_url}/verifyemail?token={verification_token}

If you did not sign up for {frontend_url} please ignore this email and do not click the link above.

Thank you!
			"
		),
	);
	if success {
		Ok(())
	} else {
		Err(badRequest!("Cannot send email to address: {email}"))
	}
}

fn check_errors(db_error: Box<dyn DatabaseError>) -> HttpResponse {
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
