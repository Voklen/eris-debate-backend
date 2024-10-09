use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

use crate::{arguments_helper::get_response_arguments, badRequest, unwrap_or_esalate, AppState};

#[get("/arguments/{parent_id}")]
async fn get_arguments_endpoint(
	path: web::Path<String>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let id = match path.parse() {
		Ok(id) => id,
		Err(e) => return badRequest!("Parent ID is not an integer: {e}"),
	};
	let args_result = get_response_arguments(id, &app_state.dbpool).await;
	let args = unwrap_or_esalate!(args_result);

	let body = json!({
		"args": args
	});
	HttpResponse::Ok().body(body.to_string())
}
