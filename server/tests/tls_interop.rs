use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cepheus_trader_server::engine::{BbsRegistry, Engine, EngineError};
use cepheus_trader_server::store::{
    FlightLegPurpose, FlightLegRecord, ShipLocationRecord, Store, StoreError,
};
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

fn normalized_display_text(input: &str) -> String {
    strip_ecma48(input)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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
    acknowledged_page_prompts: usize,
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
            acknowledged_page_prompts: 0,
        }
    }

    fn send(&mut self, bytes: &[u8]) {
        let input = self.input.as_mut().unwrap();
        input.write_all(bytes).unwrap();
        input.flush().unwrap();
    }

    fn return_to_bbs(&mut self) {
        if normalized_display_text(&self.output()).contains("Ship status: Unregistered") {
            let (state, _) = self.wait_for_any_without_paging(&["Enter/Sp)", "Register captain"]);
            if state == 0 {
                self.send(b"q");
                self.wait_for("Register captain");
            }
        }
        self.send_through_page_prompt(b"q", "Return to the BBS?", "Return to the BBS?");
        self.send(b"y");
    }

    fn output(&self) -> String {
        String::from_utf8_lossy(&self.output.lock().unwrap()).into_owned()
    }

    fn acknowledge_page_prompts(&mut self, semantic: &str) {
        const PAGE_PROMPT: &str = "[Enter/Space] Continue  [C] Continuous";
        let erased_page_prompt = format!("\r{}\r", " ".repeat(PAGE_PROMPT.len()));
        let page_prompts =
            semantic.matches("Enter/Space").count() + semantic.matches("Enter/Sp)").count();
        let erased_page_prompts = self
            .output
            .lock()
            .unwrap()
            .windows(erased_page_prompt.len())
            .filter(|bytes| *bytes == erased_page_prompt.as_bytes())
            .count();
        self.acknowledged_page_prompts = self
            .acknowledged_page_prompts
            .max(erased_page_prompts.min(page_prompts));
        while self.acknowledged_page_prompts < page_prompts {
            let erased_before = self
                .output
                .lock()
                .unwrap()
                .windows(erased_page_prompt.len())
                .filter(|bytes| *bytes == erased_page_prompt.as_bytes())
                .count();
            self.send(b" ");
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let erased_now = self
                    .output
                    .lock()
                    .unwrap()
                    .windows(erased_page_prompt.len())
                    .filter(|bytes| *bytes == erased_page_prompt.as_bytes())
                    .count();
                if erased_now > erased_before {
                    break;
                }
                let output = self.output();
                assert!(
                    Instant::now() < deadline,
                    "door did not erase its page prompt; output: {output:?}"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            self.acknowledged_page_prompts += 1;
        }
    }

    fn wait_for(&mut self, text: &str) -> String {
        self.wait_for_occurrences(text, 1)
    }

    fn wait_for_occurrences(&mut self, text: &str, minimum: usize) -> String {
        let deadline = Instant::now() + Duration::from_secs(10);
        let expected = normalized_display_text(text);
        loop {
            let output = self.output();
            let semantic = normalized_display_text(&output);
            self.acknowledge_page_prompts(&semantic);
            if semantic.matches(&expected).count() >= minimum {
                return output;
            }
            assert!(
                Instant::now() < deadline,
                "door did not render occurrence {minimum} of {text:?}; output: {output:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_for_any(&mut self, texts: &[&str]) -> (usize, String) {
        let deadline = Instant::now() + Duration::from_secs(10);
        let expected = texts
            .iter()
            .map(|text| normalized_display_text(text))
            .collect::<Vec<_>>();
        loop {
            let output = self.output();
            let semantic = normalized_display_text(&output);
            self.acknowledge_page_prompts(&semantic);
            if let Some(index) = expected.iter().position(|text| semantic.contains(text)) {
                return (index, output);
            }
            assert!(
                Instant::now() < deadline,
                "door did not render any of {texts:?}; output: {output:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_for_any_without_paging(&self, texts: &[&str]) -> (usize, String) {
        let deadline = Instant::now() + Duration::from_secs(10);
        let expected = texts
            .iter()
            .map(|text| normalized_display_text(text))
            .collect::<Vec<_>>();
        loop {
            let output = self.output();
            let semantic = normalized_display_text(&output);
            if let Some(index) = expected.iter().position(|text| semantic.contains(text)) {
                return (index, output);
            }
            assert!(
                Instant::now() < deadline,
                "door did not render any of {texts:?}; output: {output:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn send_through_page_prompt(
        &mut self,
        bytes: &[u8],
        rendered_text: &str,
        progress_text: &str,
    ) -> String {
        let deadline = Instant::now() + Duration::from_secs(10);
        let rendered = normalized_display_text(rendered_text);
        let progress = normalized_display_text(progress_text);
        let initial = normalized_display_text(&self.output());
        let rendered_before = initial.matches(&rendered).count();
        let progress_before = initial.matches(&progress).count();
        let mut acknowledged_prompts = self.acknowledged_page_prompts;
        self.send(bytes);
        loop {
            let output = self.output();
            let semantic = normalized_display_text(&output);
            self.acknowledge_page_prompts(&semantic);
            if semantic.matches(&rendered).count() > rendered_before {
                return output;
            }
            if self.acknowledged_page_prompts > acknowledged_prompts {
                acknowledged_prompts = self.acknowledged_page_prompts;
                if semantic.matches(&progress).count() == progress_before {
                    self.send(bytes);
                }
            }
            assert!(
                Instant::now() < deadline,
                "door did not render {rendered_text:?} after input; output: {output:?}"
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

fn advance_simulation_to_due(engine: &Engine, due_second: u64) {
    loop {
        engine.recover().unwrap();
        let current_second = engine.game_second().unwrap();
        if due_second <= current_second {
            return;
        }
        match engine.advance_simulation_to(due_second) {
            Ok(_) => return,
            Err(EngineError::Store(StoreError::SimulationTimeReversal { .. })) => continue,
            Err(error) => panic!("simulation advance failed: {error}"),
        }
    }
}

fn settle_arrival_checkpoint(data: &Path, identity: &PlayerIdentity) {
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut request_id = 90_000_u64;
    loop {
        assert!(
            Instant::now() < deadline,
            "terminal approach did not settle before the interop timeout"
        );
        let engine = Engine::open(data, BbsRegistry::default()).unwrap();
        let (epoch, _, _) = engine.issue_session(identity).unwrap();
        loop {
            assert!(
                Instant::now() < deadline,
                "terminal approach encounter did not settle before the interop timeout"
            );
            if let Some(encounter) = engine.pending_encounter(identity).unwrap() {
                if encounter.state == EncounterState::AwaitingPosture {
                    engine
                        .submit(
                            identity.clone(),
                            engine_request(
                                epoch,
                                request_id,
                                WireCommand::ResolveEncounter(ResolveEncounterRequest {
                                    encounter_id: encounter.encounter_id,
                                    expected_revision: encounter.revision,
                                    posture: EncounterPosture::Comply,
                                    fallbacks: vec![EncounterFallback::Surrender],
                                }),
                            ),
                        )
                        .unwrap();
                    request_id += 1;
                }
                advance_simulation_to_due(
                    &engine,
                    if encounter.next_turn_second == 0 {
                        encounter.started_second + 1_000
                    } else {
                        encounter.next_turn_second
                    },
                );
                continue;
            }
            if let Some(checkpoint) = engine.pending_checkpoint(identity).unwrap() {
                engine
                    .submit(
                        identity.clone(),
                        engine_request(
                            epoch,
                            request_id,
                            WireCommand::AcknowledgeCheckpoint {
                                checkpoint_id: checkpoint.checkpoint_id,
                            },
                        ),
                    )
                    .unwrap();
                request_id += 1;
                continue;
            }
            break;
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
                let engine = Engine::open(data, BbsRegistry::default()).unwrap();
                advance_simulation_to_due(&engine, due_second);
            }
            other => panic!("terminal approach entered an unexpected locus: {other:?}"),
        }
    }
}

fn settle_pending_arrival_encounter(data: &Path, identity: &PlayerIdentity) {
    let deadline = Instant::now() + Duration::from_secs(60);
    let engine = Engine::open(data, BbsRegistry::default()).unwrap();
    let (epoch, _, _) = engine.issue_session(identity).unwrap();
    let mut request_id = 95_000_u64;
    loop {
        assert!(
            Instant::now() < deadline,
            "arrival encounter did not settle before the interop timeout"
        );
        let Some(encounter) = engine.pending_encounter(identity).unwrap() else {
            return;
        };
        if encounter.state == EncounterState::AwaitingPosture {
            engine
                .submit(
                    identity.clone(),
                    engine_request(
                        epoch,
                        request_id,
                        WireCommand::ResolveEncounter(ResolveEncounterRequest {
                            encounter_id: encounter.encounter_id,
                            expected_revision: encounter.revision,
                            posture: EncounterPosture::Comply,
                            fallbacks: vec![EncounterFallback::Surrender],
                        }),
                    ),
                )
                .unwrap();
            request_id += 1;
        }
        advance_simulation_to_due(
            &engine,
            if encounter.next_turn_second == 0 {
                encounter.started_second + 1_000
            } else {
                encounter.next_turn_second
            },
        );
    }
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
        let engine = Engine::open(data, BbsRegistry::default()).unwrap();
        advance_simulation_to_due(&engine, leg.due_second);
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

    let packet_key: &[u8] = match profile {
        "iso646" => b"\x1b[D",
        "iso646-color" => b"\x1b[C",
        "cp437-color" => b"\x1b[B",
        other => panic!("arrival profile has no arrow-key case: {other}"),
    };

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
            session.wait_for("entered it in the task ledger");
            session.send(b"\r");
            claimed = true;
            page_number += 1;
        } else {
            // Exercise all three arrival-packet arrow actions across
            // representative profiles using the sequences a caller sends.
            session.send(packet_key);
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
            session.send_through_page_prompt(
                b"\r",
                "Captain's Command Console",
                "Captain's Command Console",
            );
        } else {
            session.send_through_page_prompt(
                b"u",
                "Captain's Command Console",
                "Captain's Command Console",
            );
        }
    } else if arrival_result == 1 {
        session.send_through_page_prompt(
            b"\r",
            "Captain's Command Console",
            "Captain's Command Console",
        );
    } else {
        session.send_through_page_prompt(
            b"u",
            "Captain's Command Console",
            "Captain's Command Console",
        );
    }
    session.send(b"m");
    const MESSAGE_HEADING: &str = "Message Management\r\n==================";
    session.wait_for(MESSAGE_HEADING);
    session.send_through_page_prompt(
        b"i",
        "Message number on this page",
        "Message number on this page",
    );
    session.send_through_page_prompt(b"1", "Console", MESSAGE_HEADING);
    session.send_through_page_prompt(
        b"l",
        "Message number on this page",
        "Message number on this page",
    );
    let classified = session.send_through_page_prompt(b"2", "Console", MESSAGE_HEADING);
    assert!(strip_ecma48(&classified).contains("Ignored"));
    assert!(
        strip_ecma48(&classified).contains("Review"),
        "classification output: {classified:?}"
    );
    session.send_through_page_prompt(b"3", "Messages", "Communications Record");
    session.send_through_page_prompt(b"q", "Console", MESSAGE_HEADING);
    session.send_through_page_prompt(
        b"q",
        "Captain's Command Console",
        "Captain's Command Console",
    );
    if columns == "80" {
        session.send_through_page_prompt(
            b"k",
            "Ship's Navigation Library",
            "Ship's Navigation Library",
        );
        session.send_through_page_prompt(
            b"q",
            "Captain's Command Console",
            "Captain's Command Console",
        );
    }
    session.return_to_bbs();
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

    session.send_through_page_prompt(b"f", "Fuel source", "Fuel and Supplies");
    session.send_through_page_prompt(b"f", "Fuel source (Q to cancel", "Refined starship fuel");
    session.send(b"1\r");
    // The 40-column profile may wrap the prompt between "to" and
    // "cancel"; match the semantic prefix shared by every width.
    let (selection, _) = session.wait_for_any(&["Tonnes (Q to", "That service"]);
    if selection == 0 {
        session.send(b"1\r");
        session.wait_for("Ship's stores have been loaded");
    } else {
        session.wait_for("That service");
    }
    session.wait_for("(Enter) Previous menu");
    session.send_through_page_prompt(b"\r", "Docked Operations", "Docked Operations");

    // Exercise the facility-backed provision service when the destination
    // has a chandlery. At a frontier port, prove the same stale/forged key is
    // rejected with in-world copy rather than inventing stock.
    let supplies = session.send_through_page_prompt(b"f", "Fuel source", "Fuel and Supplies");
    session.send(b"p");
    if strip_ecma48(&supplies).contains("P) Provisions") {
        session.wait_for("Monthly packages (Q to");
        session.send(b"1\r");
        session.wait_for("Ship's stores have been loaded");
    } else {
        session.wait_for("No bonded chandlery");
    }
    session.wait_for("(Enter) Previous menu");
    session.send_through_page_prompt(b"\r", "Docked Operations", "Docked Operations");

    // Arrival processing may add task-titled cargo, and the wire snapshot is
    // the authority for menu order. Identify the purchased lot by its rendered
    // commodity name instead of assuming the stored vector has the same order.
    let cargo_name = {
        let store = Store::open(data).unwrap();
        let player = store.player_record(identity).unwrap().unwrap();
        store
            .ship_record(player.ship_id)
            .unwrap()
            .unwrap()
            .cargo
            .iter()
            .find(|lot| {
                lot.cargo_lot_id == cargo_lot_id
                    && lot.title == cepheus_trader_server::wire::CargoTitle::PlayerOwned
                    && lot.purchase_price_per_ton != 0
            })
            .expect("purchased speculative cargo must still be aboard")
            .commodity_name
            .clone()
    };
    // The heading is written before the rest of the page.  Wait for the
    // section delimiter used below so the reader thread has captured the
    // complete cargo list before taking the output snapshot.
    let exchange = session.send_through_page_prompt(b"c", "Find market", "Cargo Exchange -");
    let semantic = strip_ecma48(&exchange);
    let cargo_section = semantic
        .rsplit_once("Cargo Exchange -")
        .and_then(|(_, page)| page.split_once("Cargo aboard"))
        .and_then(|(_, cargo)| cargo.split_once("Port research"))
        .map(|(cargo, _)| cargo)
        .expect("cargo exchange omitted its cargo section");
    // Page prompts are erased with bare carriage returns, so a captured row
    // can share a logical line with the erased prompt.  Parse the normalized
    // display stream and take the numbered item immediately before the name.
    let normalized_cargo = normalized_display_text(cargo_section);
    assert!(normalized_cargo.contains("Range"));
    assert!(normalized_cargo.contains("Min Cr"));
    assert!(normalized_cargo.contains("Q1 Cr"));
    assert!(normalized_cargo.contains("Median Cr"));
    assert!(normalized_cargo.contains("Q3 Cr"));
    assert!(normalized_cargo.contains("Max Cr"));
    assert!(normalized_cargo.contains("-price sale"));
    let cargo_selection = normalized_cargo
        .split_once(&cargo_name)
        .and_then(|(before_name, _)| before_name.split_whitespace().next_back())
        .and_then(|number| number.strip_suffix('.'))
        .and_then(|number| number.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("cargo menu omitted {cargo_name:?}: {cargo_section:?}"));
    session.send_through_page_prompt(b"s", "Cargo lot (Q to cancel", "Cargo lot (Q to cancel");
    session.send(format!("{cargo_selection}\r").as_bytes());
    session.wait_for("Tonnes (maximum");
    session.send_through_page_prompt(b"1\r", "Find market", "Cargo Exchange -");
    session.send_through_page_prompt(b"q", "Docked Operations", "Docked Operations");
    session.return_to_bbs();
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

    let game_reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let admin_reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let sysop_reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let game_address = game_reservation.local_addr().unwrap();
    let admin_address = admin_reservation.local_addr().unwrap();
    let sysop_address = sysop_reservation.local_addr().unwrap();
    drop((game_reservation, admin_reservation, sysop_reservation));
    let game_port = game_address.port().to_string();
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
        .args([
            "--config",
            bbs_config.to_str().unwrap(),
            "--server",
            "127.0.0.1",
            "--game-port",
            game_port.as_str(),
            "--sysop-port",
            sysop_port.as_str(),
            "init-credential",
        ])
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
    assert!(bootstrap_config.contains("server=127.0.0.1\n"));
    assert!(bootstrap_config.contains(&format!("game-port={game_port}\n")));
    assert!(bootstrap_config.contains(&format!("sysop-port={sysop_port}\n")));
    assert!(bootstrap_config.contains("credential-file=cepheus-trader.credential\n"));
    assert!(bootstrap_config.contains("identity-file=cepheus-trader.identities\n"));
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
    for (profile_index, (profile, columns)) in [
        ("iso646", "40"),
        ("iso646-color", "80"),
        ("cp437-plain", "80"),
        ("cp437-color", "40"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut session = DoorSession::spawn(&door, data.path(), profile, columns);
        session.send(b"\r");
        session.wait_for_any_without_paging(&["No captain is registered"]);
        session.return_to_bbs();
        let screen = session.finish();
        let semantic_screen = normalized_display_text(&screen);
        assert!(screen.contains("An Alternate Cepheus Engine"), "{screen:?}");
        assert!(screen.contains("Universe"), "{screen:?}");
        assert!(
            semantic_screen.contains("Open Game License version 1.0a"),
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
        assert_eq!(
            semantic_screen.contains("Help - Welcome to the Marches"),
            profile_index == 0,
            "the new-player orientation must appear once: {screen:?}"
        );
        if !profile.ends_with("-color") {
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
        if profile.starts_with("cp437") {
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
        hello_line.starts_with("HELLO bbs=1 player=1 epoch=")
            && hello_line.ends_with(" phase=new-user language=en-US tls=TLS1.3"),
        "{hello_line}"
    );
    let hello_epoch = hello_line
        .split(" epoch=")
        .nth(1)
        .and_then(|tail| tail.split(' ').next())
        .and_then(|value| value.parse::<u64>().ok())
        .expect("ServerHello epoch is not numeric");
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
    docked_door.send(b"\r");
    docked_door.wait_for("Arrival Packet -");
    docked_door.send(b"q");
    docked_door.wait_for("Arrival Communications Receipt");
    docked_door.send(b"\r");
    docked_door.wait_for("Docked Operations");
    docked_door.wait_for("Return to BBS");
    docked_door.send(b"j");
    docked_door.wait_for("Task Ledger");
    docked_door.wait_for("Carriage declaration");
    let offer_list = normalized_display_text(&docked_door.wait_for("Offers available here ("));
    let offer_section = &offer_list[offer_list.rfind("Offers available here (").unwrap()..];
    let offer_header_end =
        offer_section.find(" unavailable hidden)").unwrap() + " unavailable hidden)".len();
    let showing_unavailable = offer_section[offer_header_end..]
        .trim_start()
        .starts_with("None ");
    if showing_unavailable {
        docked_door.send(b"v");
        docked_door.wait_for("Offers unavailable to this captain");
    }
    docked_door.send(b"i");
    docked_door.wait_for("Offer (Q to cancel");
    docked_door.send(b"1\r");
    docked_door.wait_for("Signed Offer Instrument");
    if showing_unavailable {
        docked_door.wait_for("Unavailable to this captain:");
    }
    docked_door.wait_for("(Q) Task ledger");
    docked_door.send(b"q");
    docked_door.wait_for_occurrences("Task Ledger", 2);
    docked_door.send_through_page_prompt(b"a", "Offer (Q to cancel", "Offer (Q to cancel");
    docked_door.send(b"1\r");
    docked_door.wait_for("entered it in the task ledger.");
    docked_door.wait_for("(Enter) Previous menu");
    docked_door.send(b"\r");
    docked_door.wait_for_occurrences("Task Ledger", 3);
    docked_door.send_through_page_prompt(b"t", "Task (Q to cancel", "Task (Q to cancel");
    docked_door.send(b"1\r");
    docked_door.wait_for("Accepted Task Instrument");
    docked_door.wait_for("Pickup:");
    docked_door.wait_for("Deliver to:");
    docked_door.wait_for("(Q) Task ledger");
    docked_door.send(b"q");
    docked_door.wait_for_occurrences("Task Ledger", 4);
    docked_door.send_through_page_prompt(b"q", "Docked Operations", "Docked Operations");
    docked_door.wait_for_occurrences("Return to BBS", 2);
    docked_door.return_to_bbs();
    let docked_screen = docked_door.finish();
    let docked_semantic = normalized_display_text(&docked_screen);
    for expected in [
        "Docked Operations",
        "Cargo Exchange",
        "Jobs and Passage",
        "Fuel and Supplies",
        "Shipyard",
        "Depart",
        "Universal",
        "Signed Offer Instrument",
        "Accepted Task Instrument",
        "Deliver to:",
        "Failure charge:",
        "Accepted obligations",
    ] {
        assert!(docked_semantic.contains(expected), "{docked_screen:?}");
    }
    assert!(docked_semantic.contains("entered it in the task ledger."));
    assert!(
        docked_semantic.contains("Help: j"),
        "menu prompt did not retain the cursor and echo its selection: {docked_screen:?}"
    );
    if showing_unavailable {
        assert!(docked_semantic.contains("Unavailable to this captain:"));
    }
    let mut reconnected_door = DoorSession::spawn(&door, data.path(), "iso646", "40");
    reconnected_door.send(b"\r");
    reconnected_door.wait_for("Docked Operations");
    reconnected_door.wait_for("Return to BBS");
    // Enter refreshes the docked display; leaving the game requires Q and a
    // separate affirmative confirmation.
    reconnected_door.send(b"\r");
    reconnected_door.wait_for_occurrences("Docked Operations", 2);
    reconnected_door.send(b"j");
    reconnected_door.wait_for("Task Ledger");
    reconnected_door.wait_for("Carriage declaration");
    reconnected_door.send(b"q");
    reconnected_door.wait_for_occurrences("Docked Operations", 3);
    reconnected_door.wait_for_occurrences("Return to BBS", 3);
    reconnected_door.return_to_bbs();
    let reconnect_screen = reconnected_door.finish();
    assert!(reconnect_screen.contains("Accepted obligations"));
    assert!(reconnect_screen.contains("No.1 "));
    assert!(reconnect_screen.contains("Reserved:"));

    // Existing captains are never interrupted by a newly installed tutorial,
    // but may voluntarily open and hide the BBS-local guidance at 40 columns.
    let mut first_watch_door = DoorSession::spawn(&door, data.path(), "iso646", "40");
    first_watch_door.send(b"\r");
    first_watch_door.wait_for("Docked Operations");
    assert!(!normalized_display_text(&first_watch_door.output()).contains("Guided First Watch"));
    first_watch_door.send_through_page_prompt(
        b"u",
        "Captain's Command Console",
        "Captain's Command Console",
    );
    first_watch_door.wait_for("Guided First Watch");
    first_watch_door.send_through_page_prompt(b"w", "Taking the watch", "Taking the watch");
    first_watch_door.send_through_page_prompt(b"\r", "The people aboard", "The people aboard");
    first_watch_door.send_through_page_prompt(
        b"h",
        "Captain's Command Console",
        "Captain's Command Console",
    );
    first_watch_door.send_through_page_prompt(b"x", "Docked Operations", "Docked Operations");
    first_watch_door.return_to_bbs();
    let first_watch_screen = normalized_display_text(&first_watch_door.finish());
    assert!(first_watch_screen.contains("This is the live command"));
    assert!(first_watch_screen.contains("The people aboard"));

    // Context help pages through the real door, then restores the exact
    // operational prompt that was active when the player pressed `?`.
    let mut help_door = DoorSession::spawn(&door, data.path(), "iso646", "40");
    help_door.send(b"\r");
    help_door.wait_for("Docked Operations");
    help_door.wait_for("Return to BBS");
    help_door.send(b"?");
    help_door.wait_for("Help - Docked operations");
    help_door.wait_for("(Enter) Resume");
    help_door.send(b"\r");
    help_door.wait_for_occurrences("Return to BBS", 2);
    help_door.return_to_bbs();
    let help_screen = normalized_display_text(&help_door.finish());
    assert!(help_screen.contains("services actually available here"));
    assert!(help_screen.contains("(?) Help"));

    // Automatic continuation pauses are a durable local-player preference.
    // Disabling them must stream a long 40-column manager without weakening
    // its actual action prompt, and the choice must survive a new door process.
    let mut preference_door = DoorSession::spawn(&door, data.path(), "iso646", "40");
    preference_door.send(b"\r");
    preference_door.wait_for("Docked Operations");
    preference_door.send_through_page_prompt(
        b"u",
        "Captain's Command Console",
        "Captain's Command Console",
    );
    preference_door.send(b"p");
    preference_door.wait_for("Player Preferences");
    preference_door.wait_for("Page pauses:  Enabled");
    preference_door.send(b"h");
    preference_door.wait_for("Default Help Level");
    preference_door.wait_for("(X) Expert");
    let beginner_editor = normalized_display_text(&preference_door.output());
    let beginner_editor = &beginner_editor[beginner_editor.rfind("Default Help Level").unwrap()..];
    assert!(!beginner_editor.contains("(B) Beginner"));
    preference_door.send(b"x");
    preference_door.wait_for("Default help: Expert");
    preference_door.send(b"h");
    preference_door.wait_for_occurrences("Default Help Level", 2);
    preference_door.wait_for("(B) Beginner");
    let expert_editor = normalized_display_text(&preference_door.output());
    let expert_editor = &expert_editor[expert_editor.rfind("Default Help Level").unwrap()..];
    assert!(!expert_editor.contains("(X) Expert"));
    preference_door.send(b"b");
    preference_door.wait_for("Default help: Beginner");
    preference_door.send(b"p");
    preference_door.wait_for("Automatic Page Pauses");
    preference_door.send(b"d");
    preference_door.wait_for("Page pauses:  Disabled");
    let pauses_before = normalized_display_text(&preference_door.output())
        .matches("Enter/Space")
        .count();
    preference_door.send(b"q");
    preference_door.wait_for_occurrences("Captain's Command Console", 2);
    preference_door.send(b"c");
    preference_door.wait_for("Crew Management -");
    preference_door.wait_for("managed appointments");
    let pauses_after = normalized_display_text(&preference_door.output())
        .matches("Enter/Space")
        .count();
    assert_eq!(pauses_after, pauses_before);
    preference_door.send(b"q");
    preference_door.wait_for_occurrences("Captain's Command Console", 3);
    preference_door.send(b"x");
    preference_door.wait_for_occurrences("Docked Operations", 2);
    preference_door.return_to_bbs();
    let preference_screen = normalized_display_text(&preference_door.finish());
    assert!(preference_screen.contains("Page pauses: Disabled"));

    let mut persisted_preference_door = DoorSession::spawn(&door, data.path(), "iso646", "40");
    persisted_preference_door.send(b"\r");
    persisted_preference_door.wait_for("Docked Operations");
    persisted_preference_door.send(b"u");
    persisted_preference_door.wait_for("Captain's Command Console");
    persisted_preference_door.send(b"p");
    persisted_preference_door.wait_for("Player Preferences");
    persisted_preference_door.wait_for("Page pauses:  Disabled");
    persisted_preference_door.send(b"p");
    persisted_preference_door.wait_for("Automatic Page Pauses");
    persisted_preference_door.send(b"e");
    persisted_preference_door.wait_for("Page pauses:  Enabled");
    persisted_preference_door.send(b"q");
    persisted_preference_door.wait_for_occurrences("Captain's Command Console", 2);
    persisted_preference_door.send_through_page_prompt(
        b"x",
        "Docked Operations",
        "Docked Operations",
    );
    persisted_preference_door.return_to_bbs();
    let persisted_preference_screen = normalized_display_text(&persisted_preference_door.finish());
    assert!(persisted_preference_screen.contains("Page pauses: Disabled"));
    assert!(persisted_preference_screen.contains("Page pauses: Enabled"));

    // Exercise the Milestone 5 correspondence controls through the real door
    // rather than merely proving their wire codecs. Banking is also exercised
    // when the generated home port provides that optional service.
    let mut services_door = DoorSession::spawn(&door, data.path(), "iso646", "40");
    services_door.send(b"\r");
    let services_menu = services_door.wait_for("Return to BBS");
    let banking_available = services_menu.contains("Banking and Accounts");
    if banking_available {
        services_door.send(b"b");
        services_door.wait_for("Banking and Accounts");
        services_door.wait_for("Cr350,000");
        services_door.send(b"b");
        services_door.wait_for("Purchase one year of destination assistance");
        services_door.send(b"y");
        services_door.wait_for_occurrences("Destination aid:", 2);
        services_door.wait_for("Day 365");
        services_door.send(b"q");
        services_door.wait_for_occurrences("Docked Operations", 2);
        services_door.wait_for_occurrences("Return to BBS", 2);
    }
    services_door.send_through_page_prompt(
        b"u",
        "Captain's Command Console",
        "Captain's Command Console",
    );
    services_door.wait_for("(C/K/M/O/R/S/T) Manager");
    services_door.send(b"c");
    services_door.wait_for("Crew Management -");
    services_door.wait_for("Complement:");
    services_door.wait_for("managed appointments");
    services_door.send(b"q");
    services_door.wait_for_occurrences("Captain's Command Console", 2);
    services_door.wait_for_occurrences("(C/K/M/O/R/S/T) Manager", 2);
    services_door.send(b"s");
    services_door.wait_for("Ship Status -");
    services_door.wait_for("Next automatic upkeep:");
    services_door.wait_for("no yard order is needed");
    services_door.wait_for("operating account funded");
    services_door.wait_for("uncovered cycle");
    services_door.wait_for("damage a subsystem");
    services_door.send(b"q");
    services_door.wait_for_occurrences("Captain's Command Console", 3);
    services_door.wait_for_occurrences("(C/K/M/O/R/S/T) Manager", 3);
    services_door.send(b"m");
    services_door.wait_for("Message Management");
    services_door.send_through_page_prompt(b"c", "Recipient", "Recipient");
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
    services_door.wait_for("(Enter) Previous menu");
    services_door.send(b"\r");
    services_door.wait_for_occurrences("Message Management", 2);
    services_door.wait_for_occurrences("(Q) Console", 2);
    services_door.send_through_page_prompt(
        b"q",
        "Captain's Command Console",
        "Captain's Command Console",
    );
    services_door.wait_for_occurrences("(C/K/M/O/R/S/T) Manager", 4);
    services_door.send(b"x");
    let final_docked_occurrence = if banking_available { 3 } else { 2 };
    services_door.wait_for_occurrences("Docked Operations", final_docked_occurrence);
    services_door.wait_for_occurrences("Return to BBS", final_docked_occurrence);
    services_door.return_to_bbs();
    let services_screen = services_door.finish();
    let services_semantic = normalized_display_text(&services_screen);
    for expected in [
        "Next automatic upkeep:",
        "no yard order is needed",
        "damage a subsystem",
        "Message Management",
        "accepted for physical",
    ] {
        assert!(services_semantic.contains(expected), "{services_screen:?}");
    }
    if banking_available {
        assert!(
            services_semantic.contains("Covered through"),
            "{services_screen:?}"
        );
    }

    // Prove the Milestone 6 player boundary through the real TLS/OpenDoors
    // client without leaving the canonical merchant voyage in combat. The
    // clone starts from the same authenticated, created captain and is thrown
    // away after the assertions.
    server.stop();
    let combat_root = tempfile::tempdir().unwrap();
    copy_directory(data.path(), combat_root.path());
    let local_contact_available = {
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
        found
    };
    if local_contact_available {
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
        combat_door.send_through_page_prompt(
            b"u",
            "Captain's Command Console",
            "Captain's Command Console",
        );
        combat_door.wait_for("(C/K/M/O/R/S/T) Manager");
        combat_door.send(b"o");
        combat_door.wait_for_occurrences("Accept order or file report", 1);
        combat_door.send(b"m");
        combat_door.wait_for("Naval service");
        combat_door.send(b"r");
        combat_door.wait_for_occurrences("Operations Ledger", 2);
        combat_door.wait_for("Service: Pirate");
        combat_door.wait_for_occurrences("Accept order or file report", 2);
        combat_door.send(b"i");
        combat_door.wait_for("Contact (Q to cancel");
        combat_door.send(b"1\r");
        combat_door.wait_for("Board or inspect");
        combat_door.send(b"a");
        combat_door.wait_for("irreversible act");
        combat_door.wait_for("Confirm intercept");
        combat_door.send(b"i");
        combat_door.wait_for("Vessel Combat");
        combat_door.wait_for("Standing policy");
        combat_door.send(b"d");
        combat_door.wait_for("Joint orders sealed for this activation");
        let combat_screen = combat_door.terminate();
        combat_server.stop();
        for expected in [
            "Operations Ledger",
            "Service: Pirate",
            "Vessel Combat",
            "General quarters",
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
                    cepheus_trader_server::wire::OutcomeKind::CombatCareer(snapshot) => {
                        Some(snapshot)
                    }
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
    voyage_door.send_through_page_prompt(
        b"u",
        "Captain's Command Console",
        "Captain's Command Console",
    );
    voyage_door.send(b"p");
    voyage_door.wait_for("Player Preferences");
    voyage_door.send_through_page_prompt(b"r", "Taking the watch", "Taking the watch");
    voyage_door.send_through_page_prompt(
        b"q",
        "Captain's Command Console",
        "Captain's Command Console",
    );
    voyage_door.send_through_page_prompt(b"x", "Docked Operations", "Docked Operations");
    voyage_door.send_through_page_prompt(b"c", "Find market", "Cargo Exchange -");
    voyage_door.send_through_page_prompt(b"b", "Offer (Q to cancel", "Offer (Q to cancel");
    voyage_door.send(b"1\r");
    voyage_door.wait_for("Tonnes (maximum");
    voyage_door.send_through_page_prompt(b"1\r", "Find market", "Cargo Exchange -");
    voyage_door.send_through_page_prompt(b"q", "Docked Operations", "Docked Operations");
    voyage_door.send_through_page_prompt(
        b"d",
        "Flight Plan\r\n===========",
        "Flight Plan\r\n===========",
    );
    voyage_door.send_through_page_prompt(b"a", "Add Charted Leg", "Add Charted Leg");
    voyage_door.send(b"1");
    voyage_door.wait_for("Buy fresh course tape");
    voyage_door.send(b"o");
    voyage_door.wait_for("identifies a bad plot");
    voyage_door.send_through_page_prompt(
        b"r",
        "Flight Plan\r\n===========",
        "Flight Plan\r\n===========",
    );
    voyage_door.send_through_page_prompt(b"p", "File this plan", "Flight Plan Preview");
    voyage_door.send_through_page_prompt(b"f", "Previous menu", "Departure authorized.");
    voyage_door.send_through_page_prompt(b"\r", "Voyage Status -", "Voyage Status -");
    voyage_door.send_through_page_prompt(
        b"\r",
        "Captain's Command Console",
        "Captain's Command Console",
    );
    voyage_door.send(b"p");
    voyage_door.wait_for("Player Preferences");
    voyage_door.wait_for("First Watch:  Complete");
    voyage_door.send_through_page_prompt(
        b"q",
        "Captain's Command Console",
        "Captain's Command Console",
    );
    voyage_door.return_to_bbs();
    let voyage_screen = voyage_door.finish();
    for expected in [
        "Cargo Exchange -",
        "Cargo aboard",
        "Enter/Space",
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

    // Save an unpresented arrival state for representative terminal profiles.
    // Each copy uses the real TLS server and OpenDoors executable, while
    // preserving the canonical voyage for the final continuation.
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
        let semantic = normalized_display_text(&profile_screen);
        for expected in [
            "Arrival Packet -",
            "Communications Record",
            "entered it in the task ledger",
            "Arrival Communications Receipt",
            "Message Management",
            "Review",
        ] {
            assert!(semantic.contains(expected), "{profile}: {profile_screen:?}");
        }
        if columns == "80" {
            assert!(
                semantic.contains("Ship's Navigation Library"),
                "{profile}: {profile_screen:?}"
            );
        }
        for duplicate_prompt in [
            "(Enter) Continue\r\n\r\n(Enter) Previous menu",
            "(Enter) Return\r\n\r\n(Enter) Previous menu",
            "[Enter] Continue\r\n\r\n[Enter] Previous menu",
            "[Enter] Return\r\n\r\n[Enter] Previous menu",
        ] {
            assert!(
                !semantic.contains(duplicate_prompt),
                "{profile}: duplicate prompt in {profile_screen:?}"
            );
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
    for lifecycle_event in [
        "event=accepted",
        "event=tls-authenticated",
        "event=session-opened",
        "event=session-closed",
    ] {
        assert!(
            server_log.contains(lifecycle_event),
            "server log is missing {lifecycle_event}: {server_log}"
        );
    }
    assert!(
        reconnect.status.success(),
        "reconnect failed\nstdout: {}\nstderr: {}\nserver: {server_log}",
        String::from_utf8_lossy(&reconnect.stdout),
        String::from_utf8_lossy(&reconnect.stderr)
    );
    let reconnect_output = String::from_utf8_lossy(&reconnect.stdout);
    let reconnect_line = reconnect_output.trim();
    assert!(
        reconnect_line.starts_with("HELLO bbs=1 player=1 epoch=")
            && reconnect_line.ends_with(" phase=docked language=en-US tls=TLS1.3"),
        "{reconnect_line}"
    );
    let reconnect_epoch = reconnect_line
        .split(" epoch=")
        .nth(1)
        .and_then(|tail| tail.split(' ').next())
        .and_then(|value| value.parse::<u64>().ok())
        .expect("reconnect ServerHello epoch is not numeric");
    assert!(reconnect_epoch > hello_epoch);
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
            && post_reset_hello.ends_with(" phase=new-user language=en-US tls=TLS1.3"),
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
