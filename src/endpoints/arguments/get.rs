use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

use crate::{arguments_helper::get_response_arguments, AppState};

#[derive(Deserialize)]
struct ArgumentsRequest {
	id: i64,
}

#[get("/arguments")]
async fn get_arguments_endpoint(
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
