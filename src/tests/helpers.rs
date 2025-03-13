use solana_sdk::message::MessageHeader;
use solana_transaction_status_client_types::{
    option_serializer::OptionSerializer, UiRawMessage, UiTransactionStatusMeta,
};
use std::sync::Arc;
use tokio::sync::{Notify, OnceCell};

use crate::domain::{models::transaction::Transaction, storage::Storage};

static TEST_STATE: OnceCell<TestState> = OnceCell::const_new();

pub struct TestState {
    pub storage: Arc<Storage>,
    pub notifier: Notify,
}

pub async fn get_global_state() -> &'static TestState {
    TEST_STATE
        .get_or_init(|| async {
            TestState {
                storage: Storage::init("soldag_test")
                    .await
                    .expect("Failed to initialize test storage"),
                notifier: Notify::new(),
            }
        })
        .await
}

pub fn create_mock_message() -> UiRawMessage {
    UiRawMessage {
        header: MessageHeader::default(),
        account_keys: vec![],
        recent_blockhash: "11111111111111111111111111111111".to_string(),
        instructions: vec![],
        address_table_lookups: None,
    }
}

pub fn create_mock_meta() -> UiTransactionStatusMeta {
    UiTransactionStatusMeta {
        err: None,
        status: Ok(()),
        fee: 0,
        pre_balances: vec![],
        post_balances: vec![],
        inner_instructions: OptionSerializer::Skip,
        log_messages: OptionSerializer::Skip,
        pre_token_balances: OptionSerializer::Skip,
        post_token_balances: OptionSerializer::Skip,
        rewards: OptionSerializer::Skip,
        loaded_addresses: OptionSerializer::Skip,
        return_data: OptionSerializer::Skip,
        compute_units_consumed: OptionSerializer::Skip,
    }
}

pub fn create_mock_transaction(index: u64) -> Transaction {
    Transaction {
        signature: format!("signature_{}", index),
        message: create_mock_message(),
        meta: create_mock_meta(),
        block_time: None,
    }
}
