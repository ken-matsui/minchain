extern crate chrono;
extern crate serde;

use self::chrono::Utc;
use self::serde::Serialize;

use crypt::sha::get_double_sha256;

#[derive(Clone, Serialize, Debug)]
pub struct Block {
    timestamp: i64,
    transactions: Vec<String>,
    pub previous_block_hash: Option<String>,
    nonce: Option<u128>,
}

const GENESIS_TXN: &str = "ad9b477b42b22cdf18b1335603d07378ace83561d8398fbfc8de94196c65d806";

impl Block {
    /// Create a genesis block.
    pub fn new_genesis() -> Block {
        let transactions = vec![GENESIS_TXN.to_string()];
        Block::new(transactions, None)
    }

    /// Create a common block.
    pub fn new(transactions: Vec<String>, previous_block_hash: Option<String>) -> Block {
        println!("{}", Utc::now());

        let mut block = Block {
            timestamp: Utc::now().timestamp(),
            transactions,
            previous_block_hash,
            nonce: None,
        };
        println!("block: {:#?}", block);
        let json_block = serde_json::to_string(&block).unwrap();
        block.nonce = Some(compute_nonce_for_pow(json_block, 5));
        block
    }
}

/// Proof of Work
fn compute_nonce_for_pow(msg: String, difficulty: usize) -> u128 {
    // Nonce
    // difficultyの数字を増やせば増やすほど、末尾で揃えなければならない桁数が増える
    let suffix = "0".repeat(difficulty);
    for nonce in 0u128.. {
        // 総当たり的に数字を増やして試す
        let digest = get_double_sha256(format!("{}{}", msg, nonce));
        if digest.ends_with(&suffix) {
            return nonce;
        };
    }
    panic!("Could not find a nonce");
}
