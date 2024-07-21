use actix_web::HttpResponse;
use log::warn;
use sqlx::PgPool;

use crate::internalServerError;

pub async fn is_admin(id: i64, db_pool: &PgPool) -> Result<bool, HttpResponse> {
	let result = sqlx::query!(
		"SELECT EXISTS(SELECT 1 FROM roles WHERE id=$1 AND role='admin');",
		id,
	)
	.fetch_one(db_pool)
	.await;
	match result {
		Ok(res) => Ok(res.exists.unwrap_or(false)),
		Err(e) => {
			warn!("Error checking if admin(id={id}): {e}");
			Err(internalServerError!("Unknown error 2"))
		}
	}
}
