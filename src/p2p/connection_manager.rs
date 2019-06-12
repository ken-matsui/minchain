use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::str::from_utf8;
use std::net::{TcpListener, TcpStream, SocketAddr};

use p2p::message_manager::{MessageManager, MsgType};
use p2p::node_list::{NodeList, CoreNodeList, EdgeNodeList};

const PING_INTERVAL: Duration = Duration::from_secs(10);

fn u8_to_str(content: &[u8]) -> String {
    from_utf8(&content.to_vec()).unwrap().to_string()
}

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

#[derive(Clone)]
pub struct ConnectionManager4Edge {
    addr: SocketAddr,
    my_core_addr: SocketAddr,
    core_node_set: Arc<Mutex<CoreNodeList>>,
    mm: MessageManager,
}

impl ConnectionManager4Edge {
    pub fn new(self_addr: SocketAddr, my_core_addr: SocketAddr) -> ConnectionManager4Edge {
        println!("Initializing ConnectionManager4Edge ...");
        ConnectionManager4Edge {
            addr: self_addr,
            my_core_addr,
            core_node_set: Arc::new(Mutex::new(CoreNodeList::new())),
            mm: MessageManager::new(),
        }
    }

    /// Start standby. (for ClientCore)
    pub fn start(&mut self) { // FIXME: connection_managerと同じ内容
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
                self_clone.send_ping();
            });
        }
    }

    /// Connect to a known Core node specified by the user. (for ClientCore)
    pub fn connect_to_core_node(&mut self) {
        self.connect_to_p2pnw(self.my_core_addr);
    }

    /// 指定したCoreノードへ接続要求メッセージを送信する
    fn connect_to_p2pnw(&self, node_addr: SocketAddr) {
        let mut stream = TcpStream::connect(node_addr).unwrap();
        let msg = self.mm.build(MsgType::AddAsEdge, self.addr, None);
        thread::spawn(move || {
            stream.write(msg.as_bytes()).unwrap();
        });
    }

    #[allow(dead_code)]
    pub fn send_msg(&mut self, peer: &SocketAddr, msg: String) {
        println!("Sending ... {}", msg);
        match TcpStream::connect(peer) {
            Ok(mut stream) => {
                thread::spawn(move || {
                    stream.write(msg.as_bytes()).unwrap();
                });
            },
            Err(_) => {
                eprintln!("Connection failed for peer : {}", peer);
                self.core_node_set.lock().unwrap().list.remove(peer); // FIXME: connection_managerに同じ処理
                eprintln!("Trying to connect into P2P network ...");
                if self.core_node_set.lock().unwrap().get_list().len() != 0 {
                    let my_core_addr = self.core_node_set.lock().unwrap().get_top_peer();
                    self.my_core_addr = my_core_addr;
                    self.connect_to_core_node();
                    self.send_msg(&my_core_addr, msg);
                } else {
                    println!("No core node found in our list ...");
//                    self.ping_timer.cancel(); // TODO:
                }
            },
        }
    }

    /// Open the server socket and shift to standby mode.
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

    /// Process according to the received message.
    fn handle_message(&mut self, data: &String) {
        match self.mm.parse(data) {
            Ok(msg) => {
                println!("Connected by .. ({})", msg.my_addr);
                match msg.payload {
                    None => {
                        match msg.msg_type {
                            MsgType::Ping => {},
                            _ => {
                                // 接続情報以外のメッセージしかEdgeノードで処理することは想定していない
                                println!("Edge node does not have functions for this message!");
                            },
                        };
                    },
                    Some(mut pl) => {
                        match msg.msg_type {
                            MsgType::CoreList => {
                                // Coreノードに依頼してCoreノードのリストを受け取る口だけはある
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

    /// Send a message to confirm valid nodes.
    fn send_ping(&mut self) {
        match TcpStream::connect(self.my_core_addr) {
            Ok(mut stream) => {
                let msg = self.mm.build(MsgType::Ping, self.addr, None);
                thread::spawn(move || {
                    stream.write(msg.as_bytes()).unwrap();
                });
            },
            Err(_) => { // FIXME: self.send_msgと同じ内容
                println!("Connection failed for peer : {}", self.my_core_addr);
                self.core_node_set.lock().unwrap().remove(&self.my_core_addr);
                println!("Trying to connect into P2P network ...");
                if self.core_node_set.lock().unwrap().get_list().len() != 0 {
                    self.my_core_addr = self.core_node_set.lock().unwrap().get_top_peer();
                    self.connect_to_core_node();
                } else {
                    println!("No core node found in our list ...");
//                    self.ping_timer.cancel(); // TODO:
                }
            },
        };

        let mut self_clone = self.clone();
        thread::spawn(move || {
            thread::sleep(PING_INTERVAL);
            self_clone.send_ping();
        });
    }
}

impl Drop for ConnectionManager4Edge {
    /// Close socket. (Auto)
    fn drop(&mut self) -> () { // connection_close
        println!("Finishing ConnectionManager4Edge ...");
    }
}
