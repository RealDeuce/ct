use std::collections::HashSet;
use std::env;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;

use cepheus_trader_server::admin_psk;
use cepheus_trader_server::server::{self, AdminTlsConfig};
use tokio::net::lookup_host;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let mut listen = Vec::new();
    let mut admin_listen = Vec::new();
    let mut sysop_listen = Vec::new();
    let mut data = PathBuf::from("server-data");
    let mut admin_psk_file = None;
    let mut backup_dir = None;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--listen" => {
                listen.push(
                    arguments
                        .next()
                        .unwrap_or_else(|| usage("--listen needs an address")),
                );
            }
            "--data" => {
                data = PathBuf::from(
                    arguments
                        .next()
                        .unwrap_or_else(|| usage("--data needs a directory")),
                );
            }
            "--admin-listen" => {
                admin_listen.push(
                    arguments
                        .next()
                        .unwrap_or_else(|| usage("--admin-listen needs an address")),
                );
            }
            "--sysop-listen" => {
                sysop_listen.push(
                    arguments
                        .next()
                        .unwrap_or_else(|| usage("--sysop-listen needs an address")),
                );
            }
            "--admin-psk-file" => {
                admin_psk_file = Some(PathBuf::from(
                    arguments
                        .next()
                        .unwrap_or_else(|| usage("--admin-psk-file needs a path")),
                ));
            }
            "--backup-dir" => {
                backup_dir = Some(PathBuf::from(
                    arguments
                        .next()
                        .unwrap_or_else(|| usage("--backup-dir needs a directory")),
                ));
            }
            "--version" | "-V" => {
                println!("cepheus-trader-server {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--help" | "-h" => usage(""),
            unknown => usage(&format!("unknown argument: {unknown}")),
        }
    }
    let addresses = resolve_or_default(&listen, 7323)
        .await
        .unwrap_or_else(|error| usage(&format!("invalid --listen address: {error}")));
    let admin_addresses = resolve_or_default(&admin_listen, 7324)
        .await
        .unwrap_or_else(|error| usage(&format!("invalid --admin-listen address: {error}")));
    let sysop_addresses = resolve_or_default(&sysop_listen, 7325)
        .await
        .unwrap_or_else(|error| usage(&format!("invalid --sysop-listen address: {error}")));
    let admin_psk_file = admin_psk_file.unwrap_or_else(|| data.join("admin.psk"));
    let backup_dir = backup_dir.unwrap_or_else(|| PathBuf::from("server-backups"));
    let admin_key = admin_psk::load_or_create(&admin_psk_file).unwrap_or_else(|error| {
        usage(&format!(
            "cannot load administrator PSK {}: {error}",
            admin_psk_file.display()
        ))
    });
    let admin_tls = AdminTlsConfig {
        key: Arc::new(admin_key),
        backup_root: Arc::new(backup_dir),
    };
    if let Err(error) =
        server::run_on_addresses(addresses, admin_addresses, sysop_addresses, data, admin_tls).await
    {
        server::log(format_args!("fatal server error: {error}"));
        std::process::exit(1);
    }
}

async fn resolve_listener_addresses(specifications: &[String]) -> Result<Vec<SocketAddr>, String> {
    let mut seen = HashSet::new();
    let mut addresses = Vec::new();
    for specification in specifications {
        let resolved = lookup_host(specification.as_str())
            .await
            .map_err(|error| format!("cannot resolve {specification:?}: {error}"))?;
        let mut found = false;
        for address in resolved {
            found = true;
            if seen.insert(address) {
                addresses.push(address);
            }
        }
        if !found {
            return Err(format!("{specification:?} resolved to no addresses"));
        }
    }
    Ok(addresses)
}

async fn resolve_or_default(
    specifications: &[String],
    default_port: u16,
) -> Result<Vec<SocketAddr>, String> {
    if !specifications.is_empty() {
        return resolve_listener_addresses(specifications).await;
    }
    let defaults = resolve_listener_addresses(&[format!("localhost:{default_port}")]).await?;
    let supported = defaults
        .into_iter()
        .filter(|address| socket_family_available(*address))
        .collect::<Vec<_>>();
    if supported.is_empty() {
        Err("localhost has no supported IPv4 or IPv6 addresses".into())
    } else {
        Ok(supported)
    }
}

fn socket_family_available(address: SocketAddr) -> bool {
    let loopback = if address.is_ipv4() {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        IpAddr::V6(Ipv6Addr::LOCALHOST)
    };
    TcpListener::bind(SocketAddr::new(loopback, 0)).is_ok()
}

fn usage(error: &str) -> ! {
    if !error.is_empty() {
        eprintln!("{error}");
    }
    eprintln!(
        "Usage: cepheus-trader-server [--listen HOST:PORT]... [--data DIRECTORY] \\\n\
         [--admin-listen LOOPBACK_HOST:PORT]... [--sysop-listen HOST:PORT]... \\\n\
         [--admin-psk-file PATH] [--backup-dir DIRECTORY] [--version]"
    );
    std::process::exit(if error.is_empty() { 0 } else { 2 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hostname_resolution_keeps_every_unique_address() {
        let specifications = vec!["localhost:7323".into(), "localhost:7323".into()];
        let addresses = resolve_listener_addresses(&specifications).await.unwrap();
        assert!(!addresses.is_empty());
        let unique = addresses.iter().copied().collect::<HashSet<_>>();
        assert_eq!(addresses.len(), unique.len());
        assert!(addresses.iter().all(|address| address.port() == 7323));
    }

    #[tokio::test]
    async fn defaults_keep_all_supported_localhost_families() {
        let resolved = resolve_listener_addresses(&["localhost:7323".into()])
            .await
            .unwrap();
        let expected = resolved
            .into_iter()
            .filter(|address| socket_family_available(*address))
            .collect::<HashSet<_>>();
        let defaults = resolve_or_default(&[], 7323).await.unwrap();
        assert_eq!(defaults.into_iter().collect::<HashSet<_>>(), expected);
    }
}
