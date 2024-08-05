use crate::{internalServerError, session_helper::check_session, unwrap_or_esalate, AppState};
use actix_web::{put, web, HttpResponse, Responder};
use log::warn;
use serde::Deserialize;
use sqlx::{postgres::PgQueryResult, PgPool};

#[derive(Deserialize)]
struct PutArgumentsRequest {
	arg_id: i64,
	body: String,
}

#[put("/arguments")]
async fn put_arguments_endpoint(
	json: web::Json<PutArgumentsRequest>,
	req: actix_web::HttpRequest,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let check_result = check_session(&req, &app_state.dbpool).await;
	let id = unwrap_or_esalate!(check_result);
	let res = update_argument(id, json, &app_state.dbpool).await;
	match res {
		Ok(()) => return HttpResponse::Ok().finish(),
		Err(http_response) => http_response,
	}
}

async fn update_argument(
	user_id: i64,
	request: web::Json<PutArgumentsRequest>,
	db_pool: &PgPool,
) -> Result<(), HttpResponse> {
	let result = sqlx::query!(
		"
		WITH prev_rev_id AS (
			SELECT revision_latest AS id FROM arguments WHERE id = $1
		), new_rev AS (
			INSERT INTO revisions (body, revision_by, prev_revision) VALUES ($2, $3, (SELECT id FROM prev_rev_id)) RETURNING id
		)
		UPDATE arguments
		SET revision_latest = new_rev.id
		FROM new_rev
		WHERE arguments.id = $1
		",
		request.arg_id,
		request.body,
		user_id,
	)
	.execute(db_pool)
	.await;
	check_errors(result)
}

fn check_errors(result: Result<PgQueryResult, sqlx::Error>) -> Result<(), HttpResponse> {
	let res = match result {
		Ok(res) => Ok(res),
		Err(e) => {
			warn!("Error editing argument: {e}");
			Err(internalServerError!("Error editing argument"))
		}
	}?;
	let rows = res.rows_affected();
	if rows != 1 {
		warn!("Unexpected number of rows affected: {rows}");
		// Return success to user but log unexpected rows affected
	};
	Ok(())
}
