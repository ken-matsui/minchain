use std::net::ToSocketAddrs;

use core::state::{State, get_my_addr};
use p2p::connection_manager::ConnectionManager4Edge;

pub struct Client {
    server_state: State,
    cm: ConnectionManager4Edge,
}

impl Client {
    pub fn new(my_port: u16, core_addr: &'static str) -> Client {
        println!("Initializing ClientCore ...");
        let my_addr = get_my_addr(my_port);
        println!("Server IP address is set to ... {}", my_addr);
        let core_addr = core_addr.to_socket_addrs().unwrap().next().unwrap();

        Client {
            server_state: State::Init,
            cm: ConnectionManager4Edge::new(my_addr, core_addr),
        }
    }

    pub fn start(&mut self) {
        self.server_state = State::Active;
        self.cm.start();
        self.cm.connect_to_core_node();
    }

    #[allow(dead_code)]
    pub fn get_my_current_state(&self) -> State {
        self.server_state.clone()
    }
}

impl Drop for Client {
    fn drop(&mut self) -> () { // shutdown_server
        self.server_state = State::ShuttingDown;
        println!("Shutdown edge node ...");
    }
}
