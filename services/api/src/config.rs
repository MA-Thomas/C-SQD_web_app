use std::{env, net::SocketAddr};

#[derive(Debug, Clone)]
pub struct Config {
    pub api_addr: SocketAddr,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("CSQD_API_PORT").unwrap_or_else(|_| "8080".to_string());
        let api_addr = format!("127.0.0.1:{port}")
            .parse()
            .expect("CSQD_API_PORT must produce a valid socket address");

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://csqd:csqd@localhost:55432/csqd".to_string());

        Self {
            api_addr,
            database_url,
        }
    }
}
