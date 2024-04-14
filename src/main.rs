mod database;
mod errors;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use database::init_app_state;
use sqlx::PgPool;

#[path = "endpoints/arguments.rs"]
mod arguments;
#[path = "endpoints/users/create.rs"]
mod create_user;
#[path = "endpoints/users/login.rs"]
mod login;

use arguments::arguments_endpoint;
use create_user::create_user_endpoint;
use login::login_endpoint;

#[derive(Clone)]
struct AppState {
	dbpool: PgPool,
}

const PORT: u16 = 9000;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let app_state = init_app_state().await;
	let server = HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(app_state.clone()))
			.wrap(get_cors())
			.service(arguments_endpoint)
			.service(create_user_endpoint)
			.service(login_endpoint)
	})
	.bind(("0.0.0.0", PORT))?
	.run();
	println!("Server initialised on port {PORT}!");
	server.await
}

fn get_cors() -> Cors {
	Cors::default()
		.allowed_origin("http://localhost:3000")
		.allowed_methods(["GET", "POST"])
}
