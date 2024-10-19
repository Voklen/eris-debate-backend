use actix_web::{put, web, HttpResponse, Responder};
use log::warn;
use serde::Deserialize;
use sqlx::{postgres::PgQueryResult, PgPool};

use crate::{
	admin_helper::is_admin, badRequest, internalServerError, session_helper::check_session,
	unauthorized, unwrap_or_esalate, AppState,
};

#[derive(Deserialize)]
enum Action {
	#[serde(rename = "accept")]
	Accept,
	#[serde(rename = "reject")]
	Reject,
}

#[derive(Deserialize)]
struct TopicPutRequest {
	action: Action,
}

#[put("/topic_suggestion/{proposal_id}")]
async fn put_topic_suggestion_endpoint(
	json: web::Json<TopicPutRequest>,
	proposal_id: web::Path<String>,
	req: actix_web::HttpRequest,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let check_result = check_session(&req, &app_state.dbpool).await;
	let user_id = unwrap_or_esalate!(check_result);
	let is_admin_result = is_admin(user_id, &app_state.dbpool).await;
	let is_admin = unwrap_or_esalate!(is_admin_result);
	if !is_admin {
		return unauthorized!("You must be an admin to perform this action");
	}
	match json.action {
		Action::Accept => submit_topic_proposal(user_id, &proposal_id, &app_state.dbpool).await,
		Action::Reject => todo!(),
	}
}

async fn submit_topic_proposal(
	user_id: i64,
	proposal_id_string: &str,
	db_pool: &PgPool,
) -> HttpResponse {
	let proposal_id: i64 = match proposal_id_string.parse() {
		Ok(id) => id,
		Err(e) => return badRequest!("Proposal ID is not an integer: {e}"),
	};
	let result = sqlx::query!(
		"
			WITH proposal AS (
				SELECT name, for_argument, against_argument
				FROM topic_proposals
				WHERE id = $1
			),
			new_for_rev AS (
				INSERT INTO revisions (body, revision_by)
				SELECT for_argument, $2 FROM proposal
				RETURNING id
			),
			new_for_arg AS (
				INSERT INTO arguments (revision_latest)
				SELECT id FROM new_for_rev
				RETURNING id
			),
			new_against_rev AS (
				INSERT INTO revisions (body, revision_by)
				SELECT against_argument, $2 FROM proposal
				RETURNING id
			),
			new_against_arg AS (
				INSERT INTO arguments (revision_latest)
				SELECT id FROM new_against_rev
				RETURNING id
			)
			INSERT INTO topics (name, for_argument, against_argument)
			SELECT proposal.name, new_for_arg.id, new_against_arg.id
			FROM proposal, new_for_arg, new_against_arg;
		",
		proposal_id,
		user_id,
	)
	.execute(db_pool)
	.await;
	//TODO mark proposal as accepted
	check_errors(result)
}

fn check_errors(result: Result<PgQueryResult, sqlx::Error>) -> HttpResponse {
	let res = match result {
		Ok(res) => res,
		Err(e) => {
			warn!("Error creating topic proposal: {e}");
			return internalServerError!("Error accepting topic proposal");
		}
	};
	let rows = res.rows_affected();
	if rows != 1 {
		warn!("Unexpected number of rows affected: {rows}");
		// Return success to user but log unexpected rows affected
	};
	HttpResponse::Ok().finish()
}
