use core::fmt;
use serde::Deserialize;
use std::fmt::Display;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub enum ScanTargetType {
    Domain(String),
    Ip(IpAddr),
}
#[derive(Debug, Clone)]
pub struct ScanTarget {
    pub target: ScanTargetType,
    pub open_ports: Vec<Port>,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub port: u16,
    pub is_open: bool,
}

#[derive(Debug, Clone)]
pub struct Vhost {
    pub vhost: String,
    pub is_valid: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CrtShEntry {
    pub name_value: String,
}

impl ScanTarget {
    pub fn new(target: ScanTargetType) -> Self {
        Self {
            target,
            open_ports: vec![],
        }
    }

    pub fn ports(&self) -> &[Port] {
        &self.open_ports
    }
    pub fn to_string(&self) -> String {
        match &self.target {
            ScanTargetType::Domain(domain) => domain.to_string(),
            ScanTargetType::Ip(ip) => ip.to_string(),
        }
    }
}

impl Display for ScanTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl TryFrom<String> for ScanTarget {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.parse::<IpAddr>() {
            Ok(ip) => Ok(ScanTarget {
                target: ScanTargetType::Ip(ip),
                open_ports: vec![],
            }),
            Err(_) => Ok(ScanTarget {
                target: ScanTargetType::Domain(value),
                open_ports: vec![],
            }),
        }
    }
}
