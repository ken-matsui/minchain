use std::net::SocketAddr;
use std::collections::HashSet;

#[derive(Clone)]
pub struct EdgeNodeList {
    pub list: HashSet<SocketAddr>,
}

impl EdgeNodeList {
    pub fn new() -> EdgeNodeList {
        EdgeNodeList{ list: HashSet::new() }
    }

    /// Add a edge node to the list.
    pub fn add(&mut self, edge: SocketAddr) {
        println!("Adding edge: ({})", edge);
        self.list.insert(edge);
        println!("Current Edge List: {:?}", self.list);
    }

    /// Remove a edge node that has left from the list.
    pub fn remove(&mut self, edge: &SocketAddr) {
        if self.list.contains(edge) {
            println!("Removing edge ... ({})", *edge);
            self.list.remove(edge);
            println!("Current Edge list: {:?}", self.list);
        };
    }

    /// Overwrite in bulk after checking the connection status of multiple edge nodes.
    #[allow(dead_code)]
    pub fn overwrite(&mut self, new_list: HashSet<SocketAddr>) {
        println!("edge node list will be going to overwrite");
        self.list = new_list;
        println!("Current Edge list: {:?}", self.list);
    }

    /// Return list of currently connected states edge nodes.
    #[allow(dead_code)]
    pub fn get_list(&self) -> HashSet<SocketAddr> {
        self.list.clone()
    }
}

impl std::fmt::Display for EdgeNodeList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.list)
    }
}
