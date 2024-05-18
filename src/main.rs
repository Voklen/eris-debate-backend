mod database;
mod errors;

use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use database::init_app_state;
use log::info;
use sqlx::PgPool;

#[path = "helper/arguments.rs"]
mod arguments_helper;
#[path = "endpoints/arguments/get.rs"]
mod get_arguments;
#[path = "endpoints/login.rs"]
mod login;
#[path = "endpoints/arguments/post.rs"]
mod post_arguments;
#[path = "endpoints/signup.rs"]
mod signup;
#[path = "endpoints/topic.rs"]
mod topic;

use get_arguments::get_arguments_endpoint;
use login::login_endpoint;
use post_arguments::post_arguments_endpoint;
use signup::signup_endpoint;
use topic::topic_endpoint;

#[derive(Clone)]
struct AppState {
	dbpool: PgPool,
}

const PORT: u16 = 9000;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::init();
	let app_state = init_app_state().await;
	let server = HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(app_state.clone()))
			.wrap(get_cors())
			.service(topic_endpoint)
			.service(signup_endpoint)
			.service(login_endpoint)
			.service(get_arguments_endpoint)
			.service(post_arguments_endpoint)
	})
	.bind(("0.0.0.0", PORT))?
	.run();
	info!("Server initialised on port {PORT}!");
	server.await
}

fn get_cors() -> Cors {
	Cors::default()
		.allowed_origin("http://localhost:3000")
		.allowed_header(http::header::CONTENT_TYPE)
		.supports_credentials()
		.allowed_methods(["GET", "POST"])
}
