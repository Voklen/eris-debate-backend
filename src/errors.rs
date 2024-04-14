/// Print the error and stop the program, if running in debug mode it will panic
/// but in release builds it will print `<program name>: <error>` to stderr and
/// exit with exit code 1.
/// The macro will also format the string within it
/// ```
/// match value {
/// 	Ok(x) => x,
/// 	Err(err) => {
/// 		throw!("Error reading line: {err}")
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! throw {
    ($($message:tt)*) => {{
		use	crate::errors::throw_error_fuction;
        let res = format!($($message)*);
        throw_error_fuction(res)
    }}
}

pub fn throw_error_fuction(error_message: String) -> ! {
	#[cfg(not(debug_assertions))]
	exit_production(error_message);
	#[cfg(debug_assertions)]
	panic!("{error_message}");
}

#[allow(dead_code)]
fn exit_production(error_message: String) -> ! {
	let program_name = env!("CARGO_PKG_NAME");
	eprintln!("{program_name}: {error_message}");
	std::process::exit(1);
}

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
