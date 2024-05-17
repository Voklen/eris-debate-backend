#[macro_export]
macro_rules! unwrap_or_esalate {
	($arg:expr) => {{
		match $arg {
			Ok(res) => res,
			Err(err) => return err,
		}
	}};
}

#[macro_export]
macro_rules! internalServerError {
	($($arg:tt)*) => {{
		use actix_web::HttpResponse;
        let body = format!($($arg)*);
		HttpResponse::InternalServerError().body(body)
	}};
}

#[macro_export]
macro_rules! badRequest {
	($($arg:tt)*) => {{
		use actix_web::HttpResponse;
        let body = format!($($arg)*);
		HttpResponse::BadRequest().body(body)
	}};
}

#[macro_export]
macro_rules! unauthorized {
	($($arg:tt)*) => {{
		use actix_web::HttpResponse;
        let body = format!($($arg)*);
		HttpResponse::Unauthorized().body(body)
	}};
}
