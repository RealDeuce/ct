use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cepheus_trader_server::engine::{BbsRegistry, Engine};
use cepheus_trader_server::store::{FlightLegPurpose, FlightLegRecord, ShipLocationRecord, Store};
use cepheus_trader_server::wire::{
    Command as WireCommand, CommandRequest, EncounterFallback, EncounterPosture, EncounterState,
    PlayerIdentity, ResolveEncounterRequest,
};

fn strip_ecma48(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut plain = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == 0x1b && index + 1 < bytes.len() && bytes[index + 1] == b'[' {
            index += 2;
            while index < bytes.len() {
                let byte = bytes[index];
                index += 1;
                if (0x40..=0x7e).contains(&byte) {
                    break;
                }
            }
        } else {
            plain.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(plain).unwrap()
}

fn numeric_field(input: &str, name: &str) -> u64 {
    input
        .lines()
        .find_map(|line| line.strip_prefix(name))
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("missing numeric field {name:?} in {input:?}"))
}

fn spawn_server(
    executable: &Path,
    game_address: &str,
    admin_address: &str,
    sysop_address: &str,
    data: &Path,
) -> ServerProcess {
    let child = Command::new(executable)
        .args([
            "--listen",
            game_address,
            "--admin-listen",
            admin_address,
            "--sysop-listen",
            sysop_address,
            "--data",
            data.to_str().unwrap(),
            "--backup-dir",
            data.join("backups").to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut server = ServerProcess::new(child);
    server.wait_until_listening();
    server
}

fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

struct DoorSession {
    child: Child,
    input: Option<ChildStdin>,
    output: Arc<Mutex<Vec<u8>>>,
    reader: Option<JoinHandle<()>>,
}

enum ClientBuild {
    Existing(PathBuf),
    Temporary(tempfile::TempDir),
}

impl ClientBuild {
    fn path(&self) -> &Path {
        match self {
            Self::Existing(path) => path,
            Self::Temporary(directory) => directory.path(),
        }
    }
}

impl DoorSession {
    fn spawn(door: &Path, working_directory: &Path, profile: &str, columns: &str) -> Self {
        std::fs::write(
            working_directory.join("door.cfg"),
            format!(
                "CTConfig {}\nCTProfile {profile}\nCTColumns {columns}\nCTRows 24\n",
                working_directory.join("cepheus-trader.conf").display()
            ),
        )
        .unwrap();
        let mut child = Command::new(door)
            .current_dir(working_directory)
            .args(["-L", "-USERNAME", "Test Player"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let output = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&output);
        let reader = std::thread::spawn(move || {
            let mut bytes = [0_u8; 4096];
            loop {
                let count = stdout.read(&mut bytes).unwrap();
                if count == 0 {
                    break;
                }
                captured.lock().unwrap().extend_from_slice(&bytes[..count]);
            }
        });
        Self {
            child,
            input: Some(input),
            output,
            reader: Some(reader),
        }
    }

    fn send(&mut self, bytes: &[u8]) {
        let input = self.input.as_mut().unwrap();
        input.write_all(bytes).unwrap();
        input.flush().unwrap();
    }

    fn output(&self) -> String {
        String::from_utf8_lossy(&self.output.lock().unwrap()).into_owned()
    }

    fn wait_for(&self, text: &str) -> String {
        self.wait_for_occurrences(text, 1)
    }

    fn wait_for_occurrences(&self, text: &str, minimum: usize) -> String {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let output = self.output();
            if strip_ecma48(&output).matches(text).count() >= minimum {
                return output;
            }
            assert!(
                Instant::now() < deadline,
                "door did not render occurrence {minimum} of {text:?}; output: {output:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_for_any(&self, texts: &[&str]) -> (usize, String) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let output = self.output();
            let semantic = strip_ecma48(&output);
            if let Some(index) = texts.iter().position(|text| semantic.contains(text)) {
                return (index, output);
            }
            assert!(
                Instant::now() < deadline,
                "door did not render any of {texts:?}; output: {output:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn finish(mut self) -> String {
        drop(self.input.take());
        let status = self.child.wait().unwrap();
        self.reader.take().unwrap().join().unwrap();
        let output = self.output();
        assert!(status.success(), "door failed; output: {output:?}");
        output
    }

    fn terminate(mut self) -> String {
        drop(self.input.take());
        self.child.kill().unwrap();
        self.child.wait().unwrap();
        self.reader.take().unwrap().join().unwrap();
        self.output()
    }
}

fn engine_request(epoch: u64, request_id: u64, command: WireCommand) -> CommandRequest {
    let mut command_id = [0_u8; 16];
    command_id[0..8].copy_from_slice(&request_id.to_be_bytes());
    command_id[8..16].copy_from_slice(&request_id.rotate_left(17).to_be_bytes());
    CommandRequest {
        request_id,
        session_epoch: epoch,
        command_id,
        command,
    }
}

fn settle_arrival_checkpoint(data: &Path, identity: &PlayerIdentity) {
    for attempt in 0_u64..32 {
        let engine = Engine::open(data, BbsRegistry::default()).unwrap();
        let (epoch, _, _) = engine.issue_session(identity).unwrap();
        if let Some(encounter) = engine.pending_encounter(identity).unwrap() {
            if encounter.state == EncounterState::AwaitingPosture {
                engine
                    .submit(
                        identity.clone(),
                        engine_request(
                            epoch,
                            90_000 + attempt * 2,
                            WireCommand::ResolveEncounter(ResolveEncounterRequest {
                                encounter_id: encounter.encounter_id,
                                expected_revision: encounter.revision,
                                posture: EncounterPosture::Comply,
                                fallbacks: vec![EncounterFallback::Surrender],
                            }),
                        ),
                    )
                    .unwrap();
            }
            engine
                .advance_simulation_to(if encounter.next_turn_second == 0 {
                    encounter.started_second + 1_000
                } else {
                    encounter.next_turn_second
                })
                .unwrap();
            continue;
        }
        if let Some(checkpoint) = engine.pending_checkpoint(identity).unwrap() {
            engine
                .submit(
                    identity.clone(),
                    engine_request(
                        epoch,
                        90_001 + attempt * 2,
                        WireCommand::AcknowledgeCheckpoint {
                            checkpoint_id: checkpoint.checkpoint_id,
                        },
                    ),
                )
                .unwrap();
            continue;
        }
        drop(engine);
        let store = Store::open(data).unwrap();
        let player = store.player_record(identity).unwrap().unwrap();
        let ship = store.ship_record(player.ship_id).unwrap().unwrap();
        match ship.location {
            ShipLocationRecord::Docked { .. } => return,
            ShipLocationRecord::InFlight(leg) => {
                let due_second = leg.due_second;
                drop(store);
                Engine::open(data, BbsRegistry::default())
                    .unwrap()
                    .advance_simulation_to(due_second)
                    .unwrap();
            }
            other => panic!("terminal approach entered an unexpected locus: {other:?}"),
        }
    }
    panic!("terminal approach did not settle after encounter and checkpoint processing");
}

fn settle_pending_arrival_encounter(data: &Path, identity: &PlayerIdentity) {
    for attempt in 0_u64..32 {
        let engine = Engine::open(data, BbsRegistry::default()).unwrap();
        let (epoch, _, _) = engine.issue_session(identity).unwrap();
        let Some(encounter) = engine.pending_encounter(identity).unwrap() else {
            return;
        };
        if encounter.state == EncounterState::AwaitingPosture {
            engine
                .submit(
                    identity.clone(),
                    engine_request(
                        epoch,
                        95_000 + attempt,
                        WireCommand::ResolveEncounter(ResolveEncounterRequest {
                            encounter_id: encounter.encounter_id,
                            expected_revision: encounter.revision,
                            posture: EncounterPosture::Comply,
                            fallbacks: vec![EncounterFallback::Surrender],
                        }),
                    ),
                )
                .unwrap();
        }
        engine
            .advance_simulation_to(if encounter.next_turn_second == 0 {
                encounter.started_second + 1_000
            } else {
                encounter.next_turn_second
            })
            .unwrap();
    }
    panic!("arrival encounter did not settle under conservative orders");
}

fn advance_until_flight_leg(
    data: &Path,
    identity: &PlayerIdentity,
    matches_purpose: impl Fn(FlightLegPurpose) -> bool,
) -> FlightLegRecord {
    for _ in 0..32 {
        settle_pending_arrival_encounter(data, identity);
        let store = Store::open(data).unwrap();
        let player = store.player_record(identity).unwrap().unwrap();
        let ship = store.ship_record(player.ship_id).unwrap().unwrap();
        let leg = match ship.location {
            ShipLocationRecord::InFlight(leg) => leg,
            other => panic!("expected an in-flight voyage leg, got {other:?}"),
        };
        if matches_purpose(leg.purpose) {
            return leg;
        }
        drop(store);
        Engine::open(data, BbsRegistry::default())
            .unwrap()
            .advance_simulation_to(leg.due_second)
            .unwrap();
    }
    panic!("voyage did not reach the requested flight-leg purpose");
}

fn latest_arrival_page(output: &str) -> String {
    let semantic = strip_ecma48(output);
    semantic
        .rsplit_once("Arrival Packet -")
        .map(|(_, page)| page.to_owned())
        .unwrap_or(semantic)
}

fn exercise_arrival_profile(door: &Path, data: &Path, profile: &str, columns: &str) -> String {
    let mut session = DoorSession::spawn(door, data, profile, columns);
    session.send(b"\r");

    let mut page_number = 1;
    let mut claimed = false;
    while !claimed {
        let output = session.wait_for_occurrences("Stop review", page_number);
        let page = latest_arrival_page(&output);
        let offer = page.contains("Service:  Offer");
        if offer {
            session.send(b"\r");
            session.wait_for("Claim signed offer");
            session.send(b"a");
            session.wait_for("Offer claimed and entered");
            session.send(b"\r");
            claimed = true;
            page_number += 1;
        } else {
            // Filing a non-offer as actioned is an ordinary arrival-packet
            // operation and advances without relying on terminal arrow-key
            // encoding in the pipe-driven local harness.
            session.send(b"a");
            page_number += 1;
        }
        assert!(
            page_number < 128,
            "arrival packet did not contain an actionable offer; output: {:?}",
            session.output()
        );
    }

    session.wait_for_occurrences("Stop review", page_number);
    session.send(b"q");
    session.wait_for("Arrival Communications Receipt");
    session.send(b"\r");
    let (arrival_result, _) = session.wait_for_any(&[
        "Arrival Checkpoint",
        "Voyage Status -",
        "Docked Operations -",
    ]);
    if arrival_result == 0 {
        session.send(b"a");
        let (watch_result, _) = session.wait_for_any(&["Voyage Status -", "Docked Operations -"]);
        if watch_result == 0 {
            session.send(b"\r");
        } else {
            session.send(b"u");
        }
    } else if arrival_result == 1 {
        session.send(b"\r");
    } else {
        session.send(b"u");
    }
    session.wait_for("Captain's Command Console");
    session.send(b"m");
    const MESSAGE_HEADING: &str = "Message Management\r\n==================";
    session.wait_for(MESSAGE_HEADING);
    session.send(b"i");
    session.wait_for("Message number on this page");
    session.send(b"1");
    session.wait_for_occurrences(MESSAGE_HEADING, 2);
    session.send(b"l");
    session.wait_for_occurrences("Message number on this page", 2);
    session.send(b"2");
    let classified = session.wait_for_occurrences(MESSAGE_HEADING, 3);
    assert!(strip_ecma48(&classified).contains("Ignored"));
    assert!(
        strip_ecma48(&classified).contains("Review"),
        "classification output: {classified:?}"
    );
    session.send(b"3");
    session.wait_for_occurrences("Communications Record", 2);
    session.send(b"\r");
    session.wait_for_occurrences(MESSAGE_HEADING, 4);
    session.send(b"q");
    session.wait_for_occurrences("Captain's Command Console", 2);
    session.send(b"k");
    session.wait_for("Ship's Navigation Library");
    session.send(b"\r");
    session.wait_for_occurrences("Captain's Command Console", 3);
    session.send(b"q");
    session.finish()
}

fn complete_arrival_and_trade(
    door: &Path,
    data: &Path,
    identity: &PlayerIdentity,
    cargo_lot_id: u64,
) -> String {
    let mut session = DoorSession::spawn(door, data, "iso646", "40");
    session.send(b"\r");
    session.wait_for("Arrival Packet -");
    session.send(b"q");
    session.wait_for("Arrival Communications Receipt");
    session.send(b"\r");
    session.wait_for("Docked Operations");

    session.send(b"f");
    session.wait_for("Fuel and Supplies");
    session.send(b"f");
    session.wait_for("Fuel source (Q to cancel):");
    session.send(b"1\r");
    // The 40-column profile may wrap the prompt between "to" and
    // "cancel"; match the semantic prefix shared by every width.
    let (selection, _) = session.wait_for_any(&["Tonnes (Q to", "That service"]);
    if selection == 0 {
        session.send(b"1\r");
        session.wait_for("Ship's stores have been loaded");
        session.send(b"\r");
    } else {
        session.send(b"\r");
    }
    session.wait_for_occurrences("Docked Operations", 2);

    // Exercise the facility-backed provision service when the destination
    // has a chandlery. At a frontier port, prove the same stale/forged key is
    // rejected with in-world copy rather than inventing stock.
    session.send(b"f");
    let supplies = session.wait_for_occurrences("Fuel and Supplies", 2);
    session.send(b"p");
    if strip_ecma48(&supplies).contains("P) Provisions") {
        session.wait_for("Monthly packages (Q to");
        session.send(b"1\r");
        session.wait_for("Ship's stores have been loaded");
        session.send(b"\r");
    } else {
        session.wait_for("No bonded chandlery");
        session.send(b"\r");
    }
    session.wait_for_occurrences("Docked Operations", 3);

    // Arrival processing may add task-titled cargo, and the wire snapshot is
    // the authority for menu order. Identify the purchased lot by its rendered
    // commodity name instead of assuming the stored vector has the same order.
    let cargo_name = {
        let store = Store::open(data).unwrap();
        let player = store.player_record(identity).unwrap().unwrap();
        let commodity_id = store
            .ship_record(player.ship_id)
            .unwrap()
            .unwrap()
            .cargo
            .iter()
            .find(|lot| lot.cargo_lot_id == cargo_lot_id)
            .expect("purchased speculative cargo must still be aboard")
            .commodity_id;
        cepheus_trader_server::commerce::commodity(commodity_id)
            .expect("purchased cargo must use a catalog commodity")
            .name
            .to_owned()
    };
    session.send(b"c");
    // The heading is written before the rest of the page.  Wait for the
    // section delimiter used below so the reader thread has captured the
    // complete cargo list before taking the output snapshot.
    let exchange = session.wait_for("Port research");
    let semantic = strip_ecma48(&exchange);
    let cargo_section = semantic
        .rsplit_once("Cargo Exchange -")
        .and_then(|(_, page)| page.split_once("Cargo aboard"))
        .and_then(|(_, cargo)| cargo.split_once("Port research"))
        .map(|(cargo, _)| cargo)
        .expect("cargo exchange omitted its cargo section");
    let cargo_selection = cargo_section
        .lines()
        .find_map(|line| {
            let (number, description) = line.trim().split_once(". ")?;
            description
                .contains(&cargo_name)
                .then(|| number.parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or_else(|| panic!("cargo menu omitted {cargo_name:?}: {cargo_section:?}"));
    session.send(b"s");
    session.wait_for("Cargo lot (Q to cancel):");
    session.send(format!("{cargo_selection}\r").as_bytes());
    session.wait_for("Tonnes (maximum");
    session.send(b"1\r");
    session.wait_for_occurrences("Cargo Exchange -", 2);
    session.send(b"\r");
    session.wait_for_occurrences("Docked Operations", 3);
    session.send(b"q");
    session.finish()
}

#[test]
fn administrator_sysop_and_player_cpp_clients_interoperate_with_server() {
    let client_source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../client");
    let build = if let Some(path) = std::env::var_os("CT_INTEROP_CLIENT_BUILD") {
        ClientBuild::Existing(PathBuf::from(path))
    } else {
        let directory = tempfile::tempdir().unwrap();
        run(Command::new("cmake")
            .args(["-S"])
            .arg(&client_source)
            .args(["-B"])
            .arg(directory.path())
            .args(["-G", "Ninja"]));
        run(Command::new("cmake")
            .args(["--build"])
            .arg(directory.path())
            .args(["--parallel", "1"]));
        ClientBuild::Temporary(directory)
    };

    let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let game_address = reservation.local_addr().unwrap();
    drop(reservation);
    let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let admin_address = reservation.local_addr().unwrap();
    drop(reservation);
    let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let sysop_address = reservation.local_addr().unwrap();
    drop(reservation);
    let sysop_port = sysop_address.port().to_string();
    let data = tempfile::tempdir().unwrap();
    let admin_psk = data.path().join("admin.psk");
    let server_executable = PathBuf::from(env!("CARGO_BIN_EXE_cepheus-trader-server"));
    let game_address_text = game_address.to_string();
    let admin_address_text = admin_address.to_string();
    let sysop_address_text = sysop_address.to_string();
    let mut server = spawn_server(
        &server_executable,
        &game_address_text,
        &admin_address_text,
        &sysop_address_text,
        data.path(),
    );

    let administrator = build.path().join("cepheus-trader-admin");
    let premature = Command::new(&administrator)
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &admin_address.port().to_string(),
            "--psk-file",
            admin_psk.to_str().unwrap(),
            "--command-id",
            "44444444444444444444444444444444",
            "add-bbs",
            "Premature",
        ])
        .output()
        .unwrap();
    assert!(!premature.status.success());
    assert!(
        String::from_utf8_lossy(&premature.stderr)
            .contains("universe must be initialized before a BBS can be added")
    );
    assert!(!String::from_utf8_lossy(&premature.stderr).contains("retry with --command-id"));
    let mut initial_universe = Command::new(build.path().join("cepheus-trader-admin"))
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &admin_address.port().to_string(),
            "--psk-file",
            admin_psk.to_str().unwrap(),
            "--command-id",
            "88888888888888888888888888888888",
            "initialize-universe",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(
        initial_universe.stdin.as_mut().unwrap(),
        "INITIALIZE FEDERATION"
    )
    .unwrap();
    let initial_universe = initial_universe.wait_with_output().unwrap();
    assert!(
        initial_universe.status.success(),
        "initial universe failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&initial_universe.stdout),
        String::from_utf8_lossy(&initial_universe.stderr)
    );

    let output = Command::new(&administrator)
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &admin_address.port().to_string(),
            "--psk-file",
            admin_psk.to_str().unwrap(),
            "--command-id",
            "55555555555555555555555555555555",
            "add-bbs",
            "Dark Star",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "administrator failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let administrator_output = String::from_utf8(output.stdout).unwrap();
    assert!(administrator_output.starts_with("BBS id=1 committed="));
    assert!(administrator_output.contains(" psk="));
    let bbs_committed = administrator_output
        .split(" committed=")
        .nth(1)
        .and_then(|tail| tail.split(' ').next())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap();
    let retry = Command::new(build.path().join("cepheus-trader-admin"))
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &admin_address.port().to_string(),
            "--psk-file",
            admin_psk.to_str().unwrap(),
            "--command-id",
            "55555555555555555555555555555555",
            "add-bbs",
            "Dark Star",
        ])
        .output()
        .unwrap();
    assert!(retry.status.success());
    assert_eq!(
        String::from_utf8(retry.stdout).unwrap(),
        administrator_output
    );
    let psk = administrator_output
        .trim()
        .split("psk=")
        .nth(1)
        .unwrap()
        .to_owned();

    let bbs_config = data.path().join("cepheus-trader.conf");
    let credential_file = data.path().join("cepheus-trader.credential");
    let mut credential_creator = Command::new(build.path().join("cepheus-trader-sysop"))
        .args(["--config", bbs_config.to_str().unwrap(), "init-credential"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let input = credential_creator.stdin.as_mut().unwrap();
        writeln!(input, "1").unwrap();
        writeln!(input, "{psk}").unwrap();
    }
    let credential_output = credential_creator.wait_with_output().unwrap();
    assert!(
        credential_output.status.success(),
        "credential creation failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&credential_output.stdout),
        String::from_utf8_lossy(&credential_output.stderr)
    );
    let credential_stdout = String::from_utf8_lossy(&credential_output.stdout);
    assert!(credential_stdout.contains(&format!(
        "installation configuration created: {}",
        bbs_config.display()
    )));
    assert!(credential_stdout.contains(&format!(
        "credential created for BBS id=1: {}",
        credential_file.display()
    )));
    assert!(credential_stdout.contains(&format!(
        "next: cepheus-trader-sysop --config {} set-config",
        bbs_config.display()
    )));
    let bootstrap_config = std::fs::read_to_string(&bbs_config).unwrap();
    assert!(bootstrap_config.contains("credential-file=cepheus-trader.credential\n"));
    assert_eq!(std::fs::metadata(&credential_file).unwrap().len(), 48);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&credential_file)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let mut overwrite = Command::new(build.path().join("cepheus-trader-sysop"))
        .args(["--config", bbs_config.to_str().unwrap(), "init-credential"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let input = overwrite.stdin.as_mut().unwrap();
        writeln!(input, "1").unwrap();
        writeln!(input, "{psk}").unwrap();
    }
    let overwrite_output = overwrite.wait_with_output().unwrap();
    assert!(!overwrite_output.status.success());

    std::fs::write(
        &bbs_config,
        format!(
            "# Shared by the sysop utility and player door.\n\
             server=127.0.0.1\n\
             game-port={}\n\
             sysop-port={}\n\
             credential-file=cepheus-trader.credential\n\
             identity-file=cepheus-trader.identities\n\
             identity-name=real-name\n\
             terminal-profile=auto\n\
             terminal-columns=0\n\
             terminal-rows=0\n",
            game_address.port(),
            sysop_port,
        ),
    )
    .unwrap();

    let sysop = build.path().join("cepheus-trader-sysop");
    let unconfigured = Command::new(&sysop)
        .current_dir(data.path())
        .arg("get-config")
        .output()
        .unwrap();
    assert!(
        unconfigured.status.success(),
        "sysop get failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&unconfigured.stdout),
        String::from_utf8_lossy(&unconfigured.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&unconfigured.stdout),
        format!(
            "bbs-id=1\nrevision=0\ncommitted={bbs_committed}\nconfigured=no\n\
         bbs-name=Dark Star\npolity-name=\ntrade-combat=50\nchaos-order=50\n"
        )
    );
    let set_arguments = [
        "--expected-revision",
        "0",
        "--command-id",
        "66666666666666666666666666666666",
        "set-config",
        "Dark Star BBS",
        "Far Reach",
        "65",
        "25",
    ];
    let configured = Command::new(&sysop)
        .current_dir(data.path())
        .args(set_arguments)
        .output()
        .unwrap();
    assert!(
        configured.status.success(),
        "sysop set failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&configured.stdout),
        String::from_utf8_lossy(&configured.stderr)
    );
    let configured_text = String::from_utf8_lossy(&configured.stdout);
    assert!(configured_text.starts_with("bbs-id=1\nrevision=1\ncommitted="));
    assert!(configured_text.contains("\nconfigured=yes\n"));
    assert!(configured_text.contains("\nbbs-name=Dark Star BBS\n"));
    assert!(configured_text.contains("\npolity-name=Far Reach\n"));
    assert!(configured_text.ends_with("trade-combat=65\nchaos-order=25\n"));
    let configured_committed = numeric_field(&configured_text, "committed=");
    assert!(configured_committed > bbs_committed);
    let retry_configuration = Command::new(&sysop)
        .current_dir(data.path())
        .args(set_arguments)
        .output()
        .unwrap();
    assert!(retry_configuration.status.success());
    assert_eq!(retry_configuration.stdout, configured.stdout);
    let stale = Command::new(&sysop)
        .current_dir(data.path())
        .args([
            "--expected-revision",
            "0",
            "--command-id",
            "77777777777777777777777777777777",
            "set-config",
            "Dark Star BBS",
            "Far Reach",
            "65",
            "25",
        ])
        .output()
        .unwrap();
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("current revision is 1"));
    let read_back = Command::new(&sysop)
        .current_dir(data.path())
        .arg("get-config")
        .output()
        .unwrap();
    assert!(read_back.status.success());
    assert_eq!(read_back.stdout, configured.stdout);
    let automatic_sysop_update = Command::new(&sysop)
        .current_dir(data.path())
        .args(["set-config", "Dark Star BBS", "Far Reach", "65", "25"])
        .output()
        .unwrap();
    assert!(
        automatic_sysop_update.status.success(),
        "automatic sysop update failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&automatic_sysop_update.stdout),
        String::from_utf8_lossy(&automatic_sysop_update.stderr)
    );
    let automatic_text = String::from_utf8_lossy(&automatic_sysop_update.stdout);
    assert!(automatic_text.starts_with("bbs-id=1\nrevision=2\ncommitted="));
    assert!(automatic_text.contains("\nconfigured=yes\n"));
    assert!(automatic_text.contains("\nbbs-name=Dark Star BBS\n"));
    assert!(automatic_text.contains("\npolity-name=Far Reach\n"));
    assert!(automatic_text.ends_with("trade-combat=65\nchaos-order=25\n"));
    let automatic_committed = numeric_field(&automatic_text, "committed=");
    assert!(automatic_committed > configured_committed);
    let client = build.path().join("cepheus-trader-client");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&credential_file, std::fs::Permissions::from_mode(0o644)).unwrap();
        let insecure = Command::new(&client)
            .args([
                "127.0.0.1",
                &game_address.port().to_string(),
                credential_file.to_str().unwrap(),
                "1",
            ])
            .output()
            .unwrap();
        assert!(!insecure.status.success());
        assert!(
            String::from_utf8_lossy(&insecure.stderr)
                .contains("must not grant group or other permissions")
        );
        std::fs::set_permissions(&credential_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let door = build.path().join("cepheus-trader-door");
    for (profile, columns) in [
        ("iso646", "40"),
        ("iso646-color", "80"),
        ("cp437-color", "40"),
    ] {
        let mut session = DoorSession::spawn(&door, data.path(), profile, columns);
        session.send(b"\r");
        session.wait_for("No captain is registered");
        session.send(b"q");
        let screen = session.finish();
        let semantic_screen = strip_ecma48(&screen);
        assert!(screen.contains("An Alternate Cepheus Engine"), "{screen:?}");
        assert!(screen.contains("Universe"), "{screen:?}");
        assert!(
            screen.contains("Open Game License version 1.0a"),
            "{screen:?}"
        );
        assert!(
            semantic_screen.contains("Secure communications link"),
            "{screen:?}"
        );
        assert!(
            semantic_screen.contains("Ship status: Unregistered"),
            "{screen:?}"
        );
        assert!(
            semantic_screen.contains("No captain is registered"),
            "{screen:?}"
        );
        if profile == "iso646" {
            assert!(screen.contains('\u{c}'), "{screen:?}");
            assert!(!screen.contains('\u{1b}'), "{screen:?}");
        } else {
            assert!(screen.contains("\u{1b}[0m"), "{screen:?}");
            assert!(screen.contains("\u{1b}[2J\u{1b}[H"), "{screen:?}");
            assert!(
                screen.contains("\u{1b}[36m  Ship status: \u{1b}[0m"),
                "{screen:?}"
            );
            assert!(
                screen.contains("\u{1b}[1;35mUnregistered\r\n\u{1b}[0m"),
                "{screen:?}"
            );
        }
        if profile == "cp437-color" {
            assert!(
                screen.contains("── An Alternate Cepheus Engine"),
                "{screen:?}"
            );
            assert!(screen.contains("Universe ──"), "{screen:?}");
        }
    }
    let output = Command::new(client)
        .args([
            "127.0.0.1",
            &game_address.port().to_string(),
            credential_file.to_str().unwrap(),
            "1",
            "create-player",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "Alex Mercer",
            "Far Horizon",
            "Samir",
            "Morgan",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "client failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let creation_output = String::from_utf8_lossy(&output.stdout);
    let mut lines = creation_output.trim().lines();
    let hello_line = lines.next().expect("client omitted ServerHello output");
    assert!(
        hello_line.starts_with("HELLO bbs=1 player=1 epoch=4 committed=")
            && hello_line.ends_with(" phase=new-user tls=TLS1.3"),
        "{hello_line}"
    );
    let hello_committed = hello_line
        .split(" committed=")
        .nth(1)
        .and_then(|tail| tail.split(' ').next())
        .and_then(|value| value.parse::<u64>().ok())
        .expect("ServerHello committed sequence is not numeric");
    let created = lines.next().expect("client omitted PlayerCreated output");
    assert!(
        created.starts_with("CREATED captain=Alex Mercer ship=Far Horizon crew=4 committed=")
            && created.ends_with(" phase=docked"),
        "{created}"
    );
    let committed = created
        .split(" committed=")
        .nth(1)
        .and_then(|tail| tail.split(' ').next())
        .and_then(|value| value.parse::<u64>().ok())
        .expect("PlayerCreated committed sequence is not numeric");
    assert!(committed > hello_committed);
    assert!(lines.next().is_none());

    let status = Command::new(&administrator)
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &admin_address.port().to_string(),
            "--psk-file",
            admin_psk.to_str().unwrap(),
            "status",
        ])
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "status failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    let status_text = String::from_utf8_lossy(&status.stdout);
    assert!(status_text.contains("bbs-count=1\n"), "{status_text}");
    assert!(status_text.contains("player-count=1\n"), "{status_text}");
    assert!(status_text.contains("storage-format=1\n"), "{status_text}");

    let admin_port = admin_address.port().to_string();
    let backup_arguments = [
        "--host",
        "127.0.0.1",
        "--port",
        admin_port.as_str(),
        "--psk-file",
        admin_psk.to_str().unwrap(),
        "--command-id",
        "abababababababababababababababab",
        "live-backup",
        "interop",
    ];
    let backup = Command::new(&administrator)
        .args(backup_arguments)
        .output()
        .unwrap();
    assert!(
        backup.status.success(),
        "backup failed: {}",
        String::from_utf8_lossy(&backup.stderr)
    );
    assert!(data.path().join("backups/interop/data.mdb").is_file());
    assert!(data.path().join("backups/interop/manifest.txt").is_file());
    let backup_retry = Command::new(&administrator)
        .args(backup_arguments)
        .output()
        .unwrap();
    assert!(backup_retry.status.success());
    assert_eq!(backup_retry.stdout, backup.stdout);

    let suspended = Command::new(&sysop)
        .current_dir(data.path())
        .args(["suspend-player", "1", "field review"])
        .output()
        .unwrap();
    assert!(suspended.status.success());
    assert!(String::from_utf8_lossy(&suspended.stdout).contains("state=suspended\n"));
    let denied = Command::new(build.path().join("cepheus-trader-client"))
        .args([
            "127.0.0.1",
            &game_address.port().to_string(),
            credential_file.to_str().unwrap(),
            "1",
        ])
        .output()
        .unwrap();
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("account suspended"));
    let resumed = Command::new(&sysop)
        .current_dir(data.path())
        .args(["resume-player", "1", "field review complete"])
        .output()
        .unwrap();
    assert!(resumed.status.success());
    assert!(String::from_utf8_lossy(&resumed.stdout).contains("state=active\n"));

    let reconnect = Command::new(build.path().join("cepheus-trader-client"))
        .args([
            "127.0.0.1",
            &game_address.port().to_string(),
            credential_file.to_str().unwrap(),
            "1",
        ])
        .output()
        .unwrap();
    let mut docked_door = DoorSession::spawn(&door, data.path(), "iso646", "40");
    docked_door.send(b"\rq\r");
    std::thread::sleep(std::time::Duration::from_secs(1));
    docked_door.send(b"ji1\r\ra1\r\rqq");
    let docked_screen = docked_door.finish();
    for expected in [
        "Docked Operations",
        "Cargo Exchange",
        "Jobs and Passage",
        "Fuel and Supplies",
        "Shipyard",
        "Personnel",
        "Banking and Accounts",
        "Authorities",
        "Depart",
        "Universal",
        "Signed Offer Instrument",
        "Failure charge:",
        "entered in the task ledger.",
        "Accepted obligations",
    ] {
        assert!(docked_screen.contains(expected), "{docked_screen:?}");
    }
    let mut reconnected_door = DoorSession::spawn(&door, data.path(), "iso646", "40");
    reconnected_door.send(b"\rjqq");
    let reconnect_screen = reconnected_door.finish();
    assert!(reconnect_screen.contains("Accepted obligations"));
    assert!(reconnect_screen.contains("No.1 "));
    assert!(reconnect_screen.contains("Reserved:"));

    // Exercise the Milestone 5 account and correspondence controls through
    // the real door rather than merely proving their wire codecs.
    let mut services_door = DoorSession::spawn(&door, data.path(), "iso646", "40");
    services_door.send(b"\r");
    services_door.wait_for("Docked Operations");
    services_door.send(b"b");
    services_door.wait_for("Banking and Accounts");
    services_door.send(b"b");
    services_door.wait_for_occurrences("Destination aid:", 2);
    services_door.wait_for("Day 365");
    services_door.send(b"\r");
    services_door.wait_for_occurrences("Docked Operations", 2);
    services_door.send(b"u");
    services_door.wait_for("Captain's Command Console");
    services_door.send(b"m");
    services_door.wait_for("Message Management");
    services_door.send(b"c");
    services_door.wait_for("Recipient:");
    services_door.send(b"c");
    services_door.wait_for("BBS number");
    services_door.send(b"1\r");
    services_door.wait_for("Captain number");
    services_door.send(b"1\r");
    services_door.wait_for("TTL in weeks");
    services_door.send(b"\r");
    services_door.wait_for("Subject");
    services_door.send(b"Loading note\r");
    services_door.wait_for("Message");
    services_door.send(b"Retain one ton for bonded freight.\r");
    services_door.wait_for("accepted for physical");
    services_door.send(b"\r");
    services_door.wait_for_occurrences("Message Management", 2);
    services_door.send(b"q");
    services_door.wait_for_occurrences("Captain's Command Console", 2);
    services_door.send(b"q");
    services_door.wait_for_occurrences("Docked Operations", 3);
    services_door.send(b"q");
    let services_screen = services_door.finish();
    for expected in [
        "Banking and Accounts",
        "Covered through",
        "Message Management",
        "accepted for physical",
    ] {
        assert!(services_screen.contains(expected), "{services_screen:?}");
    }

    // Prove the Milestone 6 player boundary through the real TLS/OpenDoors
    // client without leaving the canonical merchant voyage in combat. The
    // clone starts from the same authenticated, created captain and is thrown
    // away after the assertions.
    server.stop();
    let combat_root = tempfile::tempdir().unwrap();
    copy_directory(data.path(), combat_root.path());
    {
        let identity = PlayerIdentity {
            bbs_id: 1,
            player_id: 1,
        };
        let current_second = Store::open(combat_root.path())
            .unwrap()
            .simulation_report()
            .unwrap()
            .game_second;
        let engine = Engine::open(combat_root.path(), BbsRegistry::default()).unwrap();
        let (epoch, _, _) = engine.issue_session(&identity).unwrap();
        let mut found = false;
        for hour in 0_u64..=14 * 24 {
            let batch = engine
                .submit(
                    identity.clone(),
                    engine_request(epoch, 88_000 + hour, WireCommand::GetCombatCareer),
                )
                .unwrap();
            found = batch.deliveries.iter().any(|delivery| {
                matches!(
                    &delivery.outcome.kind,
                    cepheus_trader_server::wire::OutcomeKind::CombatCareer(snapshot)
                        if !snapshot.local_contacts.is_empty()
                )
            });
            if found {
                break;
            }
            engine
                .advance_simulation_to(current_second.saturating_add((hour + 1) * 3_600))
                .unwrap();
        }
        assert!(
            found,
            "no local traffic window was found within fourteen days"
        );
    }
    let mut combat_server = spawn_server(
        &server_executable,
        &game_address_text,
        &admin_address_text,
        &sysop_address_text,
        combat_root.path(),
    );
    let mut combat_door = DoorSession::spawn(&door, combat_root.path(), "iso646", "40");
    combat_door.send(b"\r");
    combat_door.wait_for("Docked Operations");
    combat_door.send(b"u");
    combat_door.wait_for("Captain's Command Console");
    combat_door.send(b"o");
    combat_door.wait_for("Operations Ledger");
    combat_door.send(b"m");
    combat_door.wait_for("Naval service");
    combat_door.send(b"r");
    combat_door.wait_for_occurrences("Operations Ledger", 2);
    combat_door.wait_for("Service: Pirate");
    combat_door.send(b"i");
    combat_door.wait_for("Contact (Q to cancel):");
    combat_door.send(b"1\r");
    combat_door.wait_for("irreversible act");
    combat_door.send(b"i");
    combat_door.wait_for("Vessel Combat");
    combat_door.wait_for("General quarters");
    combat_door.send(b"d");
    combat_door.wait_for("Joint orders sealed for this activation");
    let combat_screen = combat_door.terminate();
    combat_server.stop();
    for expected in [
        "Operations Ledger",
        "Service: Pirate",
        "Vessel Combat",
        "General quarters",
        "Joint orders sealed for this activation",
    ] {
        assert!(combat_screen.contains(expected), "{combat_screen:?}");
    }
    {
        let engine = Engine::open(combat_root.path(), BbsRegistry::default()).unwrap();
        let identity = PlayerIdentity {
            bbs_id: 1,
            player_id: 1,
        };
        let (epoch, _, _) = engine.issue_session(&identity).unwrap();
        let career_batch = engine
            .submit(
                identity.clone(),
                engine_request(epoch, 89_000, WireCommand::GetCombatCareer),
            )
            .unwrap();
        let career = career_batch
            .deliveries
            .iter()
            .find_map(|delivery| match &delivery.outcome.kind {
                cepheus_trader_server::wire::OutcomeKind::CombatCareer(snapshot) => Some(snapshot),
                _ => None,
            })
            .expect("the career observation must be delivered");
        assert_eq!(
            career.state.mode,
            cepheus_trader_server::careers::CombatCareerMode::Pirate
        );
        assert!(!career.state.warrants.is_empty());
        let combat_batch = engine
            .submit(
                identity,
                engine_request(epoch, 89_001, WireCommand::GetCombat),
            )
            .unwrap();
        let combat = combat_batch
            .deliveries
            .iter()
            .find_map(|delivery| match &delivery.outcome.kind {
                cepheus_trader_server::wire::OutcomeKind::Combat(snapshot) => Some(snapshot),
                _ => None,
            })
            .expect("the combat observation must be delivered");
        assert!(combat.player_order_submitted);
        assert!(!combat.actors.is_empty());
    }
    server = spawn_server(
        &server_executable,
        &game_address_text,
        &admin_address_text,
        &sysop_address_text,
        data.path(),
    );

    // Exercise the merchant side through the real door: buy one whole ton of
    // speculative cargo and file a direct flight plan. The starting tanks are
    // full, so destination fuel service is exercised after the Jump has
    // consumed fuel. Low-capability ports may legitimately sell neither
    // refined nor bulk unrefined fuel; Milestone 3 owns frontier alternatives.
    let mut voyage_door = DoorSession::spawn(&door, data.path(), "iso646", "40");
    voyage_door.send(b"\r");
    voyage_door.wait_for("Docked Operations");
    voyage_door.send(b"c");
    voyage_door.wait_for("Cargo Exchange -");
    voyage_door.send(b"b");
    voyage_door.wait_for("Offer (Q to cancel):");
    voyage_door.send(b"1\r");
    voyage_door.wait_for("Tonnes (maximum");
    voyage_door.send(b"1\r");
    voyage_door.wait_for_occurrences("Cargo Exchange -", 2);
    voyage_door.send(b"\r");
    voyage_door.wait_for_occurrences("Docked Operations", 2);
    voyage_door.send(b"d");
    voyage_door.wait_for("Flight Plan\r\n===========");
    voyage_door.send(b"a");
    voyage_door.wait_for("Destination (Q to cancel):");
    voyage_door.send(b"1\r");
    voyage_door.wait_for("Buy fresh course tape");
    voyage_door.send(b"o");
    voyage_door.wait_for("identifies a bad plot");
    voyage_door.send(b"r");
    voyage_door.wait_for_occurrences("Flight Plan\r\n===========", 2);
    voyage_door.send(b"p");
    voyage_door.wait_for("Flight Plan Preview");
    voyage_door.send(b"f");
    voyage_door.wait_for("Departure authorized.");
    voyage_door.send(b"\r");
    voyage_door.wait_for("Voyage Status -");
    voyage_door.send(b"\r");
    voyage_door.wait_for("Captain's Command Console");
    voyage_door.send(b"q");
    let voyage_screen = voyage_door.finish();
    for expected in [
        "Cargo Exchange -",
        "Cargo aboard",
        "Departure authorized.",
        "The flight plan has been filed.",
    ] {
        assert!(voyage_screen.contains(expected), "{voyage_screen:?}");
    }

    // LMDB has one authoritative writer at a time. Stop the network server
    // before inspecting and accelerating the scheduled-work queue.
    server.stop();

    let identity = PlayerIdentity {
        bbs_id: 1,
        player_id: 1,
    };
    let (ship_id, cargo_lot_id, credits_before_mail, departure_due) = {
        let store = Store::open(data.path()).unwrap();
        let player = store.player_record(&identity).unwrap().unwrap();
        let ship = store.ship_record(player.ship_id).unwrap().unwrap();
        assert!(!ship.cargo.is_empty());
        let departure_due = match ship.location {
            ShipLocationRecord::InFlight(leg) => leg.due_second,
            other => panic!("expected filed departure, got {other:?}"),
        };
        (
            ship.ship_id,
            ship.cargo
                .iter()
                .find(|lot| {
                    lot.title == cepheus_trader_server::wire::CargoTitle::PlayerOwned
                        && lot.purchase_price_per_ton != 0
                })
                .expect("door purchase must create a player-owned speculative lot")
                .cargo_lot_id,
            player.credits,
            departure_due,
        )
    };

    let (pickup, jump_due) = {
        Engine::open(data.path(), BbsRegistry::default())
            .unwrap()
            .advance_simulation_to(departure_due)
            .unwrap();
        let jump_leg = advance_until_flight_leg(data.path(), &identity, |purpose| {
            matches!(purpose, FlightLegPurpose::Jump { .. })
        });
        let store = Store::open(data.path()).unwrap();
        let ship = store.ship_record(ship_id).unwrap().unwrap();
        assert!(
            ship.cargo
                .iter()
                .any(|lot| lot.cargo_lot_id == cargo_lot_id)
        );
        let custody = ship.mail_custody.clone();
        if let Some(custody) = &custody {
            assert!(custody.envelope_count > 0);
        }
        assert_eq!(
            store.audit_mail_custody().unwrap(),
            store.simulation_report().unwrap().carrier_legs
        );
        (custody, jump_leg.due_second)
    };

    // Closing and reopening the authoritative store must retain the exact
    // custody state. An empty route queue deliberately produces no bag.
    {
        let store = Store::open(data.path()).unwrap();
        let ship = store.ship_record(ship_id).unwrap().unwrap();
        assert_eq!(ship.mail_custody, pickup);
    }

    let stipend = pickup
        .as_ref()
        .map_or(0, |custody| custody.advertised_stipend_credits);

    let (arrival_report, arrival_credits, arrival_fuel, arrival_provisions) = {
        Engine::open(data.path(), BbsRegistry::default())
            .unwrap()
            .advance_simulation_to(jump_due)
            .unwrap();
        advance_until_flight_leg(data.path(), &identity, |purpose| {
            matches!(purpose, FlightLegPurpose::ApproachPort)
        });

        let store = Store::open(data.path()).unwrap();
        let player = store.player_record(&identity).unwrap().unwrap();
        let ship = store.ship_record(ship_id).unwrap().unwrap();
        assert!(ship.mail_custody.is_none());
        assert!(
            ship.cargo
                .iter()
                .any(|lot| lot.cargo_lot_id == cargo_lot_id)
        );
        assert_eq!(player.credits, credits_before_mail + stipend);
        (
            store.simulation_report().unwrap(),
            player.credits,
            ship.current_fuel_millitons,
            ship.provisions.person_days_remaining,
        )
    };

    // Reopening and advancing to the same logical second is a no-op: neither
    // stipend nor mail visibility/custody may be duplicated.
    {
        let engine = Engine::open(data.path(), BbsRegistry::default()).unwrap();
        let replay = engine.advance_simulation_to(jump_due).unwrap();
        assert_eq!(replay.processed_events, 0);
        drop(engine);
        let store = Store::open(data.path()).unwrap();
        assert_eq!(
            store.player_record(&identity).unwrap().unwrap().credits,
            arrival_credits
        );
        assert_eq!(store.simulation_report().unwrap(), arrival_report);
        assert!(
            store
                .ship_record(ship_id)
                .unwrap()
                .unwrap()
                .mail_custody
                .is_none()
        );
    }

    // Save an unpresented arrival state for each terminal profile. Each copy
    // uses the real TLS server and OpenDoors executable, while preserving the
    // canonical voyage for the final docking/fuel/sale continuation.
    let profile_root = tempfile::tempdir().unwrap();
    let profile_cases = [
        ("iso646", "40"),
        ("iso646-color", "80"),
        ("cp437-color", "40"),
    ];
    for (index, (profile, columns)) in profile_cases.into_iter().enumerate() {
        let profile_data = profile_root.path().join(index.to_string());
        copy_directory(data.path(), &profile_data);
        let mut profile_server = spawn_server(
            &server_executable,
            &game_address_text,
            &admin_address_text,
            &sysop_address_text,
            &profile_data,
        );
        let profile_screen = exercise_arrival_profile(&door, &profile_data, profile, columns);
        let semantic = strip_ecma48(&profile_screen);
        for expected in [
            "Arrival Packet -",
            "Communications Record",
            "Offer claimed and entered",
            "Arrival Communications Receipt",
            "Message Management",
            "Review",
            "Ship's Navigation Library",
        ] {
            assert!(semantic.contains(expected), "{profile}: {profile_screen:?}");
        }
        if profile == "iso646" {
            assert!(profile_screen.contains('\u{c}'));
            assert!(!profile_screen.contains('\u{1b}'));
        } else {
            assert!(profile_screen.contains("\u{1b}[2J\u{1b}[H"));
            assert!(profile_screen.contains("\u{1b}[1;35m"));
        }
        profile_server.stop();
    }

    // Complete the remaining checkpoint and physical approach before
    // returning to the real server.
    settle_arrival_checkpoint(data.path(), &identity);
    server = spawn_server(
        &server_executable,
        &game_address_text,
        &admin_address_text,
        &sysop_address_text,
        data.path(),
    );
    let completed_screen = complete_arrival_and_trade(&door, data.path(), &identity, cargo_lot_id);
    for expected in [
        "Arrival Packet -",
        "Arrival Communications Receipt",
        "Docked Operations",
        "Fuel and Supplies",
        "Cargo Exchange -",
    ] {
        assert!(completed_screen.contains(expected), "{completed_screen:?}");
    }
    server.stop();
    {
        let store = Store::open(data.path()).unwrap();
        let player = store.player_record(&identity).unwrap().unwrap();
        let ship = store.ship_record(ship_id).unwrap().unwrap();
        assert!(
            ship.cargo
                .iter()
                .all(|lot| lot.cargo_lot_id != cargo_lot_id)
        );
        assert!(ship.mail_custody.is_none());
        if completed_screen.contains("That service") {
            assert_eq!(ship.current_fuel_millitons, arrival_fuel);
        } else {
            assert_eq!(ship.current_fuel_millitons, arrival_fuel + 1_000);
        }
        if !completed_screen.contains("No bonded chandlery") {
            // A monthly package may refill exactly to the pre-approach level
            // when the ship was already close to its stores capacity.
            assert!(ship.provisions.person_days_remaining >= arrival_provisions);
        }
        assert_ne!(player.credits, arrival_credits);
    }
    server = spawn_server(
        &server_executable,
        &game_address_text,
        &admin_address_text,
        &sysop_address_text,
        data.path(),
    );

    let automatic_command_id = Command::new(build.path().join("cepheus-trader-admin"))
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &admin_address.port().to_string(),
            "--psk-file",
            admin_psk.to_str().unwrap(),
            "add-bbs",
            "Automatic ID",
        ])
        .output()
        .unwrap();
    let mut cancelled_initialization = Command::new(build.path().join("cepheus-trader-admin"))
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &admin_address.port().to_string(),
            "--psk-file",
            admin_psk.to_str().unwrap(),
            "initialize-universe",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(
        cancelled_initialization.stdin.as_mut().unwrap(),
        "do not initialize"
    )
    .unwrap();
    let cancelled_initialization = cancelled_initialization.wait_with_output().unwrap();
    let mut universe_initializer = Command::new(build.path().join("cepheus-trader-admin"))
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &admin_address.port().to_string(),
            "--psk-file",
            admin_psk.to_str().unwrap(),
            "initialize-universe",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(
        universe_initializer.stdin.as_mut().unwrap(),
        "INITIALIZE FEDERATION"
    )
    .unwrap();
    let universe_initializer = universe_initializer.wait_with_output().unwrap();
    let post_reset = Command::new(build.path().join("cepheus-trader-client"))
        .args([
            "127.0.0.1",
            &game_address.port().to_string(),
            credential_file.to_str().unwrap(),
            "1",
        ])
        .output()
        .unwrap();
    let post_reset_configuration = Command::new(&sysop)
        .current_dir(data.path())
        .arg("get-config")
        .output()
        .unwrap();
    let server_log = server.stop();
    assert!(
        reconnect.status.success(),
        "reconnect failed\nstdout: {}\nstderr: {}\nserver: {server_log}",
        String::from_utf8_lossy(&reconnect.stdout),
        String::from_utf8_lossy(&reconnect.stderr)
    );
    let reconnect_output = String::from_utf8_lossy(&reconnect.stdout);
    let reconnect_line = reconnect_output.trim();
    assert!(
        reconnect_line.starts_with("HELLO bbs=1 player=1 epoch=5 committed=")
            && reconnect_line.ends_with(" phase=docked tls=TLS1.3"),
        "{reconnect_line}"
    );
    let reconnect_committed = reconnect_line
        .split(" committed=")
        .nth(1)
        .and_then(|tail| tail.split(' ').next())
        .and_then(|value| value.parse::<u64>().ok())
        .expect("reconnect committed sequence is not numeric");
    assert!(reconnect_committed >= committed);
    assert!(
        automatic_command_id.status.success(),
        "automatic administrator command failed\nstdout: {}\nstderr: {}\n\
         server: {server_log}",
        String::from_utf8_lossy(&automatic_command_id.stdout),
        String::from_utf8_lossy(&automatic_command_id.stderr)
    );
    let add_output = String::from_utf8_lossy(&automatic_command_id.stdout);
    assert!(
        add_output.starts_with("BBS id=2 committed=") && add_output.contains(" psk="),
        "{add_output}"
    );
    assert!(!cancelled_initialization.status.success());
    assert!(
        String::from_utf8_lossy(&cancelled_initialization.stderr)
            .contains("universe initialization cancelled")
    );
    assert!(
        universe_initializer.status.success(),
        "universe initialization failed\nstdout: {}\nstderr: {}\nserver: {server_log}",
        String::from_utf8_lossy(&universe_initializer.stdout),
        String::from_utf8_lossy(&universe_initializer.stderr)
    );
    let initialization_output = String::from_utf8_lossy(&universe_initializer.stdout);
    assert!(initialization_output.starts_with("universe-id="));
    assert!(initialization_output.contains(" polities=2 systems=46 worlds=46\n"));
    let reset_committed = initialization_output
        .split(" committed=")
        .nth(1)
        .and_then(|tail| tail.split(' ').next())
        .and_then(|value| value.parse::<u64>().ok())
        .expect("universe reset committed sequence is not numeric");
    assert!(
        post_reset.status.success(),
        "post-reset client failed\nstdout: {}\nstderr: {}\nserver: {server_log}",
        String::from_utf8_lossy(&post_reset.stdout),
        String::from_utf8_lossy(&post_reset.stderr)
    );
    let post_reset_hello = String::from_utf8_lossy(&post_reset.stdout);
    let post_reset_hello = post_reset_hello.trim();
    assert!(
        post_reset_hello.starts_with("HELLO bbs=1 player=1 epoch=")
            && post_reset_hello.ends_with(" phase=new-user tls=TLS1.3"),
        "{post_reset_hello}"
    );
    let post_reset_committed = post_reset_hello
        .split(" committed=")
        .nth(1)
        .and_then(|tail| tail.split(' ').next())
        .and_then(|value| value.parse::<u64>().ok())
        .expect("post-reset committed sequence is not numeric");
    assert!(post_reset_committed >= reset_committed);
    assert!(post_reset_configuration.status.success());
    assert_eq!(
        String::from_utf8_lossy(&post_reset_configuration.stdout),
        format!(
            "bbs-id=1\nrevision=2\ncommitted={automatic_committed}\nconfigured=yes\n\
         bbs-name=Dark Star BBS\npolity-name=Far Reach\n\
         trade-combat=65\nchaos-order=25\n"
        )
    );
}

struct ServerProcess {
    child: Option<Child>,
    stderr: Option<BufReader<ChildStderr>>,
    log: String,
}

impl ServerProcess {
    fn new(mut child: Child) -> Self {
        let stderr = BufReader::new(child.stderr.take().unwrap());
        Self {
            child: Some(child),
            stderr: Some(stderr),
            log: String::new(),
        }
    }

    fn wait_until_listening(&mut self) {
        let read = self
            .stderr
            .as_mut()
            .unwrap()
            .read_line(&mut self.log)
            .unwrap();
        assert!(
            read > 0 && self.log.contains("game listener"),
            "server failed to start: {}",
            self.log
        );
    }

    fn stop(&mut self) -> String {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(mut stderr) = self.stderr.take() {
            let _ = stderr.read_to_string(&mut self.log);
        }
        self.log.clone()
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "command failed: {command:?}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
