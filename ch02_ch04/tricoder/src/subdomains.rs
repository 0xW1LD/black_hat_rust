use crate::{
    error::Error,
    model::{CrtEntry, ScanTarget, ScanTargetType},
};
use futures::{StreamExt, stream};
use reqwest::{Client};
use std::{collections::HashSet, time::Duration};
use trust_dns_resolver::{
    AsyncResolver,
    config::{ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
};

type DnsResolver = AsyncResolver<TokioConnectionProvider>;

pub async fn enumerate(http_client: &Client, target: &str, concurrency: usize) -> Result<Vec<ScanTarget>, Error> {
    dotenvy::dotenv()?;
    let token = std::env::var("API_TOKEN")?;

    println!("[*] Gathering Entries...");
    let entries: Vec<CrtEntry> = http_client
        .get(&format!("https://api.certspotter.com/v1/issuances?domain={}&include_subdomains=true&expand=dns_names", target))
        .bearer_auth(token)
        .send()
        .await?
        .json()
        .await?;

    let mut dns_resolver_opts = ResolverOpts::default();
    dns_resolver_opts.timeout = Duration::from_secs(4);

    let dns_resolver = AsyncResolver::tokio(ResolverConfig::default(), dns_resolver_opts);

    let mut subdomains: HashSet<String> = entries
        .into_iter()
        .map(|entry| entry.dns_names)
        .flatten()
        .filter(|subdomain: &String| subdomain != target)
        .filter(|subdomain: &String| !subdomain.contains("*"))
        .collect();

    subdomains.insert(target.to_string());
    println!("[+] Found {} Subdomains!", subdomains.len());

    let subdomains: Vec<ScanTarget> = stream::iter(subdomains.into_iter())
        .map(|domain| {
            let dns_resolver = dns_resolver.clone();
            async move {
            if resolves(&dns_resolver,&domain).await {
                Some(ScanTarget {
                    target: ScanTargetType::Domain(domain),
                    open_ports: Vec::new(),
                })
            } else {
                None
            }
            }
        })
        .buffer_unordered(concurrency)
        .filter_map(|s| futures::future::ready(match s {
            Some(target) => Some(target),
            None => None,
        }))
        .collect::<Vec<ScanTarget>>()
        .await;
    println!("[+] Resolved {} Subdomains!", subdomains.len());

    Ok(subdomains)
}

pub async fn resolves(dns_resolver: &DnsResolver, domain: &String) -> bool {
    dns_resolver.lookup_ip(domain).await.is_ok()
}
