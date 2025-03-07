use anyhow::Ok;
use log::info;
use solana_sdk::{hash::Hash, pubkey::Pubkey};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;

struct Header {
    num_readonly_signed_accounts: u8,
    num_readonly_unsigned_accounts: u8,
    num_required_signatures: u8,
}

struct Instruction {
    accounts: Vec<u8>,
    data: Vec<u8>,
    program_id_index: u8,
    stack_height: u8,
}

struct Message {
    account_keys: Vec<Pubkey>,
    header: Header,
    instructions: Vec<Instruction>,
    recent_block_hash: Hash,
}

pub struct Transaction {
    message: Message,
    signatures: Vec<String>,
}

pub struct Storage {
    pool: PgPool,
}

impl Storage {
    pub async fn new(database_url: &str) -> anyhow::Result<Arc<Self>> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        info!("Running migrations");
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Arc::new(Self { pool }))
    }

    pub async fn insert_transaction(&self, transaction: &Transaction) -> anyhow::Result<()> {
        Ok(())
    }
}
