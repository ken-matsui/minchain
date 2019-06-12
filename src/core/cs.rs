use std::net::{SocketAddr, ToSocketAddrs};

use crate::core::state::{State, get_my_addr};
use p2p::connection_manager::{ConnectionManager, ConnectionManager4Edge};

pub trait CS {
    /// Start standby.
    fn start(&mut self);
    /// Get server state.
    fn get_my_current_state(&self) -> State;
}

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
}

impl CS for Client {
    fn start(&mut self) {
        self.server_state = State::Active;
        self.cm.start();
        self.cm.connect_to_core_node();
    }

    #[allow(dead_code)]
    fn get_my_current_state(&self) -> State {
        self.server_state.clone()
    }
}

impl Drop for Client {
    fn drop(&mut self) -> () { // shutdown_server
        self.server_state = State::ShuttingDown;
        println!("Shutdown edge node ...");
    }
}

pub struct Server {
    server_state: State,
    cm: ConnectionManager,
    core_node_addr: Option<SocketAddr>,
}

pub trait Overload<T> {
    fn new(_: T) -> Server;
}

impl Overload<u16> for Server {
    fn new(my_port: u16) -> Server {
        println!("Initializing server ...");
        let my_addr = get_my_addr(my_port);
        println!("Server IP address is set to ... {}", my_addr);

        Server {
            server_state: State::Init,
            cm: ConnectionManager::new(my_addr),
            core_node_addr: None,
        }
    }
}

impl Overload<(u16, &'static str)> for Server {
    fn new(args: (u16, &'static str)) -> Server {
        let my_port = args.0;
        let node_addr = args.1.to_socket_addrs().unwrap().next().unwrap();

        println!("Initializing server ...");
        let my_addr = get_my_addr(my_port);
        println!("Server IP address is set to ... {}", my_addr);

        Server {
            server_state: State::Init,
            cm: ConnectionManager::new(my_addr),
            core_node_addr: Some(node_addr),
        }
    }
}

impl CS for Server {
    fn start(&mut self) {
        self.server_state = State::Standby;
        self.cm.start();
    }

    #[allow(dead_code)]
    fn get_my_current_state(&self) -> State {
        self.server_state.clone()
    }
}

impl Server {
    pub fn join_network(&mut self) {
        match self.core_node_addr {
            Some(addr) => {
                self.server_state = State::ConnectedToNetwork;
                self.cm.join_network(addr);
            },
            None => println!("This server is runnning as Genesis Core Node ..."),
        };
    }
}

impl Drop for Server {
    fn drop(&mut self) -> () { // shutdown_server
        self.server_state = State::ShuttingDown;
        println!("Shutdown server ...");
    }
}

