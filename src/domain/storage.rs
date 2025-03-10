use std::{env, sync::Arc};

use mongodb::{bson::extjson::de::Error, results::InsertOneResult, Client, Collection};

use super::models::transaction::Transaction;

pub struct Storage {
    pub transactions: Collection<Transaction>,
    // pub accounts: Collection<Account>
}

impl Storage {
    pub async fn init() -> anyhow::Result<Arc<Self>> {
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
    ) -> Result<InsertOneResult, Error> {
        let result = self
            .transactions
            .insert_one(transaction)
            .await
            .ok()
            .expect("Error inserting transaction");

        Ok(result)
    }
}
