use std::env;

use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use eris::*;
use general_helper::get_env;
use log::info;

const DEFAULT_PORT: u16 = 9000;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	env_logger::init();
	let port = get_port();
	let app_state = database::init_app_state().await;
	let server = HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(app_state.clone()))
			.wrap(get_cors())
			.service(topic::topic_endpoint)
			.service(signup::signup_endpoint)
			.service(login::login_endpoint)
			.service(get_arguments::get_arguments_endpoint)
			.service(post_argument::post_arguments_endpoint)
			.service(put_argument::put_arguments_endpoint)
			.service(delete_argument::delete_arguments_endpoint)
			.service(logout::logout_endpoint)
			.service(topics::topics_endpoint)
			.service(verify_email::verify_email_endpoint)
	})
	.bind(("0.0.0.0", port))?
	.run();
	info!("Server initialised on port {port}!");
	server.await
}

fn get_port() -> u16 {
	let mut args = env::args();
	args.next(); // Get rid of binary name
	if args.next() == Some("--port".to_owned()) {
		let string_arg = args.next().expect("Must specify port number after --port");
		string_arg.parse().expect("Port number must be a string")
	} else {
		DEFAULT_PORT
	}
}

fn get_cors() -> Cors {
	let origin = get_env("FRONTEND_URL");
	Cors::default()
		.allowed_origin(&origin)
		.allowed_header(http::header::CONTENT_TYPE)
		.supports_credentials()
		.allowed_methods(["GET", "POST", "PUT", "DELETE"])
}
