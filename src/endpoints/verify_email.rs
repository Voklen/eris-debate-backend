use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use log::{error, info, warn};
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
	let id = delete_verification_token(token, db_pool, req).await?;
	mark_user_as_verified(id, token, db_pool).await?;
	delete_unverified_user(id, token, db_pool).await;
	Ok(())
}

async fn delete_verification_token(
	token: &str,
	db_pool: &sqlx::Pool<sqlx::Postgres>,
	req: &HttpRequest,
) -> Result<i64, HttpResponse> {
	let result = sqlx::query!(
		"DELETE FROM email_verification_tokens WHERE token = $1 RETURNING id",
		token,
	)
	.fetch_one(db_pool)
	.await;
	match result {
		Ok(res) => Ok(res.id),
		Err(_) => {
			match req.peer_addr() {
				Some(ip) => info!("Failed email verification attempt from ip={ip}"),
				None => warn!("Failed email verification attempt from unknown ip"),
			};
			Err(unauthorized!("Incorrect or expired token"))
		}
	}
}

async fn mark_user_as_verified(
	id: i64,
	token: &str,
	db_pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), HttpResponse> {
	let result = sqlx::query!(
		"
			WITH user_details AS (
				SELECT email, username, password_hash
				FROM unverified_users
				WHERE id = $1
			)
			INSERT INTO users(email, username, password_hash)
			SELECT user_details.email, user_details.username, user_details.password_hash
			FROM user_details
		",
		id
	)
	.execute(db_pool)
	.await;
	match result {
		Ok(res) => {
			let rows = res.rows_affected();
			if rows != 1 {
				warn!("Unexpected number of rows affected: {rows}");
				// Return success to user but log unexpected rows affected
			};
			Ok(())
		}
		Err(e) => {
			warn!("Error updating user to verified (id={id}, token={token}): {e}");
			Err(internalServerError!(
				"Unable to verify email (don't worry, we're working on it)"
			))
		}
	}
}

async fn delete_unverified_user(id: i64, token: &str, db_pool: &sqlx::Pool<sqlx::Postgres>) {
	let result = sqlx::query!("DELETE FROM unverified_users WHERE id=$1", id)
		.execute(db_pool)
		.await;
	// Return success to user but log anything unexpected when deleting the users
	match result {
		Ok(res) => {
			let rows = res.rows_affected();
			if rows != 1 {
				error!("Unexpected number of users deleted: {rows}");
			};
		}
		Err(e) => {
			warn!("Error updating user to verified (id={id}, token={token}): {e}");
		}
	};
}
