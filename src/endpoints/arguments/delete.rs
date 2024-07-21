use crate::{
	admin_helper::is_admin, internalServerError, session_helper::check_session, unauthorized,
	unwrap_or_esalate, AppState,
};
use actix_web::{delete, web, HttpResponse, Responder};
use log::warn;
use serde::Deserialize;
use sqlx::{postgres::PgQueryResult, PgPool};

#[derive(Deserialize)]
struct ArgumentDeleteRequest {
	argument_id: i64,
}

#[delete("/arguments")]
async fn delete_arguments_endpoint(
	json: web::Json<ArgumentDeleteRequest>,
	req: actix_web::HttpRequest,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let check_result = check_session(&req, &app_state.dbpool).await;
	let id = unwrap_or_esalate!(check_result);
	let is_admin_result = is_admin(id, &app_state.dbpool).await;
	let is_admin = unwrap_or_esalate!(is_admin_result);
	if !is_admin {
		return unauthorized!("You must be an admin to perform this action");
	}
	delete_argument(json.argument_id, &app_state.dbpool).await
}

async fn delete_argument(argument_id: i64, db_pool: &PgPool) -> HttpResponse {
	let result = sqlx::query!("DELETE FROM arguments WHERE id=$1", argument_id)
		.execute(db_pool)
		.await;
	check_errors(result)
}

/// This is an admin function so responses will reveal internal data on error
fn check_errors(result: Result<PgQueryResult, sqlx::Error>) -> HttpResponse {
	let res = match result {
		Ok(res) => res,
		Err(e) => {
			warn!("Error deleting argument: {e}");
			return internalServerError!("Error deleting argument: {e}");
		}
	};
	let rows = res.rows_affected();
	if rows != 1 {
		warn!("Unexpected number of rows affected: {rows}");
		return internalServerError!("Unexpected number of rows affected: {rows}");
	};
	HttpResponse::Ok().finish()
}
