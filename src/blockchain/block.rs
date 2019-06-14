extern crate chrono;
extern crate serde;

use self::chrono::prelude::*;
use self::serde::{Serialize, Deserialize};

use transaction::pool::Transaction;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    timestamp: i64,
    transactions: Vec<String>,
    pub previous_block_hash: Option<String>,
}

impl Block {
    pub fn new_genesis() -> Block {
        Block {
            timestamp: Utc::now().timestamp(),
            transactions: vec!["ad9b477b42b22cdf18b1335603d07378ace83561d8398fbfc8de94196c65d806".to_string()],
            previous_block_hash: None,
        }
    }
    pub fn new(transactions: Vec<Transaction>, previous_block_hash: String) -> Block {
        Block {
            timestamp: Utc::now().timestamp(),
            transactions: transactions.into_iter().map(|x| serde_json::to_string(&x).unwrap()).collect(),
            previous_block_hash: Some(previous_block_hash),
        }
    }
}
