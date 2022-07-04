mod blockchain;
mod core;
mod crypt;
mod p2p;
mod transaction;

use clap::{Parser, Subcommand};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::core::client::Client;
use crate::core::server::{Overload, Server};
use blockchain::block::Block;
use blockchain::chain::Blockchain;
use p2p::message::MsgType;
use transaction::pool::{ToVecString, Transaction, TransactionPool};

const CHECK_INTERVAL: Duration = Duration::from_secs(10);
static mut FLAG_STOP_BLOCK_BUILD: bool = false;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch a server
    Server {
        /// Launch a genesis server
        #[clap(long)]
        genesis: bool,
    },

    /// Launch a client
    Client {
        /// Launch the first client
        #[clap(long)]
        first: bool,
    },

    /// Start a blockchain
    Blockchain,
}

fn wait_for_ctlc() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    while running.load(Ordering::SeqCst) {}
    println!("Interrupted by user. Exiting ...");
}

fn generate_block_with_tp(
    tp: Arc<Mutex<TransactionPool>>,
    mut bc: Blockchain,
    mut prev_block_hash: String,
) {
    let mut tp_guard = tp.lock().unwrap();
    match tp_guard.get_stored_transactions() {
        Some(result) => {
            let result_len = result.len();

            let new_block = Block::new(result.to_vec_string(), Some(prev_block_hash));
            bc.set_new_block(new_block.clone());
            prev_block_hash = bc.get_hash(&new_block);
            // ブロック生成に成功したらTransaction Poolはクリアする
            tp_guard.clear_my_transactions(result_len);
        }
        None => println!("Transaction Pool is empty ..."),
    };

    println!("Current Blockchain is ... {:#?}", bc.get_chain());
    println!("Current prev_block_hash is ... {}", prev_block_hash);

    let tp = tp.clone();
    thread::spawn(move || {
        thread::sleep(CHECK_INTERVAL);
        generate_block_with_tp(tp, bc, prev_block_hash);
    });
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Server { genesis } if *genesis => {
            let mut my_p2p_server = Server::new(50082);
            my_p2p_server.start();
            wait_for_ctlc();
        }
        Commands::Server { .. } => {
            let mut my_p2p_server = Server::new((50090, "localhost:50082"));
            my_p2p_server.start();
            my_p2p_server.join_network();
            wait_for_ctlc();
        }

        Commands::Client { first } if *first => {
            let mut my_p2p_client = Client::new(50095, "localhost:50082");
            my_p2p_client.start();

            thread::sleep(Duration::from_secs(10));

            let transaction = Transaction::new("test4", "test5", 3);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction);

            let transaction2 = Transaction::new("test6", "test7", 2);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction2);

            thread::sleep(Duration::from_secs(10));

            let transaction3 = Transaction::new("test8", "test9", 10);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction3);

            wait_for_ctlc();
        }
        Commands::Client { .. } => {
            let mut my_p2p_client = Client::new(50088, "localhost:50082");
            my_p2p_client.start();

            thread::sleep(Duration::from_secs(10));

            let transaction = Transaction::new("test1", "test2", 3);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction);

            let transaction2 = Transaction::new("test1", "test3", 2);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction2);

            thread::sleep(Duration::from_secs(10));

            let transaction3 = Transaction::new("test5", "test6", 10);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction3);

            wait_for_ctlc();
        }

        Commands::Blockchain => {
            let my_genesis_block = Block::new_genesis();
            let bc = Blockchain::new(my_genesis_block.clone());
            let tp = Arc::new(Mutex::new(TransactionPool::new()));

            let prev_block_hash = bc.get_hash(&my_genesis_block);
            println!("genesis_block_hash : {}", prev_block_hash);

            let transaction = Transaction::new("test1", "test2", 3);
            tp.lock().unwrap().set_new_transaction(transaction);

            let transaction2 = Transaction::new("test1", "test3", 2);
            tp.lock().unwrap().set_new_transaction(transaction2);

            println!("Thread for generate_block_with_tp started!");
            {
                let tp = tp.clone();
                thread::spawn(move || {
                    thread::sleep(CHECK_INTERVAL);
                    generate_block_with_tp(tp, bc, prev_block_hash);
                });
            }
            thread::sleep(Duration::from_secs(20));

            let transaction3 = Transaction::new("test5", "test6", 10);
            tp.lock().unwrap().set_new_transaction(transaction3);

            thread::sleep(Duration::from_secs(30));

            unsafe {
                FLAG_STOP_BLOCK_BUILD = true;
            }
            println!("Stop the Thread for generate_block_with_tp");
        }
    };
}
