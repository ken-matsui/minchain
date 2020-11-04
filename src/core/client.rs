use std::net::{SocketAddr, ToSocketAddrs};

use core::state::{get_my_addr, State};
use p2p::connection_manager::{ConnectionManager4Edge, Manager};
use p2p::message::MsgType;
use transaction::pool::Transaction;

pub struct Client {
    server_state: State,
    my_core_addr: SocketAddr,
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
            my_core_addr: core_addr,
            cm: ConnectionManager4Edge::new(my_addr, core_addr),
        }
    }

    pub fn start(&mut self) {
        self.server_state = State::Active;
        self.cm.start(self.cm.addr);
        self.cm.connect_to_core_node();
    }

    #[allow(dead_code)]
    pub fn get_my_current_state(&self) -> State {
        self.server_state.clone()
    }

    pub fn send_message_to_my_core_node(&mut self, msg_type: MsgType, msg: Transaction) {
        let msg_txt = self
            .cm
            .build_message(msg_type, self.cm.addr, None, Some(msg));
        println!("{}", msg_txt);
        self.cm.send_msg(&self.my_core_addr, msg_txt);
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // shutdown_server
        self.server_state = State::ShuttingDown;
        println!("Shutdown edge node ...");
    }
}
