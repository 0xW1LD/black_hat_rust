use crate::{
    error::Error,
    model::{ScanTarget, ScanTargetType, Vhost},
};
use futures::{StreamExt, stream};
use http::StatusCode;
use reqwest::{Client, header::HOST};
use std::path::PathBuf;
use std::io::{BufRead,BufReader};
use std::fs::File;

pub async fn enumerate(
    http_client: &Client,
    target: String,
    wl: PathBuf,
    concurrency: usize,
) -> Result<Vec<ScanTarget>, Error> {
    let file = File::open(wl)?;
    let list = BufReader::new(file).lines().filter_map(|l|l.ok());
    println!("[*] Loading entries from wordlist...");
    let url = match http_client.get(format!("https://{}", target)).send().await {
        Ok(_) => format!("https://{}", target),
        Err(_) => format!("http://{}", target),
    };
    let base = http_client
        .get(&url)
        .header(HOST, format!("w1ld_fake_domain.{}", target))
        .send()
        .await?;

    let base_status = base.status();
    let base_len = base.content_length().unwrap_or(0);

    println!("[*] Fuzzing Subdomains for:{}...", &url);
    let vhosts: Vec<Vhost> = stream::iter(list)
        .map(|line| {
            scan_vhost(
                line,
                http_client,
                &target,
                base_status,
                base_len,
                &url,
            )
        })
        .buffer_unordered(concurrency)
        .filter(|v| futures::future::ready(v.is_valid))
        .collect()
        .await;

    println!("[+] Found: {} vhosts", vhosts.len());

    let targets: Vec<ScanTarget> = vhosts
        .iter()
        .map(|v| ScanTarget {
            target: ScanTargetType::Domain(v.vhost.clone()),
            open_ports: vec![],
        })
        .collect();
    Ok(targets)
}

async fn scan_vhost(
    vhost: String,
    http_client: &Client,
    target: &String,
    base_status: StatusCode,
    base_len: u64,
    url: &String,
) -> Vhost {
    let resp = http_client
        .get(url)
        .header(HOST, format!("{}.{}", vhost, target))
        .send()
        .await
        .unwrap();
    let vhost_status = resp.status();
    let vhost_len = resp.content_length().unwrap_or(0);

    let is_valid = { base_status != vhost_status || base_len != vhost_len };
    if is_valid {
        println!(
            "       {:<25}[Status Code: {}, Content Length: {}]",
            vhost, vhost_status, vhost_len
        )
    };

    Vhost {
        vhost: vhost,
        is_valid,
    }
}
