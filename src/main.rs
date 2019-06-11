extern crate ctrlc;

mod core;
mod p2p;

use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use core::server::{Server, Overload};
use core::client::Client;

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
        let mut my_p2p_client = Client::new(50095, "localhost:50082");
        my_p2p_client.start();
        wait_for_ctlc();
    } else {
        eprintln!("cargo run (server|client)");
        eprintln!("cargo run server (genesis)");
    };
}
