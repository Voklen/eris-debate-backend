use actix_web::HttpResponse;
use log::warn;
use sqlx::PgPool;

use crate::{badRequest, database::arguments::Argument, internalServerError};

pub struct Topic {
	pub name: String,
	pub for_argument: Argument,
	pub against_argument: Argument,
}

pub async fn get_topic(topic_id: i64, db_pool: &PgPool) -> Result<Topic, HttpResponse> {
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
	let for_argument = Argument {
		id: res.for_id,
		body: res.for_body,
	};
	let against_argument = Argument {
		id: res.against_id,
		body: res.against_body,
	};
	Ok(Topic {
		name: res.topic_name,
		for_argument,
		against_argument,
	})
}
