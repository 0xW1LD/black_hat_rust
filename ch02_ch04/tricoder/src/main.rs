use anyhow::Result;
use clap::Parser;
use futures::{stream, StreamExt};
use reqwest::Client;
use std::time::{Duration, Instant};

mod common_ports;
mod error;
mod model;
use crate::model::Subdomain;
mod ports;
mod subdomains;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    domain: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    let target = &cli.domain;

    let http_client = Client::builder()
        .timeout(Duration::from_secs(60))
        //.proxy(reqwest::Proxy::http("http://127.0.0.1:8080")?)
        //.danger_accept_invalid_certs(true)
        .build()?;

    let port_concurrency = 100;
    let subdomain_concurrency = 10;

    let scan_start = Instant::now();

    println!("[*] Enumerating Subdomains...");
    let subdomains = subdomains::enumerate(&http_client,target).await?;

    println!("[*] Scanning Ports...");
    let scan_result: Vec<Subdomain> = stream::iter(subdomains.into_iter())
        .map(|subdomain| ports::scan_ports(port_concurrency,subdomain))
        .buffer_unordered(subdomain_concurrency)
        .collect()
        .await;

    let scan_duration = scan_start.elapsed();
    println!("[+] Scan finished in: {:?}",scan_duration);

    for subdomain in scan_result {
        println!("[+] Subdomain: {}", &subdomain.domain);
        for port in &subdomain.open_ports {
            println!("    {}", port.port);
        }
        println!("")
    }
    println!("Scan Completed.");
    Ok(())
}
