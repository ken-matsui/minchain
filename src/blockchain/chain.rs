use std::sync::{Arc, Mutex};

use crate::crypt::sha::get_double_sha256;
use crate::Block;

#[derive(Clone, Debug)]
pub struct Blockchain {
    #[allow(dead_code)]
    genesis_block: Block,
    chain: Arc<Mutex<Vec<Block>>>,
}

impl Blockchain {
    pub fn new(genesis_block: Block) -> Blockchain {
        println!("Initializing Blockchain ...");
        Blockchain {
            genesis_block: genesis_block.clone(),
            chain: Arc::new(Mutex::new(vec![genesis_block])),
        }
    }

    pub fn set_new_block(&mut self, block: Block) {
        self.chain.lock().unwrap().push(block);
    }

    #[allow(dead_code)]
    pub fn is_valid(&self, chain: Vec<Block>) -> bool {
        self.chain.lock().unwrap()[1..]
            .iter()
            .cloned()
            .enumerate()
            .any(move |(i, x): (usize, Block)| {
                self.get_hash(&chain[i]) == x.previous_block_hash.unwrap()
            })
    }

    #[allow(dead_code)]
    pub fn is_invalid(&self, chain: Vec<Block>) -> bool {
        !self.is_valid(chain)
    }

    /// 正当性確認に使うためブロックのハッシュ値を取る
    pub fn get_hash(&self, block: &Block) -> String {
        let block_string = serde_json::to_string(block).unwrap();
        get_double_sha256(block_string)
    }

    pub fn get_chain(&self) -> Vec<Block> {
        self.chain.lock().unwrap().clone()
    }
}
