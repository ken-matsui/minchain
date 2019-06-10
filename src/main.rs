extern crate ctrlc;

mod core;
mod p2p;

use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use core::server_core::{ServerCore, Overload};

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

    if args.len() > 1 && "genesis" == &args[1] {
        let mut my_p2p_server = ServerCore::new(50082);
        my_p2p_server.start();
        wait_for_ctlc();
    } else if args.len() > 1 && "node" == &args[1] {
        let mut my_p2p_server = ServerCore::new((50090, "localhost:50082".to_string()));
        my_p2p_server.start();
        my_p2p_server.join_network();
        wait_for_ctlc();
    } else {
        let mut my_p2p_server = ServerCore::new((50091, "localhost:50082".to_string()));
        my_p2p_server.start();
        my_p2p_server.join_network();
        wait_for_ctlc();
    }
}
