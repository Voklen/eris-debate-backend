use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::rc::Rc;

use crate::{badRequest, internalServerError, AppState};

#[derive(Deserialize)]
struct ArgumentsRequest {
	id: i64,
}

#[derive(Serialize)]
struct TopArgument {
	id: i64,
	body: String,
}

#[get("/arguments")]
async fn get_arguments_endpoint(
	_req: HttpRequest,
	arguments_req: web::Query<ArgumentsRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let id = arguments_req.id;
	let res = match get_response_arguments(id, &app_state.dbpool).await {
		Ok(res) => res,
		Err(http_response) => return http_response,
	};

	let body = json!({
		"args": res
	});

	HttpResponse::Ok().body(body.to_string())
}

async fn get_response_arguments(
	argument_id: i64,
	db_pool: &PgPool,
) -> Result<Rc<[TopArgument]>, HttpResponse> {
	let result = sqlx::query!(
		"SELECT id, body FROM arguments WHERE parent = $1",
		argument_id
	)
	.fetch_all(db_pool)
	.await;
	let res = match result {
		Ok(res) => Ok(res),
		Err(sqlx::Error::RowNotFound) => Err(badRequest!("User not found")),
		Err(e) => Err(internalServerError!("Error retrieving user data: {e}")),
	}?;
	let arg_vec = res
		.into_iter()
		.map(|arg| TopArgument {
			id: arg.id,
			body: arg.body,
		})
		.collect();
	Ok(arg_vec)
}
