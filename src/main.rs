extern crate ctrlc;
extern crate serde;

mod blockchain;
mod core;
mod p2p;

use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use core::cs::{CS, Client, Server, Overload};
use blockchain::block::Block;
use blockchain::blockchain::Blockchain;

use self::serde::{Serialize, Deserialize};

fn wait_for_ctlc() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    while running.load(Ordering::SeqCst) {}
    println!("Interrupted by user. Exiting ...");
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
        let mut bc = Blockchain::new(my_genesis_block.clone());
        let prev_block_hash = bc.get_hash(&my_genesis_block);
        println!("genesis_block_hash : {}" , prev_block_hash);

        let transaction = Transaction {
            sender: "test1",
            recipient: "test2",
            value: 3,
        };
        let new_block = Block::new(serde_json::to_string(&transaction).unwrap(), prev_block_hash);
        bc.set_new_block(new_block.clone());
        let new_block_hash = bc.get_hash(&new_block);
        println!("1st_block_hash : {}", new_block_hash);

        let transaction2 = Transaction {
            sender: "test1",
            recipient: "test3",
            value: 2,
        };
        let new_block2 = Block::new(serde_json::to_string(&transaction2).unwrap(), new_block_hash);
        bc.set_new_block(new_block2);

        println!("{:#?}", bc.get_chain());
        let chain = bc.get_chain();
        println!("{}", bc.is_valid(chain));
    } else {
        eprintln!("cargo run (server|client)");
        eprintln!("cargo run server (genesis)");
    };
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    sender: &'static str,
    recipient: &'static str,
    value: i32,
}
