use crate::{
    error::Error,
    model::{CrtShEntry, Subdomain},
};
use futures::{stream,StreamExt};
use reqwest::Client;
use std::{collections::HashSet, time::Duration};
use trust_dns_resolver::{
    AsyncResolver, config::{ResolverConfig, ResolverOpts}, name_server::TokioConnectionProvider
};

type DnsResolver = AsyncResolver<TokioConnectionProvider>;

pub async fn enumerate(http_client: &Client, target: &str) -> Result<Vec<Subdomain>, Error> {
    println!("[*] Gathering Entries...");
    let entries: Vec<CrtShEntry> = http_client
        .get(&format!("https://crt.sh/json?q={}", target))
        .send()
        .await?
        .json()
        .await?;
    println!("[+] Found {} Entries!",entries.len());

    let mut dns_resolver_opts = ResolverOpts::default();
    dns_resolver_opts.timeout = Duration::from_secs(4);

    let dns_resolver = AsyncResolver::tokio(
        ResolverConfig::default(),
        dns_resolver_opts,
    );

    //clean & deduplicate subdomains
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

    let subdomains: Vec<Subdomain> = stream::iter(subdomains.into_iter())
        .map(|domain| Subdomain {
            domain,
            open_ports: Vec::new(),
        })
        .filter_map(|subdomain|{
            let dns_resolver = dns_resolver.clone();
            async move {
                if resolves(&dns_resolver,&subdomain).await {
                    Some(subdomain)
                } else {
                    None
                }
            }
        })
        .collect()
        .await;

    Ok(subdomains)
}

pub async fn resolves(dns_resolver: &DnsResolver, domain: &Subdomain) -> bool {
    dns_resolver.lookup_ip(&domain.domain).await.is_ok()
}
