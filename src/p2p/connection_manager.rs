use std::io::{Read, Write};
use std::{thread, time};
use std::sync::{Arc, Mutex};
use std::str::from_utf8;
use std::net::{TcpListener, TcpStream, SocketAddr};

use p2p::message_manager::{MessageManager, MsgType};
use p2p::core_node_list::CoreNodeList;
use p2p::edge_node_list::EdgeNodeList;

const PING_INTERVAL: time::Duration = time::Duration::from_secs(10);

#[derive(Clone)]
pub struct ConnectionManager {
    addr: SocketAddr,
    my_c_addr: Option<SocketAddr>,
    core_node_set: Arc<Mutex<CoreNodeList>>,
    edge_node_set: Arc<Mutex<EdgeNodeList>>,
    mm: MessageManager,
}

impl ConnectionManager {
    pub fn new(self_addr: SocketAddr) -> ConnectionManager {
        println!("Initializing ConnectionManager ...");
        let mut core_node_list = CoreNodeList::new();
        core_node_list.add(self_addr);
        ConnectionManager {
            addr: self_addr,
            my_c_addr: None,
            core_node_set: Arc::new(Mutex::new(core_node_list)),
            edge_node_set: Arc::new(Mutex::new(EdgeNodeList::new())),
            mm: MessageManager::new(),
        }
    }

    /// Start standby. (for ServerCore)
    pub fn start(&mut self) {
        let self_clone = self.clone();
        { // Reference: https://stackoverflow.com/a/33455247
            let self_clone = self_clone.clone();
            thread::spawn(move || {
                self_clone.wait_for_access();
            });
        }
        {
            let mut self_clone = self_clone.clone();
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
        let msg = self.mm.build(MsgType::Add, self.addr, None);
        thread::spawn(move || {
            stream.write(msg.as_bytes()).unwrap();
        });
    }

    pub fn send_msg(&mut self, peer: &SocketAddr, msg: String) {
        match TcpStream::connect(peer) {
            Ok(mut stream) => {
                thread::spawn(move || {
                    stream.write(msg.as_bytes()).unwrap();
                });
            },
            Err(_) => {
                eprintln!("Connection failed for peer : {}", *peer);
                self.remove_peer(peer);
            },
        }
    }

    pub fn send_msg_to_all_peer(&mut self, msg: String) {
        println!("send_msg_to_all_peer was called!");
        let list = self.core_node_set.lock().unwrap().get_list();
        for peer in list {
            if peer != self.addr {
                println!("message will be sent to ... ({})", peer);
            };
            self.send_msg(&peer, msg.clone());
        }
    }

    /// Add a core node to the list.
    fn add_peer(&mut self, peer: &SocketAddr) {
        self.core_node_set.lock().unwrap().add(*peer);
    }

    /// Remove a core node that has left from the list.
    fn remove_peer(&mut self, peer: &SocketAddr) {
        self.core_node_set.lock().unwrap().remove(peer);
    }

    /// Add a edge node to the list.
    fn add_edge_node(&mut self, edge: &SocketAddr) {
        self.edge_node_set.lock().unwrap().add(*edge);
    }

    /// Remove a edge node that has left from the list.
    fn remove_edge_node(&mut self, edge: &SocketAddr) {
        self.edge_node_set.lock().unwrap().remove(edge);
    }

    /// Always listen during server startup.
    fn wait_for_access(&self) {
        let listener = TcpListener::bind(self.addr).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut self_clone = self.clone();
                    thread::spawn(move || {
                        let mut b = [0; 1024];
                        let n = stream.read(&mut b).unwrap();
                        self_clone.handle_message(&u8_to_str(&b[0..n]));
                    });
                },
                Err(e) => {
                    eprintln!("An error occurred while accepting a connection: {}", e);
                    continue;
                },
            };
        }
    }

    fn build_message(&self) -> String {
        let mut vec = Vec::new();
        vec.extend(self.core_node_set.lock().unwrap().get_list().into_iter());
        self.mm.build(MsgType::CoreList, self.addr, Some(vec))
    }

    fn handle_message(&mut self, data: &String) {
        match self.mm.parse(data) {
            Ok(msg) => {
                println!("Connected by .. ({})", msg.my_addr);
                match msg.payload {
                    None => {
                        match msg.msg_type {
                            MsgType::Add => {
                                println!("ADD node request was received!!");
                                self.add_peer(&msg.my_addr);
                                if self.addr != msg.my_addr {
                                    let m = self.build_message();
                                    self.send_msg_to_all_peer(m);
                                };
                            },
                            MsgType::Remove => {
                                println!("REMOVE request was received!! from: ({})", msg.my_addr);
                                self.remove_peer(&msg.my_addr);
                                let m = self.build_message();
                                self.send_msg_to_all_peer(m);
                            },
                            MsgType::Ping => {},
                            MsgType::RequestCoreList => {
                                println!("List for Core nodes was requested!!");
                                let m = self.build_message();
                                self.send_msg(&msg.my_addr, m);
                            },
                            MsgType::AddAsEdge => {
                                self.add_edge_node(&msg.my_addr);
                                let m = self.build_message();
                                self.send_msg(&msg.my_addr, m);
                            },
                            MsgType::RemoveEdge => {
                                println!("REMOVE_EDGE request was received!! from: ({})", msg.my_addr);
                                self.remove_edge_node(&msg.my_addr);
                            },
                            unknown => {
                                println!("received unknown command: {:?}", unknown);
                            },
                        };
                    },
                    Some(mut pl) => {
                        match msg.msg_type {
                            MsgType::CoreList => {
                                // TODO: 受信したリストをただ上書きしてしまうのは
                                // 本来セキュリティ的にはよろしくない。
                                // 信頼できるノードの鍵とかをセットしとく必要があるかも
                                println!("Refresh the core node list ...");
                                let mut new_core_set = CoreNodeList::new();
                                new_core_set.list = pl.drain(..).collect();
                                println!("latest core node list: {}", new_core_set);
                                self.core_node_set.lock().unwrap().overwrite(new_core_set.list);
                            },
                            unknown => {
                                eprintln!("received unknown command: {:?}", unknown);
                            },
                        };
                    },
                };
            },
            Err(e) => eprintln!("Error: {}", e),
        };
    }

    /// Check all connected core nodes every PING_INTERVAL for survival.
    fn check_peers_connection(&mut self) {
        let mut changed = false;

        let list = self.core_node_set.lock().unwrap().get_list();
        for peer in &list {
            if !self.is_alive(peer) {
                self.remove_peer(peer); // Remove dead node
                changed = true;
            }
        }
        println!("current core node list: {}", self.core_node_set.lock().unwrap());

        if changed {
            // Notify with broadcast
            let msg = self.build_message();
            self.send_msg_to_all_peer(msg);
        }

        let mut self_clone = self.clone();
        thread::spawn(move || {
            thread::sleep(PING_INTERVAL);
            self_clone.check_peers_connection();
        });
    }

    /// Send a message to confirm valid nodes.
    fn is_alive(&self, target: &SocketAddr) -> bool {
        match TcpStream::connect(target) {
            Ok(mut stream) => {
                let msg = self.mm.build(MsgType::Ping, self.addr, None);
                let result = thread::spawn(move || {
                    stream.write(msg.as_bytes())
                });
                match result.join() {
                    Ok(_) => true,
                    Err(_) => false,
                }
            },
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
                let msg = self.mm.build(MsgType::Remove, self.addr, None);
                self.send_msg(&my_c_addr, msg);
            },
        };
    }
}

fn u8_to_str(content: &[u8]) -> String {
    from_utf8(&content.to_vec()).unwrap().to_string()
}
