use std::{env, sync::Arc};

use chrono::{DateTime, Days, Utc};
use mongodb::{
    bson::{doc, Document},
    options::FindOptions,
    results::InsertOneResult,
    Client, Collection,
};

use super::models::transaction::Transaction;

pub struct Storage {
    pub transactions: Collection<Transaction>,
}

impl Storage {
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
