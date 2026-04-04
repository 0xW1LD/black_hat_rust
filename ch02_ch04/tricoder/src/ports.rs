use crate::{
    common_ports::MOST_COMMON_PORTS_100,
    model::{Port, Subdomain},
};
use futures::{StreamExt, stream};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use tokio::net::TcpStream;

pub async fn scan_ports(concurrency: usize,subdomain: Subdomain) -> Subdomain {
    let mut subdomain = subdomain.clone();
    let socket_addresses: Vec<SocketAddr> = format!("{}:1337", subdomain.domain)
        .to_socket_addrs()
        .expect("port scanner: Creating socket address")
        .collect();
    if socket_addresses.len() == 0 {
        subdomainurn subdomain;
    }

    let socket_address = socket_addresses[0];
    subdomain.open_ports = stream::iter(MOST_COMMON_PORTS_100)
        .map(|port| scan_port(socket_address, *port))
        .buffer_unordered(concurrency)
        .filter(|port| futures::future::ready(port.is_open))
        .collect()
        .await;

    subdomain
}
async fn scan_port(mut socket_address: SocketAddr, port: u16) -> Port {
    let timeout = Duration::from_secs(3);
    socket_address.set_port(port);
    let is_open = matches!(
        tokio::time::timeout(timeout, TcpStream::connect(&socket_address)).await,
        Ok(Ok(_)),
    );

    Port {
        port: port,
        is_open,
    }
}
