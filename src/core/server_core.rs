use std::net::{SocketAddr, IpAddr, Ipv4Addr, ToSocketAddrs};
use p2p::connection_manager::ConnectionManager;

#[derive(Clone)]
pub enum State {
    Init,
    Standby,
    ConnectedToNetwork,
    ShuttingDown,
}

pub struct ServerCore {
    server_state: State,
    cm: ConnectionManager,
    core_node_addr: Option<SocketAddr>,
}

pub trait Overload<T> {
    fn new(_: T) -> ServerCore;
}

impl Overload<u16> for ServerCore {
    fn new(my_port: u16) -> ServerCore {
        println!("Initializing server ...");
        const MY_IP: Ipv4Addr = get_my_ip();
        println!("Server IP address is set to ... {}", MY_IP);
        let my_addr = SocketAddr::new(IpAddr::V4(MY_IP), my_port);

        ServerCore {
            server_state: State::Init,
            cm: ConnectionManager::new(my_addr),
            core_node_addr: None,
        }
    }
}

impl Overload<(u16, String)> for ServerCore {
    fn new(args: (u16, String)) -> ServerCore {
        let my_port = args.0;
        let node_addr = args.1.to_socket_addrs().unwrap().next().unwrap();

        println!("Initializing server ...");
        const MY_IP: Ipv4Addr = get_my_ip();
        println!("Server IP address is set to ... {}", MY_IP);
        let my_addr = SocketAddr::new(IpAddr::V4(MY_IP), my_port);

        ServerCore{
            server_state: State::Init,
            cm: ConnectionManager::new(my_addr),
            core_node_addr: Some(node_addr),
        }
    }
}

impl ServerCore {
    pub fn start(&mut self) {
        self.server_state = State::Standby;
        self.cm.start();
    }

    pub fn join_network(&mut self) {
        match self.core_node_addr {
            Some(addr) => {
                self.server_state = State::ConnectedToNetwork;
                self.cm.join_network(addr);
            },
            None => println!("This server is runnning as Genesis Core Node ..."),
        };
    }

    pub fn get_my_current_state(&self) -> State {
        self.server_state.clone()
    }
}

impl Drop for ServerCore {
    fn drop(&mut self) -> () { // shutdown_server
        self.server_state = State::ShuttingDown;
        println!("Shutdown server...");
    }
}

const fn get_my_ip() -> Ipv4Addr {
    Ipv4Addr::LOCALHOST
}
