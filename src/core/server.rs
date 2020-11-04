use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;
use std::time::Duration;

use blockchain::block::Block;
use blockchain::chain::Blockchain;
use core::state::{get_my_addr, State};
use p2p::connection_manager::{ConnectionManager, Manager};
use transaction::pool::ToVecString;

const CHECK_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct Server {
    server_state: State,
    core_node_addr: Option<SocketAddr>,
    cm: ConnectionManager,
    prev_block_hash: String,
    bc: Blockchain,
}

pub trait Overload<T> {
    fn new(_: T) -> Self;
}

impl Overload<u16> for Server {
    fn new(my_port: u16) -> Self {
        println!("Initializing server ...");
        let my_addr = get_my_addr(my_port);
        println!("Server IP address is set to ... {}", my_addr);

        let my_genesis_block = Block::new_genesis();
        let bc = Blockchain::new(my_genesis_block.clone());

        Server {
            server_state: State::Init,
            core_node_addr: None,
            cm: ConnectionManager::new(my_addr),
            prev_block_hash: bc.get_hash(&my_genesis_block),
            bc,
        }
    }
}

impl Overload<(u16, &'static str)> for Server {
    fn new(args: (u16, &'static str)) -> Self {
        let my_port = args.0;
        let node_addr = args.1.to_socket_addrs().unwrap().next().unwrap();

        println!("Initializing server ...");
        let my_addr = get_my_addr(my_port);
        println!("Server IP address is set to ... {}", my_addr);

        let my_genesis_block = Block::new_genesis();
        let bc = Blockchain::new(my_genesis_block.clone());

        Server {
            server_state: State::Init,
            core_node_addr: Some(node_addr),
            cm: ConnectionManager::new(my_addr),
            prev_block_hash: bc.get_hash(&my_genesis_block),
            bc,
        }
    }
}

impl Server {
    pub fn start(&mut self) {
        self.server_state = State::Standby;
        self.cm.start(self.cm.addr);

        let mut self_clone = self.clone();
        thread::spawn(move || {
            thread::sleep(CHECK_INTERVAL);
            self_clone.generate_block_with_tp();
        });
    }

    #[allow(dead_code)]
    pub fn get_my_current_state(&self) -> State {
        self.server_state.clone()
    }

    pub fn join_network(&mut self) {
        match self.core_node_addr {
            Some(addr) => {
                self.server_state = State::ConnectedToNetwork;
                self.cm.join_network(addr);
            }
            None => println!("This server is runnning as Genesis Core Node ..."),
        };
    }

    fn generate_block_with_tp(&mut self) {
        let mut tp_guard = self.cm.tp.lock().unwrap();
        match tp_guard.get_stored_transactions() {
            Some(result) => {
                let result_len = result.len();

                let new_block =
                    Block::new(result.to_vec_string(), Some(self.prev_block_hash.clone()));
                self.bc.set_new_block(new_block.clone());
                self.prev_block_hash = self.bc.get_hash(&new_block);
                // ブロック生成に成功したらTransaction Poolはクリアする
                tp_guard.clear_my_transactions(result_len);
            }
            None => println!("Transaction Pool is empty ..."),
        };

        println!("Current Blockchain is ... {:#?}", self.bc.get_chain());
        println!("Current prev_block_hash is ... {}", self.prev_block_hash);

        let mut self_clone = self.clone();
        thread::spawn(move || {
            thread::sleep(CHECK_INTERVAL);
            self_clone.generate_block_with_tp();
        });
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // shutdown_server
        self.server_state = State::ShuttingDown;
        println!("Shutdown server ...");
    }
}
