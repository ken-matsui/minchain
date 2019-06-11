use std::net::SocketAddr;
use std::collections::HashSet;

#[derive(Clone)]
pub struct CoreNodeList {
    pub list: HashSet<SocketAddr>,
}

impl CoreNodeList {
    pub fn new() -> CoreNodeList {
        CoreNodeList{ list: HashSet::new() }
    }

    /// Add a core node to the list.
    pub fn add(&mut self, peer: SocketAddr) {
        println!("Adding peer: ({})", peer);
        self.list.insert(peer);
        println!("Current Core List: {:?}", self.list);
    }

    /// Remove a core node that has left from the list.
    pub fn remove(&mut self, peer: &SocketAddr) {
        if self.list.contains(peer) {
            println!("Removing peer...");
            self.list.remove(peer);
            println!("Current Core list: {:?}", self.list);
        };
    } // TODO: 上手くremoveされないバグ有り

    /// Overwrite in bulk after checking the connection status of multiple peers.
    pub fn overwrite(&mut self, new_list: HashSet<SocketAddr>) {
        println!("core node list will be going to overwrite");
        self.list = new_list;
        println!("Current Core list: {:?}", self.list);
    }

    /// Return list of currently connected states peer.
    pub fn get_list(&self) -> HashSet<SocketAddr> {
        self.list.clone()
    }
}

impl std::fmt::Display for CoreNodeList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.list)
    }
}
