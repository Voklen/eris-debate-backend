use std::rc::Rc;

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use base64::{engine, Engine};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use tokio::join;

use crate::{badRequest, unwrap_or_esalate};
use crate::{internalServerError, unauthorized, AppState};

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

struct TopicArguments {
	for_argument: TopArgument,
	against_argument: TopArgument,
}

#[get("/topic")]
async fn topic_endpoint(
	req: HttpRequest,
	title_req: web::Query<ArgumentsRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	// let auth = match Authorization::<Basic>::parse(&req) {
	// 	Ok(res) => res,
	// 	Err(e) => return unauthorized!("Authorization header error: {e}"),
	// };
	// let email = auth.as_ref().user_id();
	// let token = auth.as_ref().password();
	// let authorization_result = check_authorization(email, token, &app_state.dbpool).await;
	// let is_authorized = unwrap_or_esalate!(authorization_result);
	// if !is_authorized {
	// 	return unauthorized!("Incorrect username or key");
	// }
	// let title = &title_req.title;
	let id = 1;
	let topic_arguments_result = get_topic_arguments(id, &app_state.dbpool).await;
	let topic_arguments = unwrap_or_esalate!(topic_arguments_result);
	let for_arguments_future =
		get_responce_arguments(topic_arguments.for_argument.id, &app_state.dbpool);
	let against_arguments_future =
		get_responce_arguments(topic_arguments.against_argument.id, &app_state.dbpool);
	let (for_arguments_result, against_arguments_result) =
		join!(for_arguments_future, against_arguments_future);
	let for_arguments = unwrap_or_esalate!(for_arguments_result);
	let against_arguments = unwrap_or_esalate!(against_arguments_result);

	let body = json!({
		"for": for_arguments,
		"against": against_arguments,
	});
	HttpResponse::Ok().body(body.to_string())
}

/// Returns `Ok(true)` if authorized, `Ok(false)` if unauthorized and `Err(_)` on error
async fn check_authorization(
	username: &str,
	session_token: Option<&str>,
	db_pool: &PgPool,
) -> Result<bool, HttpResponse> {
	let token = session_token.ok_or(unauthorized!("No session token provided"))?;
	let base64_decoder = engine::general_purpose::URL_SAFE;
	let token_bytes = base64_decoder
		.decode(token)
		.or_else(|e| Err(unauthorized!("Session token decode error: {e}")))?;
	let result = sqlx::query!(
		r#"SELECT COUNT(1) AS "count!" FROM session_tokens WHERE username=$1 AND token=$2;"#,
		username,
		token_bytes
	)
	.fetch_one(db_pool)
	.await;
	match result {
		Ok(res) => Ok(res.count >= 1),
		Err(e) => Err(internalServerError!("Error retrieving user data: {e}")),
	}
}

async fn get_topic_arguments(
	topic_id: i64,
	db_pool: &PgPool,
) -> Result<TopicArguments, HttpResponse> {
	let result = sqlx::query!(
		"
		SELECT
			for_argument.id AS for_argument_id,
			for_argument.body AS for_argument_body,
			against_argument.id AS against_argument_id,
			against_argument.body AS against_argument_body
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
		id: res.for_argument_id,
		body: res.for_argument_body,
	};
	let against_argument = TopArgument {
		id: res.against_argument_id,
		body: res.against_argument_body,
	};
	Ok(TopicArguments {
		for_argument,
		against_argument,
	})
}

async fn get_responce_arguments(
	argument_id: i64,
	db_pool: &PgPool,
) -> Result<Vec<TopArgument>, HttpResponse> {
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
