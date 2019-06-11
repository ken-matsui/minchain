use std::net::{SocketAddr, IpAddr, Ipv4Addr, ToSocketAddrs};
use super::server_core::{State, get_my_ip};
use p2p::connection_manager_4edge::ConnectionManager4Edge;

pub struct ClientCore {
    server_state: State,
    cm: ConnectionManager4Edge,
}

impl ClientCore {
    pub fn new(my_port: u16, core_addr: String) -> ClientCore {
        println!("Initializing ClientCore ...");
        const MY_IP: Ipv4Addr = get_my_ip();
        println!("Server IP address is set to ... {}", MY_IP);
        let my_addr = SocketAddr::new(IpAddr::V4(MY_IP), my_port);

        let core_addr = core_addr.to_socket_addrs().unwrap().next().unwrap();

        ClientCore {
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

impl Drop for ClientCore {
    fn drop(&mut self) -> () { // shutdown_server
        self.server_state = State::ShuttingDown;
        println!("Shutdown edge node ...");
    }
}
