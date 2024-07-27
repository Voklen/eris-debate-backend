use actix_web::{post, web, HttpResponse, Responder};
use log::warn;
use serde::Deserialize;
use sqlx::PgPool;

use crate::internalServerError;
use crate::{unauthorized, AppState};

#[derive(Deserialize)]
struct VerifyEmailRequest {
	token: String,
}

#[post("/verifyemail")]
async fn verify_email_endpoint(
	form: web::Json<VerifyEmailRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let token = &form.token;
	match is_correct_token(token, &app_state.dbpool).await {
		Ok(true) => HttpResponse::Ok().finish(),
		Ok(false) => unauthorized!("Incorrect or expired token"),
		Err(e) => e,
	}
}

pub async fn is_correct_token(token: &str, db_pool: &PgPool) -> Result<bool, HttpResponse> {
	let result = sqlx::query!(
		"SELECT EXISTS(SELECT 1 FROM users WHERE verification_token=$1);",
		token,
	)
	.fetch_one(db_pool)
	.await;
	match result {
		Ok(res) => Ok(res.exists.unwrap_or(false)),
		Err(e) => {
			warn!("Error checking if token is correct(token={token}): {e}");
			Err(internalServerError!("Unknown error 3"))
		}
	}
}
