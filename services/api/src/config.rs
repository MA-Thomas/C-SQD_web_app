use std::{env, net::SocketAddr};

#[derive(Debug, Clone)]
pub struct Config {
    pub api_addr: SocketAddr,
    pub database_url: String,
    /// Base URL of the public web app, used to build magic sign-in links.
    pub web_base_url: String,
    /// Development-only auth mode: when true, magic sign-in links are
    /// returned in the API response instead of being delivered by email.
    /// MUST be false in any deployment reachable by other people —
    /// otherwise anyone can sign in as any email address.
    pub dev_auth: bool,
    /// Set the `Secure` attribute on session cookies. Enable in any
    /// HTTPS deployment; leave off only for local plain-HTTP development.
    pub secure_cookies: bool,
    /// Transactional email provider: "resend" or "postmark". None → log
    /// outbound mail instead of sending (local development).
    pub email_provider: Option<String>,
    pub email_api_key: Option<String>,
    /// From header for outbound mail.
    pub email_from: String,
    /// Operator inbox for inquiry notifications (CSQD_CONTACT_EMAIL).
    pub contact_email: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("CSQD_API_PORT").unwrap_or_else(|_| "8080".to_string());
        // Loopback by default; containers and hosted deployments set
        // CSQD_API_BIND=0.0.0.0 (behind TLS termination).
        let bind = env::var("CSQD_API_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
        let api_addr = format!("{bind}:{port}")
            .parse()
            .expect("CSQD_API_BIND/CSQD_API_PORT must produce a valid socket address");

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://csqd:csqd@localhost:55432/csqd".to_string());

        let web_base_url =
            env::var("CSQD_WEB_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        // Default to dev auth ON so local development keeps working out of
        // the box; deployments must set CSQD_DEV_AUTH=0 and wire real email
        // delivery.
        let dev_auth = env_flag("CSQD_DEV_AUTH", true);
        let secure_cookies = env_flag("CSQD_SECURE_COOKIES", false);

        let email_provider = env::var("CSQD_EMAIL_PROVIDER")
            .ok()
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());
        let email_api_key = env::var("CSQD_EMAIL_API_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let email_from = env::var("CSQD_EMAIL_FROM")
            .unwrap_or_else(|_| "C-SQD <no-reply@csqd.local>".to_string());
        let contact_email =
            env::var("CSQD_CONTACT_EMAIL").unwrap_or_else(|_| "admin@csqd.local".to_string());

        Self {
            api_addr,
            database_url,
            web_base_url,
            dev_auth,
            secure_cookies,
            email_provider,
            email_api_key,
            email_from,
            contact_email,
        }
    }
}

fn env_flag(name: &str, default: bool) -> bool {
    match env::var(name) {
        Ok(value) => matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "on"),
        Err(_) => default,
    }
}
