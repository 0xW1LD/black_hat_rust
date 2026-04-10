use core::fmt;
use serde::Deserialize;
use std::fmt::Display;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub enum ScanTarget {
    Domain(Subdomain),
    Ip(IpAddress),
}
#[derive(Debug, Clone)]
pub struct IpAddress {
    pub ip: IpAddr,
    pub open_ports: Vec<Port>,
}

#[derive(Debug, Clone)]
pub struct Subdomain {
    pub domain: String,
    pub open_ports: Vec<Port>,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub port: u16,
    pub is_open: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CrtShEntry {
    pub name_value: String,
}

impl ScanTarget {
    pub fn ports(&self) -> Vec<Port> {
        match &self {
            ScanTarget::Domain(domain) => domain.open_ports.clone(),
            ScanTarget::Ip(ip) => ip.open_ports.clone(),
        }
    }
}

impl Display for ScanTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ScanTarget::Domain(domain) => write!(f, "{}", domain.domain),
            ScanTarget::Ip(ip) => write!(f, "{}", ip.ip),
        }
    }
}

impl TryFrom<String> for ScanTarget {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.parse::<IpAddr>() {
            Ok(ip) => Ok(ScanTarget::Ip(IpAddress {
                ip,
                open_ports: vec![],
            })),
            Err(_) => Ok(ScanTarget::Domain(Subdomain {
                domain: value,
                open_ports: vec![],
            })),
        }
    }
}
