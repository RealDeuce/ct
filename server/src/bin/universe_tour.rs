use std::env;
use std::path::PathBuf;
use std::time::Instant;

use cepheus_trader_server::cpu_time::thread_cpu_time_ns;
use cepheus_trader_server::simulation::{MessageClass, SECONDS_PER_DAY, SimulationReport};
use cepheus_trader_server::store::{BbsSettings, ConfigureBbsResult, Store};
use cepheus_trader_server::universe::INITIAL_SYSTEMS;
use cepheus_trader_server::wire::COMMAND_ID_BYTES;

fn main() {
    if let Err(error) = run() {
        eprintln!("universe tour failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut data = None;
    let mut days = 28_u64;
    let mut shown_legs = 8_usize;
    let mut bbs_count = 0_u32;
    let mut list_systems = false;
    let mut shown_system_cpu = 20_usize;
    let mut map_size_gib = 1_usize;
    let mut settlement_edge = false;
    let mut initialize_only = false;
    let mut maximum_events = None;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--data" => data = Some(PathBuf::from(required_value(&mut arguments, "--data")?)),
            "--days" => days = required_value(&mut arguments, "--days")?.parse()?,
            "--show-legs" => shown_legs = required_value(&mut arguments, "--show-legs")?.parse()?,
            "--bbs-count" => bbs_count = required_value(&mut arguments, "--bbs-count")?.parse()?,
            "--map-size-gib" => {
                map_size_gib = required_value(&mut arguments, "--map-size-gib")?.parse()?
            }
            "--settlement-edge" => settlement_edge = true,
            "--initialize-only" => initialize_only = true,
            "--max-events" => {
                maximum_events = Some(required_value(&mut arguments, "--max-events")?.parse()?)
            }
            "--list-systems" => list_systems = true,
            "--show-system-cpu" => {
                let value = required_value(&mut arguments, "--show-system-cpu")?;
                shown_system_cpu = if value == "all" {
                    usize::MAX
                } else {
                    value.parse()?
                };
            }
            "--version" | "-V" => {
                println!("cepheus-trader-universe-tour {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => usage(None),
            _ => usage(Some(&format!("unknown argument: {argument}"))),
        }
    }
    let data = data.ok_or(
        "--data DIRECTORY is required; the tour never chooses or deletes a database for you",
    )?;
    if map_size_gib == 0 {
        return Err("--map-size-gib must be positive".into());
    }
    let map_size_bytes = map_size_gib
        .checked_mul(1024 * 1024 * 1024)
        .ok_or("LMDB map size overflow")?;
    let store = Store::open_with_map_size(&data, map_size_bytes)?;
    if store.stellar_systems()?.is_empty() {
        if bbs_count > 200 {
            return Err("--bbs-count must be no greater than 200".into());
        }
        let seeds = (0..INITIAL_SYSTEMS.len())
            .map(|index| [0x40_u8.wrapping_add(index as u8); 32])
            .collect::<Vec<_>>();
        store.initialize_universe(&[0xf0; COMMAND_ID_BYTES], *b"CT-TOUR-UNIV-v1!", &seeds, &[])?;
        for index in 0..bbs_count {
            let ordinal = u8::try_from(index + 1)?;
            let credential = store.add_bbs(
                &[0x10_u8.wrapping_add(ordinal); COMMAND_ID_BYTES],
                &format!("Baseline BBS {ordinal}"),
                [0x30_u8.wrapping_add(ordinal); 32],
            )?;
            let settings = BbsSettings {
                bbs_name: format!("Baseline BBS {ordinal}"),
                polity_name: format!("Baseline Polity {ordinal}"),
                trade_combat: ((index % 3) * 50) as u8,
                chaos_order: (((index / 3) % 3) * 50) as u8,
            };
            match store.configure_bbs(
                credential.bbs_id,
                &[0x50_u8.wrapping_add(ordinal); COMMAND_ID_BYTES],
                0,
                &settings,
                [0x90_u8.wrapping_add(ordinal); 32],
            )? {
                ConfigureBbsResult::Updated(configuration) if configuration.configured => {}
                other => {
                    return Err(format!("BBS {ordinal} configuration failed: {other:?}").into());
                }
            }
        }
        println!(
            "initialized deterministic Federation with {} BBS polities at {}",
            bbs_count,
            data.display()
        );
    } else {
        if bbs_count != 0 && store.bbs_credentials()?.len() != bbs_count as usize {
            return Err(format!(
                "existing database has {} BBS credentials, not requested --bbs-count {}",
                store.bbs_credentials()?.len(),
                bbs_count
            )
            .into());
        }
        println!("continuing universe at {}", data.display());
    }

    let mut systems = store.simulation_systems()?;
    let bbs_prime_extent = store
        .bbs_credentials()?
        .into_iter()
        .map(|credential| -> Result<f64, Box<dyn std::error::Error>> {
            let home = store
                .bbs_home(credential.bbs_id)?
                .ok_or_else(|| format!("BBS {} has no materialized home", credential.bbs_id))?;
            let capital = systems
                .iter()
                .find(|system| system.system_id == home.capital_system_id)
                .ok_or_else(|| format!("BBS {} capital is missing", credential.bbs_id))?;
            Ok(capital
                .position_parsecs
                .iter()
                .map(|coordinate| coordinate * coordinate)
                .sum::<f64>()
                .sqrt())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .fold(0.0_f64, f64::max);
    if settlement_edge {
        if bbs_prime_extent <= 0.0 {
            return Err("--settlement-edge requires at least one materialized BBS polity".into());
        }
        let storage_before_materialization = store.storage_bytes()?;
        let wall_start = Instant::now();
        let cpu_start = thread_cpu_time_ns();
        let materialized =
            store.materialize_settlement_capacity_fixture(bbs_prime_extent, [0xc7; 32])?;
        let wall_seconds = wall_start.elapsed().as_secs_f64();
        let cpu_seconds = cpu_start
            .zip(thread_cpu_time_ns())
            .map(|(before, after)| after.saturating_sub(before) as f64 / 1_000_000_000.0);
        let storage_after_materialization = store.storage_bytes()?;
        println!(
            "settlement-materialization: E={:.6} normal-through={:.6} outer-edge={:.6} sampled-candidates={} added-systems={} inhabited-added={} falloff-added={} forced-uninhabited-added={} resolved-cells-added={} total-systems={}",
            materialized.settlement_extent_parsecs,
            materialized.normal_population_radius_parsecs,
            materialized.outer_radius_parsecs,
            materialized.sampled_candidates,
            materialized.stellar_components_added,
            materialized.inhabited_components_added,
            materialized.falloff_components_added,
            materialized.forced_uninhabited_components_added,
            materialized.resolved_cells_added,
            materialized.total_systems,
        );
        println!(
            "settlement-materialization-performance: wall-seconds={wall_seconds:.6} thread-cpu-seconds={} storage-before-bytes={} storage-after-bytes={} storage-growth-bytes={}",
            cpu_seconds
                .map(|value| format!("{value:.6}"))
                .unwrap_or_else(|| "unavailable".into()),
            storage_before_materialization,
            storage_after_materialization,
            storage_after_materialization.saturating_sub(storage_before_materialization),
        );
        systems = store.simulation_systems()?;
    }
    let maximum_distance = systems
        .iter()
        .map(|system| {
            system
                .position_parsecs
                .iter()
                .map(|coordinate| coordinate * coordinate)
                .sum::<f64>()
                .sqrt()
        })
        .fold(0.0_f64, f64::max);
    println!(
        "systems={} bbs={} bbs-prime-extent-parsecs={bbs_prime_extent:.6} maximum-materialized-distance-parsecs={maximum_distance:.6}",
        systems.len(),
        store.bbs_credentials()?.len()
    );
    if list_systems {
        for system in &systems {
            println!(
                "system id={} name={:?} pop={} tl={} starport={} j2-neighbors={}",
                system.system_id,
                system.name,
                system.population,
                system.tech_level,
                starport_code(system.starport),
                system.jump_two_neighbors.len()
            );
        }
    }

    if initialize_only {
        let storage = store.storage_bytes()?;
        let expected_systems = systems.len();
        drop(store);
        let reopened = Store::open_with_map_size(&data, map_size_bytes)?;
        let recovered_systems = reopened.simulation_systems()?.len();
        if recovered_systems != expected_systems {
            return Err(format!(
                "reopened database has {recovered_systems} simulation systems, expected {expected_systems}"
            )
            .into());
        }
        println!("initialization-check=PASS systems={recovered_systems} storage-bytes={storage}");
        return Ok(());
    }

    let before = store.simulation_report()?;
    let storage_before = store.storage_bytes()?;
    let target = before
        .game_second
        .checked_add(
            days.checked_mul(SECONDS_PER_DAY)
                .ok_or("day count overflow")?,
        )
        .ok_or("tour target time overflow")?;
    let advance = store.advance_simulation_toward(target, maximum_events)?;
    let after = store.simulation_report()?;
    let storage_after = store.storage_bytes()?;
    let audited = store.audit_mail_custody()?;

    println!(
        "advanced from second {} toward {} ending {} reached-target={} (requested-days={}); processed-events={} system-days={} departures={} arrivals={} player-travel={}",
        advance.starting_second,
        advance.target_second,
        advance.ending_second,
        advance.reached_target,
        days,
        advance.processed_events,
        advance.system_day_events,
        advance.traffic_departure_events,
        advance.traffic_arrival_events,
        advance.player_travel_events
    );
    let wall_seconds = advance.wall_nanoseconds as f64 / 1_000_000_000.0;
    let cpu_seconds = advance
        .thread_cpu_nanoseconds
        .map(|value| value as f64 / 1_000_000_000.0);
    println!(
        "performance: wall-seconds={wall_seconds:.6} thread-cpu-seconds={} events-per-wall-second={:.3}",
        cpu_seconds
            .map(|value| format!("{value:.6}"))
            .unwrap_or_else(|| "unavailable".into()),
        if wall_seconds == 0.0 {
            0.0
        } else {
            advance.processed_events as f64 / wall_seconds
        }
    );
    println!(
        "observed-ceiling: system-days-per-cpu-second={} universe-days-per-cpu-second={} universe-days-per-wall-second={}",
        format_rate(advance.system_days_per_cpu_second()),
        format_rate(advance.universe_days_per_cpu_second(systems.len())),
        format_rate(advance.universe_days_per_wall_second(systems.len()))
    );
    let mut timings = advance.per_system.iter().collect::<Vec<_>>();
    timings.sort_by_key(|timing| std::cmp::Reverse(timing.thread_cpu_nanoseconds));
    timings.truncate(shown_system_cpu.min(timings.len()));
    for timing in timings {
        let name = systems
            .iter()
            .find(|system| system.system_id == timing.system_id)
            .map(|system| system.name.as_str())
            .unwrap_or("unknown");
        println!(
            "system-cpu id={} name={:?} cpu-ms={:.6} system-days={} departures={} arrivals={} cpu-us-per-system-day={}",
            timing.system_id,
            name,
            timing.thread_cpu_nanoseconds as f64 / 1_000_000.0,
            timing.system_day_events,
            timing.traffic_departure_events,
            timing.traffic_arrival_events,
            if timing.system_day_events == 0 {
                "n/a".into()
            } else {
                format!(
                    "{:.3}",
                    timing.thread_cpu_nanoseconds as f64
                        / 1_000.0
                        / timing.system_day_events as f64
                )
            }
        );
    }
    println!(
        "storage: before-bytes={} after-bytes={} growth-bytes={} growth-bytes-per-processed-system-day={} growth-bytes-per-completed-calendar-day={}",
        storage_before,
        storage_after,
        storage_after.saturating_sub(storage_before),
        if advance.system_day_events == 0 {
            "n/a".into()
        } else {
            format!(
                "{:.3}",
                storage_after.saturating_sub(storage_before) as f64
                    / advance.system_day_events as f64
            )
        },
        if days == 0 || !advance.reached_target {
            "n/a".into()
        } else {
            format!(
                "{:.3}",
                storage_after.saturating_sub(storage_before) as f64 / days as f64
            )
        },
    );
    print_report("cumulative", &after);
    print_delta(&before, &after);
    println!("custody-audit=PASS legs={audited}");
    for leg in store.recent_carrier_legs(shown_legs)? {
        println!(
            "carrier-leg id={} ship={} bag={} {}->{} custody={} due={} delivered={}",
            leg.carrier_leg_id,
            leg.traffic_ship_id,
            leg.mailbag_id,
            leg.origin_system_id,
            leg.destination_system_id,
            leg.custody_second,
            leg.due_second,
            leg.delivered_second
                .map(|second| second.to_string())
                .unwrap_or_else(|| "pending".into())
        );
    }

    drop(store);
    let reopened = Store::open_with_map_size(&data, map_size_bytes)?;
    let recovered = reopened.simulation_report()?;
    if recovered != after || reopened.audit_mail_custody()? != audited {
        return Err("reopened database does not match the committed tour state".into());
    }
    println!("recovery-check=PASS game-second={}", recovered.game_second);
    Ok(())
}

fn required_value(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    arguments
        .next()
        .ok_or_else(|| format!("{option} needs a value").into())
}

fn starport_code(value: u8) -> char {
    match value {
        0 => 'A',
        1 => 'B',
        2 => 'C',
        3 => 'D',
        4 => 'E',
        _ => 'X',
    }
}

fn print_report(label: &str, report: &SimulationReport) {
    println!(
        "{label}: second={} system-days={} messages={} remote-envelopes={} waiting={} in-transit={} delivered={} expired={}",
        report.game_second,
        report.system_days_processed,
        report.message_count(),
        report.remote_envelope_count(),
        report.envelopes_waiting,
        report.envelopes_in_transit,
        report.envelopes_delivered,
        report.envelopes_expired
    );
    for class in MessageClass::ALL {
        println!(
            "{label}: messages.{}={}",
            class.label(),
            report.messages_by_class[class as usize]
        );
    }
    println!(
        "{label}: local-deliveries={} traffic-ships={} ships-in-transit={} mailbags={} bags-delivered={} carrier-legs={} legs-delivered={} scheduled={}",
        report.local_deliveries,
        report.traffic_ships,
        report.traffic_ships_in_transit,
        report.mailbags,
        report.mailbags_delivered,
        report.carrier_legs,
        report.carrier_legs_delivered,
        report.scheduled_events
    );
}

fn print_delta(before: &SimulationReport, after: &SimulationReport) {
    println!(
        "interval: messages={} remote-envelopes={} traffic-ships={} mailbags={} deliveries={} expiries={}",
        after.message_count() - before.message_count(),
        after.remote_envelope_count() - before.remote_envelope_count(),
        after.traffic_ships - before.traffic_ships,
        after.mailbags - before.mailbags,
        after.envelopes_delivered - before.envelopes_delivered,
        after.envelopes_expired - before.envelopes_expired
    );
}

fn format_rate(rate: Option<f64>) -> String {
    rate.map(|value| format!("{value:.6}"))
        .unwrap_or_else(|| "unavailable".into())
}

fn usage(error: Option<&str>) -> ! {
    if let Some(error) = error {
        eprintln!("{error}");
    }
    eprintln!(
        "Usage: cepheus-trader-universe-tour --data DIRECTORY [--days N] [--bbs-count N]\n\
         [--settlement-edge] [--map-size-gib N] [--list-systems]\n\
         [--initialize-only] [--max-events N]\n\
         [--show-system-cpu N|all] [--show-legs N]\n\
         The command initializes an empty database but never resets or deletes an existing one."
    );
    std::process::exit(if error.is_some() { 2 } else { 0 });
}
