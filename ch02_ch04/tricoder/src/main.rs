use anyhow::Result;
use clap::Parser;
use futures::{StreamExt, stream};
use reqwest::{Client, redirect};
use std::path::PathBuf;
use std::time::{Duration, Instant};

mod common_ports;
mod error;
mod model;
use crate::model::{
    ScanTarget,
    ScanTargetType::{Domain, Ip},
};
mod ports;
mod subdomains;
mod vhosts;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Domain/IP to target for scan
    #[arg(short, long)]
    target: String,

    /// If set swaps from DNS to Vhost Subdomain scan
    #[arg(short, long)]
    wordlist: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    let target = ScanTarget::try_from(cli.target)?;
    let wordlist = cli.wordlist;

    let http_client = Client::builder()
        .timeout(Duration::from_secs(60))
        //.proxy(reqwest::Proxy::all("http://127.0.0.1:8080")?)
        //.danger_accept_invalid_certs(true)
        .redirect(redirect::Policy::none())
        .http1_only()
        .build()?;

    let port_concurrency = 100;
    let vhost_concurrency = 50;
    let scan_concurrency = 10;

    let scan_start = Instant::now();
    let scan_result: Vec<ScanTarget>;

    let targets: Vec<ScanTarget> = match (target.target, wordlist) {
        (Domain(domain), Some(wl)) => {
            vhosts::enumerate(&http_client, domain, wl, vhost_concurrency).await?;
            let scan_duration = scan_start.elapsed();
            println!("[+] Scan finished in: {:?}", scan_duration);
            return Ok(());
        }
        (Domain(domain), None) => {
            println!("[*] Domain found, enumerating subdomains...");
            subdomains::enumerate(&http_client, &domain)
                .await?
                .into_iter()
                .collect()
        }
        (Ip(ip), wl) => {
            println!("[*] Ip found, skipping subdomain enumeration...");
            match wl {
                Some(_) => println!("[*] Wordlist was found but not needed, ignoring..."),
                None => {}
            }
            vec![ScanTarget::new(Ip(ip))]
        }
    };

    println!("[*] Scanning Ports...");
    scan_result = stream::iter(targets)
        .map(|ip| ports::scan_ports(port_concurrency, ip))
        .buffer_unordered(scan_concurrency)
        .collect()
        .await;

    let scan_duration = scan_start.elapsed();
    println!("[+] Scan finished in: {:?}", scan_duration);

    for target in scan_result {
        println!("[+] Target: {}", target);
        println!("{:<10} STATE","PORT");
        for port in target.ports() {
            println!("{:<10} {}",port.port,"OPEN")
        }
    }
    println!("Scan Completed.");
    Ok(())
}
