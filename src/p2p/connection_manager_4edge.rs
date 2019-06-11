use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::str::from_utf8;
use std::net::{TcpListener, TcpStream, SocketAddr};
use p2p::message_manager::{MessageManager, MsgType};
use p2p::core_node_list::CoreNodeList;

const PING_INTERVAL: Duration = Duration::from_secs(10);

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

fn u8_to_str(content: &[u8]) -> String { // FIXME: connection_managerに同じのある
    from_utf8(&content.to_vec()).unwrap().to_string()
}
