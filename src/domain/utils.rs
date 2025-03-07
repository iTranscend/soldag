//! This module defines a structure and methods for postgresURL construction

use std::{
    env, fmt,
    net::{SocketAddr, ToSocketAddrs},
};

/// Represents a PostgreSQL URL constructed from environment variables.
pub struct PostgresUrl {
    pub user: String,
    pub password: String,
    pub address: SocketAddr,
    pub db_name: String,
}

impl PostgresUrl {
    /// Constructs a `PostgresUrl` from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the required environment variables are not set
    /// or if the database address is invalid.
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            user: env::var("POSTGRES_USER")?,
            password: env::var("POSTGRES_PASSWORD")?,
            address: env::var("DB_ADDR")?
                .to_socket_addrs()?
                .next()
                .ok_or_else(|| anyhow::anyhow!("No DB address supplied"))?,
            db_name: env::var("POSTGRES_DB")?,
        })
    }
}

impl fmt::Display for PostgresUrl {
    /// Formats the `PostgresUrl` as a PostgreSQL connection string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "postgres://{}:{}@{}/{}",
            self.user, self.password, self.address, self.db_name
        )
    }
}
