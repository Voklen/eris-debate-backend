use actix_web::HttpResponse;
use log::warn;
use serde::Serialize;
use sqlx::PgPool;
use std::rc::Rc;

use crate::{badRequest, internalServerError};

#[derive(Serialize)]
pub struct TopArgument {
	pub id: i64,
	pub body: String,
}

pub async fn get_response_arguments(
	argument_id: i64,
	db_pool: &PgPool,
) -> Result<Rc<[TopArgument]>, HttpResponse> {
	let result = sqlx::query!(
		"
		SELECT arguments.id, revisions.body
		FROM arguments
		JOIN revisions ON arguments.revision_latest = revisions.id
		WHERE parent = $1
		",
		argument_id
	)
	.fetch_all(db_pool)
	.await;
	let res = match result {
		Ok(res) => Ok(res),
		Err(sqlx::Error::RowNotFound) => Err(badRequest!("Argument not found")),
		Err(e) => {
			warn!("Unexpected error when getting argument (id={argument_id}): {e}");
			Err(internalServerError!("Error retrieving argument"))
		}
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
