use std::env;

pub fn get_env(var_name: &str) -> String {
	let msg = format!("env variable {var_name} should be set");
	env::var(var_name).expect(&msg)
}
