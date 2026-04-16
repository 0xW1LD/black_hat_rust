use crate::{
    error::Error,
    model::{CrtShEntry, ScanTarget, ScanTargetType},
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
    println!("[*] Gathering Entries...");
    let entries: Vec<CrtShEntry> = http_client
        .get(&format!("https://crt.sh/json?q={}", target))
        .send()
        .await?
        .json()
        .await?;

    let mut dns_resolver_opts = ResolverOpts::default();
    dns_resolver_opts.timeout = Duration::from_secs(4);

    let dns_resolver = AsyncResolver::tokio(ResolverConfig::default(), dns_resolver_opts);

    let mut subdomains: HashSet<String> = entries
        .into_iter()
        .map(|entry| {
            entry
                .name_value
                .split('\n')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        })
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
