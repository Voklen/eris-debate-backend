use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use log::{error, warn};

use crate::general_helper::get_env;

/// Sends an email using SMTP info from the environment variables.
/// Returns true on success and false on failure. The reasoning behind this is
/// that to the user we would just display "Email failed to send" (if we show
/// anything). So the specifics don't matter and instead this just logs them
/// and returns a clean boolean.
pub fn send_email(to_address: &str, subject: &str, body: String) -> bool {
	let name = get_env("EMAIL_NAME");
	let from_address = get_env("EMAIL_ADDRESS");
	let password = get_env("EMAIL_PASSWORD");

	let from_email = match from_address.parse() {
		Ok(res) => res,
		Err(e) => {
			error!("Error parsing EMAIL_ADDRESS {from_address}: {e}");
			return false;
		}
	};
	let to_email = match to_address.parse() {
		Ok(res) => res,
		Err(e) => {
			warn!("Error parsing to_address {to_address}: {e}");
			return false;
		}
	};
	let from = Mailbox::new(Some(name), from_email);
	let to = Mailbox::new(None, to_email);
	let email_result = Message::builder()
		.from(from)
		.to(to)
		.subject(subject)
		.header(ContentType::TEXT_PLAIN)
		.body(body);
	let email = match email_result {
		Ok(res) => res,
		Err(e) => {
			warn!("Error building email: {e}");
			return false;
		}
	};
	let creds = Credentials::new(from_address, password);

	// Open a remote connection to mail server
	let url = get_env("EMAIL_URL");
	let mailer = SmtpTransport::relay(&url)
		.unwrap()
		.credentials(creds)
		.build();

	// Send the email
	match mailer.send(&email) {
		Ok(_) => true,
		Err(e) => {
			warn!("Could not send email: {e}");
			false
		}
	}
}
