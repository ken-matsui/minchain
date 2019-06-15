use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Clone)]
pub enum State {
    Init,
    Standby,
    Active,
    ConnectedToNetwork,
    ShuttingDown,
}

#[inline]
pub fn get_my_addr(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}
