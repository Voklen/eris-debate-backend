use actix_web::{get, web, HttpResponse, Responder};
use log::warn;
use serde_json::json;
use sqlx::PgPool;
use tokio::try_join;

use crate::{
	arguments_helper::{get_response_arguments, TopArgument},
	badRequest, internalServerError, unwrap_or_esalate, AppState,
};

struct Topic {
	name: String,
	for_argument: TopArgument,
	against_argument: TopArgument,
}

#[get("/topic/{topic_id}")]
async fn get_topic_endpoint(
	path: web::Path<String>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let id = unwrap_or_esalate!(get_id(path));
	let topic_result = get_topic(id, &app_state.dbpool).await;
	let topic = unwrap_or_esalate!(topic_result);
	HttpResponse::Ok().body(topic)
}

fn get_id(path: web::Path<String>) -> Result<i64, HttpResponse> {
	match path.parse() {
		Ok(id) => Ok(id),
		Err(e) => Err(badRequest!("Invalid topic id: {e}")),
	}
}

async fn get_topic(id: i64, dbpool: &PgPool) -> Result<String, HttpResponse> {
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
			for_revision.body AS for_body,
			against_argument.id AS against_id,
			against_revision.body AS against_body
		FROM
			topics
		JOIN
			arguments AS for_argument ON topics.for_argument = for_argument.id
		JOIN
			arguments AS against_argument ON topics.against_argument = against_argument.id
		JOIN
			revisions AS for_revision ON for_argument.revision_latest = for_revision.id
		JOIN
			revisions AS against_revision ON against_argument.revision_latest = against_revision.id
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
	};
	let against_argument = TopArgument {
		id: res.against_id,
		body: res.against_body,
	};
	Ok(Topic {
		name: res.topic_name,
		for_argument,
		against_argument,
	})
}
