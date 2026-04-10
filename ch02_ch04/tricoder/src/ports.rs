use crate::{
    common_ports::MOST_COMMON_PORTS_100,
    model::{Port, ScanTarget, ScanTargetType},
};
use futures::{StreamExt, stream};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use tokio::net::TcpStream;

pub async fn scan_ports(concurrency: usize, target: ScanTarget) -> ScanTarget {
    let mut target = target.clone();
    let socket_addresses: Vec<SocketAddr> = match &target.target {
        ScanTargetType::Domain(domain) => format!("{}:1337", domain)
            .to_socket_addrs()
            .expect("port scanner: Creating socket address")
            .collect(),
        ScanTargetType::Ip(ip) => vec![SocketAddr::new(*ip, 0)],
    };
    if socket_addresses.len() == 0 {
        return target;
    }

    let socket_address = socket_addresses[0];
    let open_ports: Vec<Port> = stream::iter(MOST_COMMON_PORTS_100)
        .map(|port| scan_port(socket_address, *port))
        .buffer_unordered(concurrency)
        .filter(|port| futures::future::ready(port.is_open))
        .collect()
        .await;
    target.open_ports = open_ports;

    target
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
