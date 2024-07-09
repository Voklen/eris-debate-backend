use std::env;

use crate::internalServerError;
use actix_web::HttpResponse;
use argon2::{
	password_hash::{rand_core::OsRng, PasswordHashString, SaltString},
	Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use log::error;

pub fn hash(input: &[u8]) -> Result<PasswordHashString, HttpResponse> {
	let argon2 = Argon2::default();
	let salt = SaltString::generate(&mut OsRng);
	match argon2.hash_password(input, &salt) {
		Ok(hash) => Ok(hash.serialize()),
		Err(e) => {
			error!("Error hashing password: {e}");
			Err(internalServerError!("Password error"))
		}
	}
}

pub fn hash_string(input: &str) -> Result<PasswordHashString, HttpResponse> {
	let bytes = input.as_bytes();
	hash(bytes)
}

pub fn session_token_hash(input: &[u8]) -> Result<PasswordHashString, HttpResponse> {
	let argon2 = Argon2::default();
	let pepper =
		env::var("SESSION_TOKEN_PEPPER").expect("env variable SESSION_TOKEN_PEPPER should be set");
	// Use pepper as salt
	let pepper_as_salt = match SaltString::from_b64(&pepper) {
		Ok(salt) => salt,
		Err(e) => {
			error!("SESSION_TOKEN_PEPPER cannot be parsed: {e}");
			return Err(internalServerError!("Bad server hashing configuration"));
		}
	};
	match argon2.hash_password(input, &pepper_as_salt) {
		Ok(hash) => Ok(hash.serialize()),
		Err(e) => {
			error!("Error hashing password: {e}");
			Err(internalServerError!("Password error"))
		}
	}
}

pub fn check_hashes(unhashed_bytes: &[u8], stored_hash: &str) -> Result<bool, HttpResponse> {
	let hash = match PasswordHash::new(stored_hash) {
		Ok(hash) => hash,
		Err(e) => {
			error!("Invalid hash '{stored_hash}' in database: {e}");
			return Err(internalServerError!(
				"Error checking hash, we're looking into it"
			));
		}
	};
	let is_correct = Argon2::default()
		.verify_password(unhashed_bytes, &hash)
		.is_ok();
	Ok(is_correct)
}
