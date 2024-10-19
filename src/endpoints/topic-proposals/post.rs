use actix_web::{post, web, HttpResponse, Responder};
use log::warn;
use serde::Deserialize;
use sqlx::{postgres::PgQueryResult, PgPool};

use crate::{internalServerError, session_helper::check_session, unwrap_or_esalate, AppState};

#[derive(Deserialize)]
struct TopicPostRequest {
	name: String,
	for_argument: String,
	against_argument: String,
	reason: String,
}

#[post("/topic-proposals")]
async fn post_topic_proposals(
	json: web::Json<TopicPostRequest>,
	req: actix_web::HttpRequest,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let check_result = check_session(&req, &app_state.dbpool).await;
	let id = unwrap_or_esalate!(check_result);
	let res = submit_topic_proposal(id, json, &app_state.dbpool).await;
	match res {
		Ok(()) => return HttpResponse::Ok().finish(),
		Err(http_response) => http_response,
	}
}

async fn submit_topic_proposal(
	user_id: i64,
	request: web::Json<TopicPostRequest>,
	db_pool: &PgPool,
) -> Result<(), HttpResponse> {
	let result = sqlx::query!(
		"
		INSERT INTO topic_proposals (name, created_by, for_argument, against_argument, reason)
		VALUES ($1, $2, $3, $4, $5)
		",
		request.name,
		user_id,
		request.for_argument,
		request.against_argument,
		request.reason,
	)
	.execute(db_pool)
	.await;
	check_errors(result)
}

fn check_errors(result: Result<PgQueryResult, sqlx::Error>) -> Result<(), HttpResponse> {
	let res = match result {
		Ok(res) => Ok(res),
		Err(e) => {
			warn!("Error creating topic proposal: {e}");
			Err(internalServerError!("Error submitting topic proposal"))
		}
	}?;
	let rows = res.rows_affected();
	if rows != 1 {
		warn!("Unexpected number of rows affected: {rows}");
		// Return success to user but log unexpected rows affected
	};
	Ok(())
}
