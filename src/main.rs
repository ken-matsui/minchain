extern crate ctrlc;
extern crate serde;

mod blockchain;
mod core;
mod p2p;
mod transaction;

use std::env;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use core::cs::{CS, Client, Server, Overload};
use blockchain::block::Block;
use blockchain::blockchain::Blockchain;
use transaction::pool::{Transaction, TransactionPool};

const CHECK_INTERVAL: Duration = Duration::from_secs(10);

fn wait_for_ctlc() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    while running.load(Ordering::SeqCst) {}
    println!("Interrupted by user. Exiting ...");
}

fn generate_block_with_tp(tp: Arc<Mutex<TransactionPool>>, bc: Blockchain, prev_block_hash: String) {
    let mut bc = bc;
    let mut prev_block_hash = prev_block_hash;

    let mut tp_guard = tp.lock().unwrap();
    match tp_guard.get_stored_transactions() {
        Some(result) => {
            let result_len = result.len();

            let new_block = Block::new(result, prev_block_hash.clone());
            bc.set_new_block(new_block.clone());
            prev_block_hash = bc.get_hash(&new_block);
            // ブロック生成に成功したらTransaction Poolはクリアする
            tp_guard.clear_my_transactions(result_len);
        },
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
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        eprintln!("cargo run (server|client)");
        eprintln!("cargo run server (genesis)");
    } else if &args[1] == "server" {
        if args.len() > 2 && &args[2] == "genesis" {
            let mut my_p2p_server = Server::new(50082);
            my_p2p_server.start();
            wait_for_ctlc();
        } else {
            let mut my_p2p_server = Server::new((50090, "localhost:50082"));
            my_p2p_server.start();
            my_p2p_server.join_network();
            wait_for_ctlc();
        };
    } else if &args[1] == "client" {
        if args.len() > 2 && &args[2] == "1" {
            let mut my_p2p_client = Client::new(50095, "localhost:50082");
            my_p2p_client.start();
            wait_for_ctlc();
        } else {
            let mut my_p2p_client = Client::new(50095, "localhost:50082");
            my_p2p_client.start();
            wait_for_ctlc();
        };
    } else if &args[1] == "blockchain" {
        let my_genesis_block = Block::new_genesis();
        let bc = Blockchain::new(my_genesis_block.clone());
        let tp = Arc::new(Mutex::new(TransactionPool::new()));
        let prev_block_hash = bc.get_hash(&my_genesis_block);
        println!("genesis_block_hash : {}" , prev_block_hash);

        let transaction = Transaction::new("test1", "test2", 3);
        tp.lock().unwrap().set_new_transaction(transaction);

        let transaction2 = Transaction::new("test1", "test3", 2);
        tp.lock().unwrap().set_new_transaction(transaction2);

        {
            let tp = tp.clone();
            let bc = bc.clone();
            thread::spawn(move || {
                thread::sleep(CHECK_INTERVAL);
                generate_block_with_tp(tp, bc, prev_block_hash);
            });
        }

        thread::sleep(Duration::from_secs(15));

        let transaction3 = Transaction::new("test5", "test6", 10);
        tp.lock().unwrap().set_new_transaction(transaction3);

        wait_for_ctlc();
    } else {
        eprintln!("cargo run (server|client)");
        eprintln!("cargo run server (genesis)");
        eprintln!("cargo run blockchain");
    };
}
