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
pub mod delete_argument;
#[path = "helper/email.rs"]
pub mod email_helper;
#[path = "helper/general.rs"]
pub mod general_helper;
#[path = "endpoints/arguments/get.rs"]
pub mod get_arguments;
#[path = "endpoints/topic/get.rs"]
pub mod get_topic;
#[path = "helper/hashing.rs"]
pub mod hashing_helper;
#[path = "endpoints/login.rs"]
pub mod login;
#[path = "endpoints/logout.rs"]
pub mod logout;
#[path = "endpoints/arguments/post.rs"]
pub mod post_argument;
#[path = "endpoints/topic/post.rs"]
pub mod post_topic;
#[path = "endpoints/arguments/put.rs"]
pub mod put_argument;
#[path = "helper/session.rs"]
pub mod session_helper;
#[path = "endpoints/signup.rs"]
pub mod signup;
#[path = "endpoints/topics.rs"]
pub mod topics;
#[path = "endpoints/verify_email.rs"]
pub mod verify_email;
