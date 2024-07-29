use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use log::{info, warn};
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
	req: HttpRequest,
	form: web::Json<VerifyEmailRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let token = &form.token;
	match is_correct_token(token, &app_state.dbpool, &req).await {
		Ok(()) => HttpResponse::Ok().finish(),
		Err(e) => e,
	}
}

pub async fn is_correct_token(
	token: &str,
	db_pool: &PgPool,
	req: &HttpRequest,
) -> Result<(), HttpResponse> {
	let result = sqlx::query!(
		"DELETE FROM email_verification_tokens WHERE token = $1 RETURNING id",
		token,
	)
	.fetch_one(db_pool)
	.await;
	let id = match result {
		Ok(res) => Ok(res.id),
		//TODO add more fine-graned error checking
		Err(_) => {
			match req.peer_addr() {
				Some(ip) => info!("Failed email verification attempt from ip={ip}"),
				None => warn!("Failed email verification attempt from unknown ip"),
			};
			Err(unauthorized!("Incorrect or expired token"))
		}
	}?;
	let result = sqlx::query!("UPDATE users SET email_verified = true WHERE id = $1", id)
		.execute(db_pool)
		.await;
	match result {
		Ok(_) => Ok(()), //TODO check number of rows affected
		Err(e) => {
			warn!("Error updating user to verified (id={id}, token={token}): {e}");
			Err(internalServerError!(
				"Unable to verify email (don't worry, we're working on it)"
			))
		}
	}
}
