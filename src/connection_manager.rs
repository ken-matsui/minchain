use std::io::{Read, Write};
use std::{thread, time};
use std::str::from_utf8;
use std::net::{TcpListener, TcpStream, SocketAddr};

use ::message_manager::{MessageManager, MsgType};
use ::core_node_list::CoreNodeList;
use std::collections::HashSet;

const PING_INTERVAL: time::Duration = time::Duration::from_secs(30);

#[derive(Clone)]
pub struct ConnectionManager {
    addr: SocketAddr,
    my_c_addr: Option<SocketAddr>,
    core_node_set: CoreNodeList,
    mm: MessageManager,
}

impl ConnectionManager {
    pub fn new(addr: SocketAddr) -> ConnectionManager {
        println!("Initializing ConnectionManager ...");
        let mut core_node_list = CoreNodeList::new();
        core_node_list.add(addr);
        ConnectionManager {
            addr,
            my_c_addr: None,
            core_node_set: core_node_list,
            mm: MessageManager::new(),
        }
    }

    /// Start standby. (for ServerCore)
    pub fn start(&mut self) {
        { // Reference: https://stackoverflow.com/a/33455247
            let self_clone = self.clone();
            thread::spawn(move || {
                self_clone.wait_for_access();
            });
        }
        {
            let mut self_clone = self.clone();
            thread::spawn(move || {
                thread::sleep(PING_INTERVAL);
                self_clone.check_peers_connection();
            });
        }
    }

    /// Connect to a known Core node specified by the user. (for ServerCore)
    pub fn join_network(&mut self, node_addr: SocketAddr) {
        self.my_c_addr = Some(node_addr);
        self.connect_to_p2pnw(node_addr);
    }

    fn connect_to_p2pnw(&self, node_addr: SocketAddr) {
        let mut stream = TcpStream::connect(node_addr).unwrap();
        let msg = self.mm.build(MsgType::Add, self.addr.port(), None);
        thread::spawn(move || {
            stream.write(msg.as_bytes()).unwrap();
        });
    }

    pub fn send_msg(&mut self, peer: &SocketAddr, msg: String) {
        let mut stream = TcpStream::connect(peer).unwrap();
        thread::spawn(move || {
            stream.write(msg.as_bytes()).unwrap();
        });
    }

    pub fn send_msg_to_all_peer(&mut self, msg: String) {
         println!("send_msg_to_all_peer was called!");
         for peer in self.core_node_set.list.clone() {
             if peer != self.addr {
                 println!("message will be sent to ... {}", peer);
             };
             self.send_msg(&peer, msg.clone());
         }
    }

    /// Add a core node to the list.
    fn add_peer(&mut self, peer: &SocketAddr) {
        self.core_node_set.add(*peer);
    }

    /// Remove a core node that has left from the list.
    fn remove_peer(&mut self, peer: &SocketAddr) {
        self.core_node_set.remove(peer);
    }

    fn wait_for_access(&self) {
        let listener = TcpListener::bind(self.addr).unwrap();
        loop {
            println!("Waiting for the connection ...");
            match listener.accept() {
                Ok((mut stream, addr)) => {
                    println!("Connected by .. {}", addr);
                    let mut self_clone = self.clone();
                    thread::spawn(move || {
                        let mut b = [0; 1024];
                        let n = stream.read(&mut b).unwrap();
                        self_clone.handle_message(&addr, &u8_to_str(&b[0..n]));
                    });
                },
                Err(e) => {
                    println!("An error occurred while accepting a connection: {}", e);
                    continue;
                }
            };
        }
    }

    fn build_message(&self) -> String {
        let mut vec = Vec::new();
        vec.extend(self.core_node_set.list.clone().into_iter());
        self.mm.build(MsgType::CoreList, self.addr.port(), Some(vec))
    }

    fn handle_message(&mut self, addr: &SocketAddr, data: &String) {
        match self.mm.parse(data) {
            Ok((msg_type, payload)) => {
                match payload {
                    None => {
                        match msg_type {
                            MsgType::Add => {
                                println!("ADD node request was received!!");
                                self.add_peer(addr);
                                if self.addr != *addr {
                                    let msg = self.build_message();
                                    self.send_msg_to_all_peer(msg);
                                };
                            },
                            MsgType::Remove => {
                                println!("REMOVE request was received!! from {}", addr);
                                self.remove_peer(addr);
                                let msg = self.build_message();
                                self.send_msg_to_all_peer(msg);
                            },
                            MsgType::Ping => {},
                            MsgType::RequestCoreList => {
                                println!("List for Core nodes was requested!!");
                                let msg = self.build_message();
                                self.send_msg(addr, msg);
                            },
                            unknown => {
                                println!("received unknown command: {:?}", unknown);
                            },
                        };
                    },
                    Some(mut pl) => {
                        match msg_type {
                            MsgType::CoreList => {
                                // TODO: 受信したリストをただ上書きしてしまうのは
                                // 本来セキュリティ的にはよろしくない。
                                // 信頼できるノードの鍵とかをセットしとく必要があるかも
                                println!("Refresh the core node list ...");
                                let mut new_core_set = CoreNodeList::new();
                                new_core_set.list = pl.drain(..).collect();
                                println!("latest core node list: {}", new_core_set);
                                self.core_node_set = new_core_set;
                            },
                            unknown => {
                                eprintln!("received unknown command: {:?}", unknown);
                            },
                        };
                    },
                };
            },
            Err(e) => eprintln!("{}", e),
        };
    }

    /// 接続されている全てのCoreノードを生存確認する 30秒毎
    fn check_peers_connection(&mut self) {
        let dead_c_node_set: HashSet<SocketAddr> =
            self.core_node_set.list
                .iter()
                .cloned()
                .filter(|x| !self.is_alive(x))
                .collect();

        if dead_c_node_set.len() > 0 {
            println!("Removing: {:?}", dead_c_node_set);
            self.core_node_set.list = &self.core_node_set.list - &dead_c_node_set;
            println!("current core node list: {}", self.core_node_set);

            // ブロードキャストで通知する
            let msg = self.build_message();
            self.send_msg_to_all_peer(msg);
        } else {
            println!("current core node list: {}", self.core_node_set);
        };

        let mut self_clone = self.clone();
        thread::spawn(move || {
            thread::sleep(PING_INTERVAL);
            self_clone.check_peers_connection();
        });
    }

    /// Send a message to confirm valid nodes.
    fn is_alive(&self, target: &SocketAddr) -> bool {
        let mut stream = TcpStream::connect(target).unwrap();
        let msg = self.mm.build(MsgType::Ping, 50082, None);
        let result = thread::spawn(move || {
            stream.write(msg.as_bytes())
        });
        match result.join() {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl Drop for ConnectionManager {
    /// Close socket.
    fn drop(&mut self) -> () { // connection_close
        // Send a leave request.
        println!("Closing connection ...");
        match self.my_c_addr {
            None => {},
            Some(my_c_addr) => {
                let msg = self.mm.build(MsgType::Remove, self.addr.port(), None);
                self.send_msg(&my_c_addr, msg);
            },
        };
    }
}

fn u8_to_str(content: &[u8]) -> String {
    from_utf8(&content.to_vec()).unwrap().to_string()
}
