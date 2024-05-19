use crate::{internalServerError, notFound, unwrap_or_esalate, AppState};
use actix_web::{get, web, HttpResponse, Responder};
use log::error;
use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;

#[derive(Serialize)]
struct Topic {
	name: String,
	id: i64,
}

#[get("/topics")]
async fn topics_endpoint(app_state: web::Data<AppState>) -> impl Responder {
	let topics_result = get_topics(&app_state.dbpool).await;
	let topics = unwrap_or_esalate!(topics_result);
	let body = json!({
		"topics": topics
	});
	HttpResponse::Ok().body(body.to_string())
}

async fn get_topics(db_pool: &PgPool) -> Result<Vec<Topic>, HttpResponse> {
	let result = sqlx::query!("SELECT name, id FROM topics")
		.fetch_all(db_pool)
		.await;
	let res = match result {
		Ok(res) => Ok(res),
		Err(sqlx::Error::RowNotFound) => Err(notFound!("No topics avaliable")),
		Err(e) => {
			error!("Unexpected error when retrieving topics: {e}");
			Err(internalServerError!("Error retrieving topic"))
		}
	}?;

	let topics = res
		.into_iter()
		.map(|topic| Topic {
			name: topic.name,
			id: topic.id,
		})
		.collect();
	Ok(topics)
}
