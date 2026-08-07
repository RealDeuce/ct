use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use cepheus_trader_server::admin_psk;
use cepheus_trader_server::server::{self, AdminTlsConfig};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let mut listen = "127.0.0.1:7323".to_owned();
    let mut admin_listen = "127.0.0.1:7324".to_owned();
    let mut sysop_listen = "127.0.0.1:7325".to_owned();
    let mut data = PathBuf::from("server-data");
    let mut admin_psk_file = None;
    let mut backup_dir = None;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--listen" => {
                listen = arguments
                    .next()
                    .unwrap_or_else(|| usage("--listen needs an address"));
            }
            "--data" => {
                data = PathBuf::from(
                    arguments
                        .next()
                        .unwrap_or_else(|| usage("--data needs a directory")),
                );
            }
            "--admin-listen" => {
                admin_listen = arguments
                    .next()
                    .unwrap_or_else(|| usage("--admin-listen needs an address"));
            }
            "--sysop-listen" => {
                sysop_listen = arguments
                    .next()
                    .unwrap_or_else(|| usage("--sysop-listen needs an address"));
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
    let address: SocketAddr = listen
        .parse()
        .unwrap_or_else(|_| usage("invalid --listen address"));
    let admin_address: SocketAddr = admin_listen
        .parse()
        .unwrap_or_else(|_| usage("invalid --admin-listen address"));
    let sysop_address: SocketAddr = sysop_listen
        .parse()
        .unwrap_or_else(|_| usage("invalid --sysop-listen address"));
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
    if let Err(error) = server::run(address, admin_address, sysop_address, data, admin_tls).await {
        eprintln!("fatal server error: {error}");
        std::process::exit(1);
    }
}

fn usage(error: &str) -> ! {
    if !error.is_empty() {
        eprintln!("{error}");
    }
    eprintln!(
        "Usage: cepheus-trader-server [--listen ADDRESS] [--data DIRECTORY] \\\n\
         [--admin-listen LOOPBACK_ADDRESS] [--sysop-listen ADDRESS] \\\n\
         [--admin-psk-file PATH] [--backup-dir DIRECTORY] [--version]"
    );
    std::process::exit(if error.is_empty() { 0 } else { 2 });
}
