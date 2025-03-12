use std::{env, sync::Arc};

use chrono::{DateTime, Days, Utc};
use mongodb::{
    bson::{self, doc, extjson::de::Error, oid::ObjectId, Bson, Document},
    options::FindOptions,
    results::InsertOneResult,
    Client, Collection,
};

use super::models::transaction::Transaction;

pub struct Storage {
    pub transactions: Collection<Transaction>,
    // pub accounts: Collection<Account>
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
        // let accounts: Collection<Account> = db.collection("account");

        Ok(Arc::new(Storage {
            transactions,
            // accounts,
        }))
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

        dbg!(&query);

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

        // dbg!(&cursor.current());
        let mut transactions: Vec<Transaction> = Vec::new();

        while cursor.advance().await? {
            // dbg!(&cursor.current());
            transactions.push(cursor.deserialize_current()?);
        }

        // let transactions = cursor
        //     .try_collect::<Vec<Transaction>>()
        //     .await?;

        Ok((transactions, next))
    }

    // pub async fn get_transaction_by_id(&self, id: String) -> Result<Transaction, anyhow::Error> {
    // // pub async fn get_transaction_by_id(&self) -> Result<Transaction, anyhow::Error> {
    //     let mut document = Document::new();
    //     // document.insert("id", id);
    //     document.insert("_id", id);
    //     let transaction = match self
    //         .transactions
    //         .find_one(document)
    //         .await?
    //         {
    //             Some(transaction) => transaction,
    //             None => return Err(anyhow::anyhow!("Transaction not found")),
    //         };
    //         Ok(transaction)
    //     }
    // .find_one(doc! { "_id": ObjectId::parse_str("67d05a4682aee97fad3348dc".to_string())? })
}
