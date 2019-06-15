extern crate ctrlc;
extern crate serde;

mod blockchain;
mod core;
mod crypt;
mod p2p;
mod transaction;

use std::env;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use core::server::{Overload, Server};
use core::client::Client;
use transaction::pool::Transaction;
use p2p::message::MsgType;

fn help() {
    eprintln!("cargo run (server|client)");
    eprintln!("cargo run server (genesis)");
    eprintln!("cargo run client (1)");
}

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
        help();
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

            thread::sleep(Duration::from_secs(10));

            let transaction = Transaction::new("test4".to_string(), "test5".to_string(), 3);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction);

            let transaction2 = Transaction::new("test6".to_string(), "test7".to_string(), 2);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction2);

            thread::sleep(Duration::from_secs(10));

            let transaction3 = Transaction::new("test8".to_string(), "test9".to_string(), 10);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction3);

            wait_for_ctlc();
        } else {
            let mut my_p2p_client = Client::new(50088, "localhost:50082");
            my_p2p_client.start();

            thread::sleep(Duration::from_secs(10));

            let transaction = Transaction::new("test1".to_string(), "test2".to_string(), 3);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction);

            let transaction2 = Transaction::new("test1".to_string(), "test3".to_string(), 2);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction2);

            thread::sleep(Duration::from_secs(10));

            let transaction3 = Transaction::new("test5".to_string(), "test6".to_string(), 10);
            my_p2p_client.send_message_to_my_core_node(MsgType::NewTransaction, transaction3);

            wait_for_ctlc();
        };
    } else {
        help();
    };
}
