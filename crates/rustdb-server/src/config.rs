use std::{env, net::SocketAddr};

pub struct Config {
    pub database_url: String,
    pub encryption_key: String,
    pub bind: SocketAddr,
    pub query_timeout: u64,
    pub connect_timeout: u64,
    pub max_rows: usize,
    pub cors_origin: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let number =
            |key: &str, default: u64, max: u64| -> Result<u64, Box<dyn std::error::Error>> {
                let value = env::var(key)
                    .unwrap_or_else(|_| default.to_string())
                    .parse::<u64>()?;
                if value == 0 || value > max {
                    return Err(format!("Invalid {key}").into());
                }
                Ok(value)
            };
        Ok(Self {
            database_url: env::var("RUSTDB_DATABASE_URL")?,
            encryption_key: env::var("RUSTDB_ENCRYPTION_KEY")?,
            bind: SocketAddr::new(
                env::var("RUSTDB_BIND_ADDRESS")
                    .unwrap_or_else(|_| "127.0.0.1".into())
                    .parse()?,
                number("RUSTDB_PORT", 8080, 65535)? as u16,
            ),
            query_timeout: number("RUSTDB_QUERY_TIMEOUT_SECONDS", 30, 300)?,
            connect_timeout: number("RUSTDB_CONNECT_TIMEOUT_SECONDS", 5, 60)?,
            max_rows: number("RUSTDB_MAX_ROWS", 1000, 10000)? as usize,
            cors_origin: env::var("RUSTDB_CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:5173".into()),
        })
    }
}
