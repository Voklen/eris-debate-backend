use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
	dbpool: PgPool,
}

pub mod database;
pub mod errors;

#[path = "helper/admin.rs"]
pub mod admin_helper;
#[path = "helper/arguments.rs"]
pub mod arguments_helper;
#[path = "endpoints/arguments/delete.rs"]
pub mod delete_arguments_endpoint;
#[path = "helper/email.rs"]
pub mod email_helper;
#[path = "endpoints/arguments/get.rs"]
pub mod get_arguments;
#[path = "helper/hashing.rs"]
pub mod hashing_helper;
#[path = "endpoints/login.rs"]
pub mod login;
#[path = "endpoints/logout.rs"]
pub mod logout;
#[path = "endpoints/arguments/post.rs"]
pub mod post_arguments;
#[path = "helper/session.rs"]
pub mod session_helper;
#[path = "endpoints/signup.rs"]
pub mod signup;
#[path = "endpoints/topic.rs"]
pub mod topic;
#[path = "endpoints/topics.rs"]
pub mod topics;
