use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::rc::Rc;
use tokio::try_join;

use crate::{badRequest, internalServerError, AppState};

#[derive(Deserialize)]
struct ArgumentsRequest {
	// title: String,
}

struct Argument {
	id: i64,
	parent: i64,
	body: String,
}

#[derive(Serialize)]
struct TopArgument {
	id: i64,
	body: String,
}

struct Topic {
	name: String,
	for_argument: TopArgument,
	against_argument: TopArgument,
}

#[get("/topic")]
async fn topic_endpoint(
	req: HttpRequest,
	title_req: web::Query<ArgumentsRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let id = 1;
	let body = match get_body(id, &app_state.dbpool).await {
		Ok(res) => res,
		Err(http_response) => return http_response,
	};

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
			against_argument.id AS against_id,
			against_argument.body AS against_body
		FROM
			topics
		JOIN
			arguments AS for_argument ON topics.for_argument = for_argument.id
		JOIN
			arguments AS against_argument ON topics.against_argument = against_argument.id
		WHERE
			topics.id = $1;
		",
		topic_id
	)
	.fetch_one(db_pool)
	.await;
	let res = match result {
		Ok(res) => Ok(res),
		Err(sqlx::Error::RowNotFound) => Err(badRequest!("User not found")),
		Err(e) => Err(internalServerError!("Error retrieving user data: {e}")),
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
