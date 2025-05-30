use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
	dbpool: PgPool,
}

pub mod database;
pub mod endpoints;
pub mod errors;

#[path = "helper/admin.rs"]
pub mod admin_helper;
#[path = "helper/email.rs"]
pub mod email_helper;
#[path = "helper/general.rs"]
pub mod general_helper;
#[path = "helper/hashing.rs"]
pub mod hashing_helper;
#[path = "helper/session.rs"]
pub mod session_helper;
