use std::net::{IpAddr, Ipv4Addr, SocketAddr};



pub const PORT: u16 = 42069;
pub const LOOPBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
pub const LOOPBACK_SOCKET: SocketAddr = SocketAddr::new(LOOPBACK, PORT);

/// How long the server should wait between checking for client messages
pub const SERVER_POLL_DELAY_MS: u64 = 200;
