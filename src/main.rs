use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use eris::database::init_app_state;
use log::info;

use eris::delete_arguments_endpoint::delete_arguments_endpoint;
use eris::get_arguments::get_arguments_endpoint;
use eris::login::login_endpoint;
use eris::logout::logout_endpoint;
use eris::post_arguments::post_arguments_endpoint;
use eris::signup::signup_endpoint;
use eris::topic::topic_endpoint;
use eris::topics::topics_endpoint;

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
			.service(delete_arguments_endpoint)
			.service(logout_endpoint)
			.service(topics_endpoint)
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
		.allowed_methods(["GET", "POST", "DELETE"])
}
