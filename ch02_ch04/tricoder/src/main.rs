use anyhow::Error;
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::env;

mod common_ports;
mod error;
use crate::error::Error::CliUsage;
mod model;
use crate::model::Subdomain;
mod ports;
mod subdomains;

fn main() -> Result<(),Error>{
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(CliUsage.into());
    }
    let target = &args[1];

    let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(100)
        .build()?;

    pool.install(|| {
        let scan_result: Vec<Subdomain> =
            subdomains::enumerate(&http_client,target)
            .expect("main: Enumerating subdomains")
            .into_par_iter()
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