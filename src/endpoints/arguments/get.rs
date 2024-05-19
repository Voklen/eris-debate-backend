use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;

use crate::{arguments_helper::get_response_arguments, unwrap_or_esalate, AppState};

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
	let args_result = get_response_arguments(id, &app_state.dbpool).await;
	let args = unwrap_or_esalate!(args_result);

	let body = json!({
		"args": args
	});
	HttpResponse::Ok().body(body.to_string())
}
