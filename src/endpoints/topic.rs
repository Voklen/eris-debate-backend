use actix_web::{get, web, HttpResponse, Responder};
use log::warn;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use tokio::try_join;

use crate::{
	arguments_helper::{get_response_arguments, TopArgument},
	badRequest, internalServerError, unwrap_or_esalate, AppState,
};

#[derive(Deserialize)]
struct ArgumentsRequest {
	id: i64,
}

struct Topic {
	name: String,
	for_argument: TopArgument,
	against_argument: TopArgument,
}

#[get("/topic")]
async fn topic_endpoint(
	title_req: web::Query<ArgumentsRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let id = title_req.id;
	let body_result = get_body(id, &app_state.dbpool).await;
	let body = unwrap_or_esalate!(body_result);
	HttpResponse::Ok().body(body)
}

async fn get_body(id: i64, dbpool: &PgPool) -> Result<String, HttpResponse> {
	let topic = get_topic_arguments(id, dbpool).await?;

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

async fn get_topic_arguments(topic_id: i64, db_pool: &PgPool) -> Result<Topic, HttpResponse> {
	let result = sqlx::query!(
		"
		SELECT
			topics.name AS topic_name,
			for_argument.id AS for_id,
			for_argument.body AS for_body,
			for_user.username AS for_username,
			against_argument.id AS against_id,
			against_argument.body AS against_body,
			against_user.username AS against_username
		FROM
			topics
		JOIN
			arguments AS for_argument ON topics.for_argument = for_argument.id
		JOIN
			arguments AS against_argument ON topics.against_argument = against_argument.id
		JOIN
			users AS for_user ON for_argument.created_by = for_user.id
		JOIN
			users AS against_user ON against_argument.created_by = against_user.id
		WHERE
			topics.id = $1;
		",
		topic_id
	)
	.fetch_one(db_pool)
	.await;
	let res = match result {
		Ok(res) => Ok(res),
		Err(sqlx::Error::RowNotFound) => Err(badRequest!("Topic not found")),
		Err(e) => {
			warn!("Unexpected error when retrieving topic (id={topic_id}): {e}");
			Err(internalServerError!("Error retrieving topic"))
		}
	}?;
	let for_argument = TopArgument {
		id: res.for_id,
		body: res.for_body,
		username: res.for_username,
	};
	let against_argument = TopArgument {
		id: res.against_id,
		body: res.against_body,
		username: res.against_username,
	};
	Ok(Topic {
		name: res.topic_name,
		for_argument,
		against_argument,
	})
}
