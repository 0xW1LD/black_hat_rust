use anyhow::Result;
use clap::Parser;
use futures::{StreamExt, stream};
use reqwest::Client;
use std::{net::IpAddr, time::{Duration, Instant}};

mod common_ports;
mod error;
mod model;
use crate::model::{IpAddress, ScanTarget, Subdomain};
mod ports;
mod subdomains;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    target: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    let target = match &cli.target.parse::<IpAddr>() {
        Ok(ip) => ScanTarget::Ip(IpAddress{ip: *ip, open_ports: vec![]}),
        Err(_) => ScanTarget::Domain(Subdomain{domain: cli.target,open_ports: vec![]})
    };

    let http_client = Client::builder()
        .timeout(Duration::from_secs(60))
        //.proxy(reqwest::Proxy::http("http://127.0.0.1:8080")?)
        //.danger_accept_invalid_certs(true)
        .build()?;

    let port_concurrency = 100;
    let scan_concurrency = 10;

    let scan_start = Instant::now();
    let scan_result: Vec<ScanTarget>;

    let targets: Vec<ScanTarget> = match target{
         ScanTarget::Domain(domain) => {
            println!("[*] Domain found, enumerating subdomains...");
            subdomains::enumerate(&http_client,&domain.domain)
                .await?
                .into_iter()
                .map(ScanTarget::Domain)
                .collect()
        },
        ScanTarget::Ip(ip) => {
            println!("[*] Ip found, skipping subdomain enumeration...");
            vec![ScanTarget::Ip(ip)]
        },
    };

    println!("[*] Scanning Ports...");
    scan_result = stream::iter(targets)
    .map(|ip| ports::scan_ports(port_concurrency, ip))
    .buffer_unordered(scan_concurrency)
    .collect()
    .await;


    let scan_duration = scan_start.elapsed();
    println!("[+] Scan finished in: {:?}",scan_duration);

    for target in scan_result {
        match target {
            ScanTarget::Domain(domain) => {
                println!("[+] Target: {}", &domain.domain);
                for port in &domain.open_ports {
                    println!("      {}",port.port)
                }
            }
            ScanTarget::Ip(ip) => {
                println!("[+] Target: {}", &ip.ip);
                for port in &ip.open_ports {
                    println!("      {}",port.port)
                }

            }
        }
    }
    println!("Scan Completed.");
    Ok(())
}
