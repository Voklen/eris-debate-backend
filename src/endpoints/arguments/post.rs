use crate::{internalServerError, session_helper::check_session, unwrap_or_esalate, AppState};
use actix_web::{post, web, HttpResponse, Responder};
use log::warn;
use serde::Deserialize;
use sqlx::{postgres::PgQueryResult, PgPool};

#[derive(Deserialize)]
struct ArgumentsRequest {
	parent: i64,
	body: String,
}

#[post("/arguments")]
async fn post_arguments_endpoint(
	json: web::Json<ArgumentsRequest>,
	req: actix_web::HttpRequest,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let check_result = check_session(&req, &app_state.dbpool).await;
	let id = unwrap_or_esalate!(check_result);
	let res = create_argument(id, json, &app_state.dbpool).await;
	match res {
		Ok(()) => return HttpResponse::Ok().finish(),
		Err(http_response) => http_response,
	}
}

async fn create_argument(
	user_id: i64,
	request: web::Json<ArgumentsRequest>,
	db_pool: &PgPool,
) -> Result<(), HttpResponse> {
	let result = sqlx::query!(
		"
		WITH new_rev AS (
			INSERT INTO revisions (body, revision_by) VALUES ($2, $3) RETURNING id
		)
		INSERT INTO arguments (revision_latest, parent)
		SELECT id, $1
		FROM new_rev;
		",
		request.parent,
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
			warn!("Error creating argument: {e}");
			Err(internalServerError!("Error creating argument"))
		}
	}?;
	let rows = res.rows_affected();
	if rows != 1 {
		warn!("Unexpected number of rows affected: {rows}");
		// Return success to user but log unexpected rows affected
	};
	Ok(())
}
