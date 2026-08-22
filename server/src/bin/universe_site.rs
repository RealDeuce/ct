use std::env;
use std::path::PathBuf;

use cepheus_trader_server::atlas::{AtlasVisibility, initial_snapshot, read_snapshot, write_site};

fn main() {
    if let Err(error) = run() {
        eprintln!("universe site generation failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut data = None;
    let mut output = None;
    let mut initial_universe = false;
    let mut visibility = AtlasVisibility::UniversallyKnown;
    let mut arguments = env::args().skip(1);

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--data" => data = Some(PathBuf::from(required_value(&mut arguments, "--data")?)),
            "--output" => output = Some(PathBuf::from(required_value(&mut arguments, "--output")?)),
            "--initial-universe" => initial_universe = true,
            "--visibility" => {
                visibility = match required_value(&mut arguments, "--visibility")?.as_str() {
                    "universally-known" => AtlasVisibility::UniversallyKnown,
                    "omniscient" => AtlasVisibility::Omniscient,
                    value => {
                        return Err(format!(
                            "unknown visibility {value:?}; use universally-known or omniscient"
                        )
                        .into());
                    }
                }
            }
            "--version" | "-V" => {
                println!("cepheus-trader-universe-site {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                usage();
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {argument}").into()),
        }
    }

    let output = output.ok_or("--output DIRECTORY is required")?;
    if initial_universe && data.is_some() {
        return Err("use exactly one of --data DIRECTORY and --initial-universe".into());
    }
    let snapshot = if initial_universe {
        if visibility == AtlasVisibility::Omniscient {
            return Err("--initial-universe only supports universally-known visibility".into());
        }
        initial_snapshot()
    } else {
        let data = data.ok_or("use exactly one of --data DIRECTORY and --initial-universe")?;
        read_snapshot(data, visibility)?
    };

    write_site(&snapshot, &output)?;
    println!(
        "wrote {} {} systems at game second {} to {}",
        snapshot.systems.len(),
        snapshot.visibility.as_str(),
        snapshot.game_second,
        output.display()
    );
    Ok(())
}

fn required_value(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    arguments
        .next()
        .ok_or_else(|| format!("{option} requires a value").into())
}

fn usage() {
    println!(
        "\
Generate a self-contained Cepheus Trader browser atlas snapshot.

Usage:
  cepheus-trader-universe-site --data DIRECTORY --output DIRECTORY [OPTIONS]
  cepheus-trader-universe-site --initial-universe --output DIRECTORY

Options:
  --data DIRECTORY       Existing Cepheus Trader LMDB directory (opened read-only)
  --initial-universe     Generate the fixed initial Federation map without a database
  --output DIRECTORY     New directory to create; existing paths are never replaced
  --visibility SCOPE     universally-known (default) or omniscient
  -h, --help             Show this help
  -V, --version          Show the program version
"
    );
}
