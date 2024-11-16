use dotenvy::dotenv;
use lazy_static::lazy_static;
use secrecy::Secret;
use std::env as std_env;

// Define a lazily evaluated static. lazy_static is needed because std_env::var is not a const function.
lazy_static! {
    pub static ref JWT_SECRET: Secret<String> = set_token();
    pub static ref DATABASE_URL: Secret<String> = set_database_url();
    pub static ref REDIS_HOST_NAME: String = set_redis_host();
}

fn set_token() -> Secret<String> {
    dotenv().ok(); // Load environment variables
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }
    Secret::new(secret)
}

fn set_database_url() -> Secret<String> {
    dotenv().ok();
    let database_url = std_env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    if database_url.is_empty() {
        panic!("DATABASE_URL must not be empty.");
    }
    Secret::new(database_url)
}

fn set_redis_host() -> String {
    dotenv().ok();
    std_env::var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned())
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const POSTGRES_PASSWORD_ENV_VAR: &str = "POSTGRES_PASSWORD";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}
