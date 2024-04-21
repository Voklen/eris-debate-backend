use actix_web::{http::header::ContentType, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{internalServerError, AppState};

#[derive(Deserialize)]
struct ArgumentsRequest {
	parent: i64,
	body: String,
}

#[derive(Serialize)]
struct TopArgument {
	id: i64,
	body: String,
}

#[post("/arguments")]
async fn post_arguments_endpoint(
	json: web::Json<ArgumentsRequest>,
	app_state: web::Data<AppState>,
) -> impl Responder {
	let res = create_argument(json, &app_state.dbpool).await;
	match res {
		Ok(()) => {
			return HttpResponse::Ok()
				.content_type(ContentType::json())
				.body("")
		}
		Err(http_response) => http_response,
	}
}

async fn create_argument(
	request: web::Json<ArgumentsRequest>,
	db_pool: &PgPool,
) -> Result<(), HttpResponse> {
	let result = sqlx::query!(
		"INSERT INTO arguments (parent, body) VALUES ($1, $2)",
		request.parent,
		request.body,
	)
	.execute(db_pool)
	.await;
	let res = match result {
		Ok(res) => Ok(res),
		Err(e) => Err(internalServerError!("Error retrieving user data: {e}")),
	}?;
	if res.rows_affected() != 1 {
		println!(
			"Unexpected number of rows affected: {}",
			res.rows_affected()
		);
		return Err(internalServerError!("Unexpected number of rows affected"));
	};
	Ok(())
}
