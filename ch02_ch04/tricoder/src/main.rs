use anyhow::Result;
use clap::Parser;
use reqwest::blocking::Client;

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

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    let target = &cli.domain;

    let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let pool = rayon::ThreadPoolBuilder::new().num_threads(100).build()?;

    pool.install(|| {
        let scan_result: Vec<Subdomain> = subdomains::enumerate(&http_client, target)
            .expect("main: Enumerating subdomains")
            .into_iter()
            .map(ports::scan_ports)
            .collect();

        for subdomain in scan_result {
            println!("{}", &subdomain.domain);
            for port in &subdomain.open_ports {
                println!("    {}", port.port);
            }
            println!("")
        }
    });
    println!("Scan Completed.");
    Ok(())
}
