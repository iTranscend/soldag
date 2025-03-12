//! Storage module for handling MongoDB database operations.
//!
//! This module provides the core database functionality for the SolDag application,
//! managing transaction storage and retrieval operations. It uses MongoDB as the backend
//! and provides an abstraction layer for database operations.

use std::{env, sync::Arc};

use chrono::{DateTime, Days, Utc};
use mongodb::{
    bson::{doc, Document},
    options::FindOptions,
    results::InsertOneResult,
    Client, Collection,
};

use super::models::transaction::Transaction;

/// Storage struct representing the MongoDB database connection and collections.
///
/// This struct holds the MongoDB collections and provides methods for database operations.
/// It is designed to be thread-safe and can be shared across different parts of the application.
pub struct Storage {
    /// Collection for storing Solana transactions
    pub transactions: Collection<Transaction>,
}

impl Storage {
    /// Initializes a new Storage instance with MongoDB connection.
    ///
    /// This function creates a new connection to MongoDB using either the MONGO_URI
    /// environment variable or a default localhost connection string. It sets up the
    /// database and collections needed for the application.
    ///
    /// # Returns
    ///
    /// * `eyre::Result<Arc<Self>>` - A thread-safe reference to the Storage instance
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * MongoDB connection fails
    /// * Database initialization fails
    pub async fn init() -> eyre::Result<Arc<Self>> {
        let uri = match env::var("MONGO_URI") {
            Ok(v) => v.to_string(),
            Err(_) => "mongodb://localhost:27017/?directConnection=true".to_string(),
        };

        let client = Client::with_uri_str(uri).await?;
        let db = client.database("soldag");

        let transactions: Collection<Transaction> = db.collection("transactions");

        Ok(Arc::new(Storage { transactions }))
    }

    /// Inserts a single transaction into the database.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to insert
    ///
    /// # Returns
    ///
    /// * `eyre::Result<InsertOneResult>` - The result of the insertion operation
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails
    pub async fn insert_transaction(
        &self,
        transaction: Transaction,
    ) -> eyre::Result<InsertOneResult> {
        let result = self
            .transactions
            .insert_one(transaction)
            .await
            .ok()
            .expect("Error inserting transaction");

        Ok(result)
    }

    /// Retrieves transactions from the database with pagination support.
    ///
    /// # Arguments
    ///
    /// * `id` - Optional transaction signature to filter by
    /// * `day` - Optional date to filter transactions by day
    /// * `count` - Number of transactions to return
    /// * `offset` - Number of transactions to skip
    ///
    /// # Returns
    ///
    /// * `eyre::Result<(Vec<Transaction>, Option<u64>)>` - A tuple containing:
    ///   - Vector of transactions
    ///   - Optional next offset for pagination
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Database query fails
    /// * Deserialization of results fails
    pub async fn get_transactions(
        &self,
        id: Option<String>,
        day: Option<DateTime<Utc>>,
        count: u64,
        offset: u64,
    ) -> eyre::Result<(Vec<Transaction>, Option<u64>)> {
        let mut query = Document::new();
        if let Some(id) = id {
            query.insert("signature", id);
        }
        if let Some(day) = day {
            let start_of_day = day;
            let end_of_day = day
                .checked_add_days(Days::new(1))
                .unwrap_or(DateTime::<Utc>::MAX_UTC);
            query.insert(
                "block_time",
                doc! {
                    "$gte": start_of_day,
                    "$lte": end_of_day,
                },
            );
        }

        let (total, mut cursor) = tokio::try_join!(
            self.transactions.count_documents(query.clone()),
            self.transactions.find(query).with_options(
                FindOptions::builder()
                    .limit(count as i64)
                    .skip(offset)
                    .build(),
            )
        )?;

        let next = count.saturating_add(offset);
        let next = (next < total).then_some(next);

        let mut transactions: Vec<Transaction> = Vec::new();

        while cursor.advance().await? {
            transactions.push(cursor.deserialize_current()?);
        }

        Ok((transactions, next))
    }
}
