use std::sync::{Mutex, Arc};

use blockchain::block::Block;
use crypt::sha::get_double_sha256;

#[derive(Clone, Debug)]
pub struct Blockchain {
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

    pub fn is_valid(&self, chain: Vec<Block>) -> bool {
        self.chain
            .lock()
            .unwrap()
            [1..]
            .to_vec()
            .into_iter()
            .enumerate()
            .find(move |(i, x): &(usize, Block)| {
                self.get_hash(&chain[*i]) == x.clone().previous_block_hash.unwrap()
            })
            .is_some()
    }

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
