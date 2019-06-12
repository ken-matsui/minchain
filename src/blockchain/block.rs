extern crate chrono;
extern crate serde;

use self::chrono::prelude::*;
use self::serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    timestamp: i64,
    transaction: String,
    pub previous_block_hash: Option<String>,
}

impl Block {
    pub fn new_genesis() -> Block {
        Block {
            timestamp: Utc::now().timestamp(),
            transaction: "AD9B477B42B22CDF18B1335603D07378ACE83561D8398FBFC8DE94196C65D806".to_string(),
            previous_block_hash: None,
        }
    }
    pub fn new(transaction: String, previous_block_hash: String) -> Block {
        Block {
            timestamp: Utc::now().timestamp(),
            transaction,
            previous_block_hash: Some(previous_block_hash),
        }
    }
}
