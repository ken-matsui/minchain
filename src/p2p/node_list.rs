use std::net::SocketAddr;
use std::collections::HashSet;

pub trait NodeList {
    /// Add a core node to the list.
    fn add(&mut self, _: SocketAddr);
    /// Remove a core node that has left from the list.
    fn remove(&mut self, _: &SocketAddr);
    /// Overwrite in bulk after checking the connection status of multiple peers.
    fn overwrite(&mut self, _: HashSet<SocketAddr>);
    /// Return list of currently connected states peer.
    fn get_list(&self) -> HashSet<SocketAddr>;
    /// Return peer at the top of the list.
    fn get_top_peer(&self) -> SocketAddr;
}

#[derive(Clone)]
pub struct CoreNodeList {
    list: HashSet<SocketAddr>,
}

impl CoreNodeList {
    pub fn new() -> CoreNodeList {
        CoreNodeList{ list: HashSet::new() }
    }
}

impl NodeList for CoreNodeList {
    fn add(&mut self, peer: SocketAddr) {
        println!("Adding peer: ({})", peer);
        self.list.insert(peer);
        println!("Current Core List: {:?}", self.list);
    }

    fn remove(&mut self, peer: &SocketAddr) {
        if self.list.contains(peer) {
            println!("Removing peer ... ({})", *peer);
            self.list.remove(peer);
            println!("Current Core list: {:?}", self.list);
        };
    }

    fn overwrite(&mut self, new_list: HashSet<SocketAddr>) {
        println!("core node list will be going to overwrite");
        self.list = new_list;
        println!("Current Core list: {:?}", self.list);
    }

    fn get_list(&self) -> HashSet<SocketAddr> {
        self.list.clone()
    }

    fn get_top_peer(&self) -> SocketAddr {
        let mut vec = Vec::new();
        vec.extend(self.list.clone().into_iter());
        vec[0]
    }
}

impl CoreNodeList {
    /// 与えられたpeerがリストに含まれているかどうかをチェックする
    pub fn has_this_peer(&self, peer: &SocketAddr) -> bool {
        self.list.contains(peer)
    }
}

impl std::fmt::Display for CoreNodeList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self.list)
    }
}

#[derive(Clone)]
pub struct EdgeNodeList {
    list: HashSet<SocketAddr>,
}

impl EdgeNodeList {
    pub fn new() -> EdgeNodeList {
        EdgeNodeList { list: HashSet::new() }
    }
}

impl NodeList for EdgeNodeList {
    fn add(&mut self, edge: SocketAddr) {
        println!("Adding edge: ({})", edge);
        self.list.insert(edge);
        println!("Current Edge List: {:?}", self.list);
    }

    fn remove(&mut self, edge: &SocketAddr) {
        if self.list.contains(edge) {
            println!("Removing edge ... ({})", *edge);
            self.list.remove(edge);
            println!("Current Edge list: {:?}", self.list);
        };
    }

    #[allow(dead_code)]
    fn overwrite(&mut self, new_list: HashSet<SocketAddr>) {
        println!("edge node list will be going to overwrite");
        self.list = new_list;
        println!("Current Edge list: {:?}", self.list);
    }

    #[allow(dead_code)]
    fn get_list(&self) -> HashSet<SocketAddr> {
        self.list.clone()
    }

    #[allow(dead_code)]
    fn get_top_peer(&self) -> SocketAddr {
        let mut vec = Vec::new();
        vec.extend(self.list.clone().into_iter());
        vec[0]
    }
}

impl std::fmt::Display for EdgeNodeList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self.list)
    }
}
