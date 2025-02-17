use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;
use tokio::try_join;

use crate::{
	badRequest,
	database::{arguments::get_response_arguments, topic::get_topic},
	unwrap_or_esalate, AppState,
};

#[get("/topic/{topic_id}")]
async fn get_topic_endpoint(
	path: web::Path<String>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let id = unwrap_or_esalate!(get_id(path));
	let topic_result = get_topic_json(id, &app_state.dbpool).await;
	let topic = unwrap_or_esalate!(topic_result);
	HttpResponse::Ok().body(topic)
}

fn get_id(path: web::Path<String>) -> Result<i64, HttpResponse> {
	match path.parse() {
		Ok(id) => Ok(id),
		Err(e) => Err(badRequest!("Invalid topic id: {e}")),
	}
}

async fn get_topic_json(id: i64, dbpool: &PgPool) -> Result<String, HttpResponse> {
	let topic = get_topic(id, dbpool).await?;

	// The arguments against are all those that respond to the for argument
	let arguments_against_future = get_response_arguments(topic.for_argument.id, dbpool);
	// The arguments for are all those that respond to the against argument
	let arguments_for_future = get_response_arguments(topic.against_argument.id, dbpool);

	let (arguments_against, arguments_for) =
		try_join!(arguments_against_future, arguments_for_future)?;

	let body = json!({
		"name": topic.name,
		"for": {
			"title": topic.for_argument.body,
			"opposingID": topic.against_argument.id,
			"arguments": arguments_for
		},
		"against": {
			"title": topic.against_argument.body,
			"opposingID": topic.for_argument.id,
			"arguments": arguments_against
		},
	});
	Ok(body.to_string())
}
