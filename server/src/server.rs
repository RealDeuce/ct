//! Development TCP adapter and engine actor.

use std::collections::HashMap;
use std::io;
use std::net::{Shutdown, SocketAddr, TcpStream as StdTcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use socket2::{Domain, Protocol, Socket, Type};
use thiserror::Error;
#[cfg(test)]
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Semaphore, mpsc, oneshot, watch};
use tokio::task::JoinHandle;

use crate::engine::BbsRegistry;
use crate::engine::{Engine, EngineError};
use crate::store::{
    BbsConfiguration, BbsCredential, BbsSettings, ConfigureBbsResult, Delivery, OperationalStatus,
    PlayerAccessRecord, PlayerAccessState, PlayerTravelTransition, SetPlayerAccessResult,
    StoreError, SysopDirectiveKind, SysopDirectiveRecord,
};
use crate::tls::{PskCredential, TlsServer};
use crate::traffic::{TrafficContact, TrafficSnapshot};
use crate::universe::UniverseInitialization;
use crate::wire::{
    MAX_FRAME_BYTES, PROTOCOL_VERSION, PlayerIdentity, WireError, decode_client_hello_with_version,
    decode_close, decode_request, encode_checkpoint_ready, encode_close, encode_close_for_version,
    encode_encounter_ready, encode_phase_changed, encode_response, encode_server_hello,
    encode_server_stopping, encode_session_replaced, encode_traffic_movement,
    encode_traffic_snapshot,
};
use crate::{admin_wire, sysop_wire, wire};

const CONNECTION_QUEUE_DEPTH: usize = 64;
const ENGINE_QUEUE_DEPTH: usize = 256;
const AUTHENTICATION_TIMEOUT: Duration = Duration::from_secs(10);
const LIVE_CLOCK_PULSE: Duration = Duration::from_secs(1);
const LIVE_CLOCK_EVENT_SLICE: u64 = 1_024;
const MAX_PENDING_GAME_AUTHENTICATIONS: usize = 64;
const MAX_ACTIVE_GAME_SESSIONS: usize = 256;
const MAX_ACTIVE_GAME_SESSIONS_PER_BBS: usize = 64;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("network I/O error: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    AdminWire(#[from] admin_wire::AdminWireError),
    #[error(transparent)]
    SysopWire(#[from] sysop_wire::SysopWireError),
    #[error("authoritative engine stopped")]
    EngineStopped,
    #[error("authoritative engine failure: {0}")]
    Engine(String),
    #[error("TLS failure: {0}")]
    Tls(String),
    #[error("hello BBS identifier does not match the authenticated TLS PSK identity")]
    BbsIdentityMismatch,
    #[error("authenticated TLS PSK identity is not a canonical BBS identifier")]
    InvalidBbsPskIdentity,
    #[error("TLS worker stopped unexpectedly")]
    TlsWorkerStopped,
    #[error("administrator listener must bind to a loopback address")]
    AdminNotLoopback,
    #[error("at least one {0} listener address is required")]
    NoListenerAddresses(&'static str),
    #[error("all network listener tasks stopped unexpectedly")]
    ListenerStopped,
    #[error("game connection limit reached")]
    ConnectionLimit,
}

#[derive(Clone)]
pub struct AdminTlsConfig {
    pub key: Arc<Vec<u8>>,
    pub backup_root: Arc<PathBuf>,
}

enum EngineMessage {
    ClockPulse {
        sampled_at: Instant,
    },
    OpenSession {
        identity: PlayerIdentity,
        reply: oneshot::Sender<Result<SessionOpening, String>>,
    },
    CloseSession {
        identity: PlayerIdentity,
        epoch: u64,
    },
    Submit {
        identity: PlayerIdentity,
        request: crate::wire::CommandRequest,
    },
    AcknowledgeOutbox(u64),
    AddBbs {
        command_id: [u8; wire::COMMAND_ID_BYTES],
        name: String,
        reply: oneshot::Sender<Result<BbsCredential, String>>,
    },
    InitializeUniverse {
        command_id: [u8; wire::COMMAND_ID_BYTES],
        reply: oneshot::Sender<Result<UniverseInitialization, String>>,
    },
    GetBbsConfiguration {
        bbs_id: u32,
        reply: oneshot::Sender<Result<BbsConfiguration, String>>,
    },
    ConfigureBbs {
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        expected_revision: u64,
        settings: BbsSettings,
        reply: oneshot::Sender<Result<ConfigureBbsResult, String>>,
    },
    GetPlayerAccess {
        identity: PlayerIdentity,
        reply: oneshot::Sender<Result<PlayerAccessRecord, String>>,
    },
    SetPlayerAccess {
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        player_id: u32,
        expected_revision: u64,
        state: PlayerAccessState,
        reason: String,
        reply: oneshot::Sender<Result<SetPlayerAccessResult, String>>,
    },
    GetStatus {
        reply: oneshot::Sender<Result<OperationalStatus, String>>,
    },
    LiveBackup {
        backup_root: PathBuf,
        label: String,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        reply: oneshot::Sender<Result<OperationalStatus, String>>,
    },
    Shutdown {
        reply: oneshot::Sender<()>,
    },
    IssueSysopDirective {
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        player_id: u32,
        kind: SysopDirectiveKind,
        reply: oneshot::Sender<Result<SysopDirectiveRecord, String>>,
    },
}

enum EngineEvent {
    Delivery(Box<Delivery>),
    PhaseChanged(Box<PlayerTravelTransition>),
    CheckpointReady {
        identity: PlayerIdentity,
        committed_sequence: u64,
        checkpoint: Box<wire::CheckpointSnapshot>,
    },
    EncounterReady {
        identity: PlayerIdentity,
        committed_sequence: u64,
        encounter: Box<wire::EncounterSnapshot>,
    },
    TrafficSnapshot {
        identity: PlayerIdentity,
        committed_sequence: u64,
        snapshot: Box<TrafficSnapshot>,
    },
    TrafficMovement {
        identity: PlayerIdentity,
        committed_sequence: u64,
        system_id: u64,
        observed_second: u64,
        contact: Box<TrafficContact>,
    },
    UniverseReset,
    PlayerAccessChanged(Box<PlayerAccessRecord>),
    Fatal(String),
}

struct SessionOpening {
    epoch: u64,
    committed_sequence: u64,
    phase: wire::PlayerPhase,
    traffic_snapshot: Option<TrafficSnapshot>,
    checkpoint: Option<wire::CheckpointSnapshot>,
    encounter: Option<wire::EncounterSnapshot>,
}

struct Observer {
    epoch: u64,
    system_id: Option<u64>,
    last_second: u64,
}

fn emit_best_effort(sender: &mpsc::Sender<EngineEvent>, event: EngineEvent) -> bool {
    match sender.try_send(event) {
        Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => true,
        Err(mpsc::error::TrySendError::Closed(_)) => false,
    }
}

async fn shutdown_signal() -> io::Result<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = terminate.recv() => Ok(()),
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await
    }
}

fn emit_observer_movements(
    engine: &Engine,
    sender: &mpsc::Sender<EngineEvent>,
    identity: &PlayerIdentity,
    observer: &mut Observer,
    through_second: u64,
) -> Result<bool, EngineError> {
    let Some(system_id) = observer.system_id else {
        observer.last_second = through_second;
        return Ok(true);
    };
    let sequence = engine.committed_sequence()?;
    for contact in engine.traffic_movements(system_id, observer.last_second, through_second)? {
        if !emit_best_effort(
            sender,
            EngineEvent::TrafficMovement {
                identity: identity.clone(),
                committed_sequence: sequence,
                system_id,
                observed_second: contact.edge_second,
                contact: Box::new(contact),
            },
        ) {
            return Ok(false);
        }
    }
    observer.last_second = through_second;
    Ok(true)
}

fn emit_transition(
    engine: &Engine,
    sender: &mpsc::Sender<EngineEvent>,
    observers: &mut HashMap<PlayerIdentity, Observer>,
    transition: PlayerTravelTransition,
) -> Result<bool, EngineError> {
    if let Some(observer) = observers.get_mut(&transition.identity)
        && !emit_observer_movements(
            engine,
            sender,
            &transition.identity,
            observer,
            transition.status.current_game_second,
        )?
    {
        return Ok(false);
    }
    if sender
        .blocking_send(EngineEvent::PhaseChanged(Box::new(transition.clone())))
        .is_err()
    {
        return Ok(false);
    }
    if let Some(checkpoint) = engine.pending_checkpoint(&transition.identity)?
        && sender
            .blocking_send(EngineEvent::CheckpointReady {
                identity: transition.identity.clone(),
                committed_sequence: transition.committed_sequence,
                checkpoint: Box::new(checkpoint),
            })
            .is_err()
    {
        return Ok(false);
    }
    if let Some(encounter) = engine.pending_encounter(&transition.identity)?
        && sender
            .blocking_send(EngineEvent::EncounterReady {
                identity: transition.identity.clone(),
                committed_sequence: transition.committed_sequence,
                encounter: Box::new(encounter),
            })
            .is_err()
    {
        return Ok(false);
    }
    if let Some(observer) = observers.get_mut(&transition.identity) {
        let old_system = observer.system_id;
        observer.system_id = match transition.phase {
            wire::PlayerPhase::Docked | wire::PlayerPhase::Interplanetary => {
                Some(transition.status.current_system_id)
            }
            wire::PlayerPhase::Jump | wire::PlayerPhase::NewUser | wire::PlayerPhase::Terminal => {
                None
            }
            wire::PlayerPhase::Encounter => Some(transition.status.current_system_id),
        };
        observer.last_second = transition.status.current_game_second;
        if observer.system_id.is_some()
            && observer.system_id != old_system
            && let Some(snapshot) = engine.traffic_snapshot(&transition.identity)?
            && !emit_best_effort(
                sender,
                EngineEvent::TrafficSnapshot {
                    identity: transition.identity,
                    committed_sequence: transition.committed_sequence,
                    snapshot: Box::new(snapshot),
                },
            )
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn emit_advance(
    engine: &Engine,
    sender: &mpsc::Sender<EngineEvent>,
    observers: &mut HashMap<PlayerIdentity, Observer>,
    advance: crate::store::SimulationAdvance,
) -> Result<bool, EngineError> {
    for transition in advance.player_transitions {
        if !emit_transition(engine, sender, observers, transition)? {
            return Ok(false);
        }
    }
    for (identity, observer) in observers.iter_mut() {
        if !emit_observer_movements(engine, sender, identity, observer, advance.ending_second)? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Clone)]
struct EngineHandle {
    sender: mpsc::Sender<EngineMessage>,
}

impl EngineHandle {
    async fn issue_session(&self, identity: PlayerIdentity) -> Result<SessionOpening, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::OpenSession { identity, reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn close_session(&self, identity: PlayerIdentity, epoch: u64) {
        let _ = self
            .sender
            .send(EngineMessage::CloseSession { identity, epoch })
            .await;
    }

    async fn add_bbs(
        &self,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        name: String,
    ) -> Result<BbsCredential, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::AddBbs {
                command_id,
                name,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn initialize_universe(
        &self,
        command_id: [u8; wire::COMMAND_ID_BYTES],
    ) -> Result<UniverseInitialization, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::InitializeUniverse { command_id, reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn bbs_configuration(&self, bbs_id: u32) -> Result<BbsConfiguration, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::GetBbsConfiguration { bbs_id, reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn configure_bbs(
        &self,
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        expected_revision: u64,
        settings: BbsSettings,
    ) -> Result<ConfigureBbsResult, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::ConfigureBbs {
                bbs_id,
                command_id,
                expected_revision,
                settings,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn player_access(
        &self,
        identity: PlayerIdentity,
    ) -> Result<PlayerAccessRecord, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::GetPlayerAccess { identity, reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn set_player_access(
        &self,
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        player_id: u32,
        expected_revision: u64,
        state: PlayerAccessState,
        reason: String,
    ) -> Result<SetPlayerAccessResult, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::SetPlayerAccess {
                bbs_id,
                command_id,
                player_id,
                expected_revision,
                state,
                reason,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn status(&self) -> Result<OperationalStatus, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::GetStatus { reply })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn live_backup(
        &self,
        backup_root: PathBuf,
        label: String,
        command_id: [u8; wire::COMMAND_ID_BYTES],
    ) -> Result<OperationalStatus, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::LiveBackup {
                backup_root,
                label,
                command_id,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }

    async fn shutdown(&self) {
        let (reply, receiver) = oneshot::channel();
        if self
            .sender
            .send(EngineMessage::Shutdown { reply })
            .await
            .is_ok()
        {
            let _ = receiver.await;
        }
    }

    async fn issue_sysop_directive(
        &self,
        bbs_id: u32,
        command_id: [u8; wire::COMMAND_ID_BYTES],
        player_id: u32,
        kind: SysopDirectiveKind,
    ) -> Result<SysopDirectiveRecord, ServerError> {
        let (reply, receiver) = oneshot::channel();
        self.sender
            .send(EngineMessage::IssueSysopDirective {
                bbs_id,
                command_id,
                player_id,
                kind,
                reply,
            })
            .await
            .map_err(|_| ServerError::EngineStopped)?;
        receiver
            .await
            .map_err(|_| ServerError::EngineStopped)?
            .map_err(ServerError::Engine)
    }
}

fn spawn_engine(
    data_path: PathBuf,
    bbs_registry: BbsRegistry,
) -> (
    EngineHandle,
    mpsc::Receiver<EngineEvent>,
    thread::JoinHandle<()>,
    std::sync::mpsc::Receiver<()>,
) {
    let (input_sender, mut input_receiver) = mpsc::channel(ENGINE_QUEUE_DEPTH);
    let (event_sender, event_receiver) = mpsc::channel(ENGINE_QUEUE_DEPTH);
    let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);
    let join = thread::Builder::new()
        .name("ct-authoritative-engine".into())
        .spawn(move || {
            let run = || -> Result<(), EngineError> {
                let engine = Engine::open(data_path, bbs_registry)?;
                let recovered = engine.recover()?;
                let mut live_clock = crate::clock::LiveClock::now(engine.game_second()?);
                let mut observers = HashMap::<PlayerIdentity, Observer>::new();
                let mut last_lag_report = Instant::now() - Duration::from_secs(60);
                let _ = ready_sender.send(());
                for transition in recovered.player_transitions {
                    if !emit_transition(&engine, &event_sender, &mut observers, transition)? {
                        return Ok(());
                    }
                }
                for delivery in engine.pending_outbox()? {
                    if event_sender
                        .blocking_send(EngineEvent::Delivery(Box::new(delivery)))
                        .is_err()
                    {
                        return Ok(());
                    }
                }
                while let Some(message) = input_receiver.blocking_recv() {
                    match message {
                        EngineMessage::ClockPulse { sampled_at } => {
                            let target = live_clock.target_second(sampled_at);
                            let advance =
                                engine.advance_simulation_toward(target, LIVE_CLOCK_EVENT_SLICE)?;
                            if advance.ending_second < target
                                && last_lag_report.elapsed() >= Duration::from_secs(60)
                            {
                                eprintln!(
                                    "live clock lag: target={} committed={} lag={}m events={} wall={}ns cpu={}ns",
                                    target,
                                    advance.ending_second,
                                    target - advance.ending_second,
                                    advance.processed_events,
                                    advance.wall_nanoseconds,
                                    advance.thread_cpu_nanoseconds.unwrap_or(0),
                                );
                                last_lag_report = Instant::now();
                            }
                            if !emit_advance(&engine, &event_sender, &mut observers, advance)? {
                                return Ok(());
                            }
                        }
                        EngineMessage::OpenSession { identity, reply } => {
                            match engine.issue_session(&identity) {
                                Ok((epoch, committed_sequence, phase)) => {
                                    let traffic_snapshot = engine.traffic_snapshot(&identity)?;
                                    let current_second = engine.game_second()?;
                                    observers.insert(
                                        identity.clone(),
                                        Observer {
                                            epoch,
                                            system_id: traffic_snapshot
                                                .as_ref()
                                                .map(|snapshot| snapshot.system_id),
                                            last_second: current_second,
                                        },
                                    );
                                    let _ = reply.send(Ok(SessionOpening {
                                        epoch,
                                        committed_sequence,
                                        phase,
                                        traffic_snapshot,
                                        checkpoint: engine.pending_checkpoint(&identity)?,
                                        encounter: engine.pending_encounter(&identity)?,
                                    }));
                                }
                                Err(error @ EngineError::PlayerAccessDenied(_)) => {
                                    let _ = reply.send(Err(error.to_string()));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::CloseSession { identity, epoch } => {
                            if observers
                                .get(&identity)
                                .is_some_and(|observer| observer.epoch == epoch)
                            {
                                observers.remove(&identity);
                            }
                        }
                        EngineMessage::Submit { identity, request } => {
                            let batch = engine.submit(identity.clone(), request)?;
                            for delivery in batch.deliveries {
                                if event_sender
                                    .blocking_send(EngineEvent::Delivery(Box::new(delivery)))
                                    .is_err()
                                {
                                    return Ok(());
                                }
                            }
                            for transition in batch.player_transitions {
                                if !emit_transition(
                                    &engine,
                                    &event_sender,
                                    &mut observers,
                                    transition,
                                )? {
                                    return Ok(());
                                }
                            }
                            if let Some(observer) = observers.get_mut(&identity) {
                                let snapshot = engine.traffic_snapshot(&identity)?;
                                let new_system = snapshot.as_ref().map(|value| value.system_id);
                                if new_system != observer.system_id {
                                    observer.system_id = new_system;
                                    observer.last_second = engine.game_second()?;
                                    if let Some(snapshot) = snapshot
                                        && !emit_best_effort(
                                            &event_sender,
                                            EngineEvent::TrafficSnapshot {
                                                identity,
                                                committed_sequence: engine.committed_sequence()?,
                                                snapshot: Box::new(snapshot),
                                            },
                                        )
                                    {
                                        return Ok(());
                                    }
                                }
                            }
                        }
                        EngineMessage::AcknowledgeOutbox(sequence) => {
                            engine.acknowledge_outbox(sequence)?;
                        }
                        EngineMessage::AddBbs {
                            command_id,
                            name,
                            reply,
                        } => match engine.add_bbs(command_id, &name) {
                            Ok(credential) => {
                                let _ = reply.send(Ok(credential));
                            }
                            Err(error @ EngineError::Store(StoreError::UniverseNotInitialized)) => {
                                let _ = reply.send(Err(error.to_string()));
                            }
                            Err(error) => {
                                let _ = reply.send(Err(error.to_string()));
                                return Err(error);
                            }
                        },
                        EngineMessage::InitializeUniverse { command_id, reply } => {
                            match engine.initialize_universe(command_id) {
                                Ok(initialization) => {
                                    live_clock.reanchor(engine.game_second()?, Instant::now());
                                    observers.clear();
                                    if event_sender
                                        .blocking_send(EngineEvent::UniverseReset)
                                        .is_err()
                                    {
                                        return Ok(());
                                    }
                                    let _ = reply.send(Ok(initialization));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::GetBbsConfiguration { bbs_id, reply } => {
                            match engine.bbs_configuration(bbs_id) {
                                Ok(configuration) => {
                                    let _ = reply.send(Ok(configuration));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::ConfigureBbs {
                            bbs_id,
                            command_id,
                            expected_revision,
                            settings,
                            reply,
                        } => {
                            match engine.configure_bbs(
                                bbs_id,
                                command_id,
                                expected_revision,
                                &settings,
                            ) {
                                Ok(configuration) => {
                                    let _ = reply.send(Ok(configuration));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::GetPlayerAccess { identity, reply } => {
                            match engine.player_access(&identity) {
                                Ok(access) => {
                                    let _ = reply.send(Ok(access));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::SetPlayerAccess {
                            bbs_id,
                            command_id,
                            player_id,
                            expected_revision,
                            state,
                            reason,
                            reply,
                        } => {
                            match engine.set_player_access(
                                bbs_id,
                                command_id,
                                player_id,
                                expected_revision,
                                state,
                                &reason,
                            ) {
                                Ok(result) => {
                                    if let SetPlayerAccessResult::Updated(access) = &result
                                        && access.state != PlayerAccessState::Active
                                        && event_sender
                                            .blocking_send(EngineEvent::PlayerAccessChanged(
                                                Box::new(access.clone()),
                                            ))
                                            .is_err()
                                    {
                                        return Ok(());
                                    }
                                    let _ = reply.send(Ok(result));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::GetStatus { reply } => {
                            match engine.operational_status() {
                                Ok(status) => {
                                    let _ = reply.send(Ok(status));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                        EngineMessage::LiveBackup {
                            backup_root,
                            label,
                            command_id,
                            reply,
                        } => {
                            match engine.live_backup(&backup_root, &label, &command_id) {
                                Ok(status) => {
                                    let _ = reply.send(Ok(status));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                }
                            }
                        }
                        EngineMessage::Shutdown { reply } => {
                            let _ = reply.send(());
                            return Ok(());
                        }
                        EngineMessage::IssueSysopDirective {
                            bbs_id,
                            command_id,
                            player_id,
                            kind,
                            reply,
                        } => {
                            match engine.issue_sysop_directive(
                                bbs_id,
                                command_id,
                                player_id,
                                kind,
                            ) {
                                Ok(directive) => {
                                    let _ = reply.send(Ok(directive));
                                }
                                Err(error @ EngineError::Store(
                                    StoreError::InvalidSysopDirective(_),
                                )) => {
                                    let _ = reply.send(Err(error.to_string()));
                                }
                                Err(error) => {
                                    let _ = reply.send(Err(error.to_string()));
                                    return Err(error);
                                }
                            }
                        }
                    }
                }
                Ok(())
            };
            if let Err(error) = run() {
                let _ = event_sender.blocking_send(EngineEvent::Fatal(error.to_string()));
            }
        })
        .expect("spawn authoritative engine thread");
    (
        EngineHandle {
            sender: input_sender,
        },
        event_receiver,
        join,
        ready_receiver,
    )
}

#[derive(Clone)]
struct ActiveSession {
    epoch: u64,
    outbound: mpsc::Sender<Vec<u8>>,
    replaced: watch::Sender<bool>,
    socket: Option<Arc<StdTcpStream>>,
}

#[derive(Default)]
struct Sessions {
    players: Mutex<HashMap<PlayerIdentity, ActiveSession>>,
}

impl Sessions {
    async fn active_count(&self) -> u32 {
        u32::try_from(self.players.lock().await.len()).unwrap_or(u32::MAX)
    }

    async fn replace_limited(
        &self,
        identity: PlayerIdentity,
        session: ActiveSession,
    ) -> Result<Option<ActiveSession>, ServerError> {
        let mut players = self.players.lock().await;
        if !players.contains_key(&identity) {
            if players.len() >= MAX_ACTIVE_GAME_SESSIONS
                || players
                    .keys()
                    .filter(|active| active.bbs_id == identity.bbs_id)
                    .count()
                    >= MAX_ACTIVE_GAME_SESSIONS_PER_BBS
            {
                return Err(ServerError::ConnectionLimit);
            }
        }
        Ok(players.insert(identity, session))
    }

    #[cfg(test)]
    async fn replace(
        &self,
        identity: PlayerIdentity,
        session: ActiveSession,
    ) -> Option<ActiveSession> {
        self.players.lock().await.insert(identity, session)
    }

    async fn remove_if_current(&self, identity: &PlayerIdentity, epoch: u64) {
        let mut players = self.players.lock().await;
        if players
            .get(identity)
            .is_some_and(|session| session.epoch == epoch)
        {
            players.remove(identity);
        }
    }

    async fn deliver(&self, delivery: &Delivery) -> DeliveryDisposition {
        let session = {
            let players = self.players.lock().await;
            let Some(session) = players.get(&delivery.identity) else {
                return DeliveryDisposition::NoSession;
            };
            session.clone()
        };
        if session.epoch > delivery.session_epoch {
            return DeliveryDisposition::Obsolete;
        }
        if session.epoch != delivery.session_epoch {
            return DeliveryDisposition::NoSession;
        }
        let frame = match encode_response(
            delivery.request_id,
            delivery.session_epoch,
            &delivery.outcome,
        ) {
            Ok(frame) => frame,
            Err(_) => return DeliveryDisposition::NoSession,
        };
        if session.outbound.send(frame).await.is_ok() {
            DeliveryDisposition::Enqueued
        } else {
            DeliveryDisposition::NoSession
        }
    }

    async fn phase_changed(&self, transition: &PlayerTravelTransition) {
        let session = {
            let players = self.players.lock().await;
            players.get(&transition.identity).cloned()
        };
        let Some(session) = session else {
            return;
        };
        let Ok(frame) = encode_phase_changed(session.epoch, transition) else {
            return;
        };
        if session.outbound.try_send(frame).is_err() {
            let _ = session.replaced.send(true);
            if let Some(socket) = session.socket {
                let _ = socket.shutdown(Shutdown::Both);
            }
            self.remove_if_current(&transition.identity, session.epoch)
                .await;
        }
    }

    async fn checkpoint_ready(
        &self,
        identity: &PlayerIdentity,
        committed_sequence: u64,
        checkpoint: &wire::CheckpointSnapshot,
    ) {
        let session = {
            let players = self.players.lock().await;
            players.get(identity).cloned()
        };
        if let Some(session) = session {
            if let Ok(frame) =
                encode_checkpoint_ready(session.epoch, committed_sequence, checkpoint)
            {
                let _ = session.outbound.try_send(frame);
            }
        }
    }
    async fn encounter_ready(
        &self,
        identity: &PlayerIdentity,
        committed_sequence: u64,
        encounter: &wire::EncounterSnapshot,
    ) {
        let session = {
            let players = self.players.lock().await;
            players.get(identity).cloned()
        };
        if let Some(session) = session {
            if let Ok(frame) = encode_encounter_ready(session.epoch, committed_sequence, encounter)
            {
                let _ = session.outbound.try_send(frame);
            }
        }
    }

    async fn traffic_snapshot(
        &self,
        identity: &PlayerIdentity,
        committed_sequence: u64,
        snapshot: &TrafficSnapshot,
    ) {
        let session = {
            let players = self.players.lock().await;
            players.get(identity).cloned()
        };
        let Some(session) = session else {
            return;
        };
        if let Ok(frame) = encode_traffic_snapshot(session.epoch, committed_sequence, snapshot) {
            let _ = session.outbound.try_send(frame);
        }
    }

    async fn traffic_movement(
        &self,
        identity: &PlayerIdentity,
        committed_sequence: u64,
        system_id: u64,
        observed_second: u64,
        contact: &TrafficContact,
    ) {
        let session = {
            let players = self.players.lock().await;
            players.get(identity).cloned()
        };
        let Some(session) = session else {
            return;
        };
        if let Ok(frame) = encode_traffic_movement(
            session.epoch,
            committed_sequence,
            system_id,
            observed_second,
            contact,
        ) {
            let _ = session.outbound.try_send(frame);
        }
    }

    async fn close_all_for_universe_reset(&self) {
        let sessions = {
            let mut players = self.players.lock().await;
            players
                .drain()
                .map(|(_, session)| session)
                .collect::<Vec<_>>()
        };
        for session in sessions {
            let _ = session.replaced.send(true);
            if let Some(socket) = session.socket {
                let _ = socket.shutdown(Shutdown::Both);
            }
        }
    }

    async fn close_all_for_shutdown(&self) {
        let sessions = {
            let mut players = self.players.lock().await;
            players
                .drain()
                .map(|(_, session)| session)
                .collect::<Vec<_>>()
        };
        for session in sessions {
            if let Ok(frame) = encode_server_stopping(session.epoch) {
                let _ = session.outbound.send(frame).await;
            }
            let _ = session.replaced.send(true);
        }
    }

    async fn close_for_access_change(&self, access: &PlayerAccessRecord) {
        let session = self.players.lock().await.remove(&access.identity);
        let Some(session) = session else {
            return;
        };
        let state = match access.state {
            PlayerAccessState::Active => "active",
            PlayerAccessState::Suspended => "suspended",
            PlayerAccessState::Removed => "permanently removed",
        };
        let reason = if access.reason.is_empty() {
            format!("player access {state}")
        } else {
            format!("player access {state}: {}", access.reason)
        };
        if let Ok(frame) = encode_close(session.epoch, &reason) {
            let _ = session.outbound.try_send(frame);
        }
        let _ = session.replaced.send(true);
        if let Some(socket) = session.socket {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
}

enum DeliveryDisposition {
    Enqueued,
    Obsolete,
    NoSession,
}

#[derive(Clone, Copy)]
enum ListenerRole {
    Game,
    Administrator,
    Sysop,
}

enum AcceptedConnection {
    Game(TcpStream, SocketAddr),
    Administrator(TcpStream, SocketAddr),
    Sysop(TcpStream, SocketAddr),
}

struct AcceptTasks(Vec<JoinHandle<()>>);

impl Drop for AcceptTasks {
    fn drop(&mut self) {
        for task in &self.0 {
            task.abort();
        }
    }
}

fn bind_listener(address: SocketAddr) -> io::Result<TcpListener> {
    let socket = Socket::new(
        Domain::for_address(address),
        Type::STREAM,
        Some(Protocol::TCP),
    )?;
    if address.is_ipv6() {
        socket.set_only_v6(true)?;
    }
    socket.bind(&address.into())?;
    socket.listen(1024)?;
    socket.set_nonblocking(true)?;
    TcpListener::from_std(socket.into())
}

fn bind_listeners(
    addresses: &[SocketAddr],
    description: &'static str,
) -> Result<Vec<TcpListener>, ServerError> {
    if addresses.is_empty() {
        return Err(ServerError::NoListenerAddresses(description));
    }
    addresses
        .iter()
        .copied()
        .map(bind_listener)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ServerError::from)
}

fn listener_address_list(listeners: &[TcpListener]) -> io::Result<String> {
    listeners
        .iter()
        .map(|listener| listener.local_addr().map(|address| address.to_string()))
        .collect::<Result<Vec<_>, _>>()
        .map(|addresses| addresses.join(", "))
}

fn spawn_accept_tasks(
    listeners: Vec<TcpListener>,
    role: ListenerRole,
    sender: &mpsc::Sender<Result<AcceptedConnection, io::Error>>,
    tasks: &mut Vec<JoinHandle<()>>,
) {
    for listener in listeners {
        let sender = sender.clone();
        tasks.push(tokio::spawn(async move {
            loop {
                let result = listener.accept().await.map(|(socket, peer)| match role {
                    ListenerRole::Game => AcceptedConnection::Game(socket, peer),
                    ListenerRole::Administrator => AcceptedConnection::Administrator(socket, peer),
                    ListenerRole::Sysop => AcceptedConnection::Sysop(socket, peer),
                });
                let failed = result.is_err();
                if sender.send(result).await.is_err() || failed {
                    break;
                }
            }
        }));
    }
}

pub async fn run(
    game_address: SocketAddr,
    admin_address: SocketAddr,
    sysop_address: SocketAddr,
    data_path: PathBuf,
    admin_tls: AdminTlsConfig,
) -> Result<(), ServerError> {
    run_on_addresses(
        vec![game_address],
        vec![admin_address],
        vec![sysop_address],
        data_path,
        admin_tls,
    )
    .await
}

pub async fn run_on_addresses(
    game_addresses: Vec<SocketAddr>,
    admin_addresses: Vec<SocketAddr>,
    sysop_addresses: Vec<SocketAddr>,
    data_path: PathBuf,
    admin_tls: AdminTlsConfig,
) -> Result<(), ServerError> {
    if admin_addresses
        .iter()
        .any(|address| !address.ip().is_loopback())
    {
        return Err(ServerError::AdminNotLoopback);
    }
    let game_listeners = bind_listeners(&game_addresses, "game")?;
    let admin_listeners = bind_listeners(&admin_addresses, "administrator")?;
    let sysop_listeners = bind_listeners(&sysop_addresses, "sysop")?;
    let game_listener_text = listener_address_list(&game_listeners)?;
    let admin_listener_text = listener_address_list(&admin_listeners)?;
    let sysop_listener_text = listener_address_list(&sysop_listeners)?;
    let bbs_registry = BbsRegistry::default();
    let (engine, mut engine_events, engine_thread, engine_ready) =
        spawn_engine(data_path, bbs_registry.clone());
    tokio::task::spawn_blocking(move || engine_ready.recv())
        .await
        .map_err(|_| ServerError::EngineStopped)?
        .map_err(|_| ServerError::EngineStopped)?;
    eprintln!(
        "Cepheus Trader game listeners on {game_listener_text}; administrator listeners on \
         {admin_listener_text}; sysop listeners on {sysop_listener_text}"
    );
    let sessions = Arc::new(Sessions::default());
    let pending_game_authentications = Arc::new(Semaphore::new(MAX_PENDING_GAME_AUTHENTICATIONS));
    let clock_engine = engine.clone();
    tokio::spawn(async move {
        let start = tokio::time::Instant::now() + LIVE_CLOCK_PULSE;
        let mut interval = tokio::time::interval_at(start, LIVE_CLOCK_PULSE);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            match clock_engine.sender.try_send(EngineMessage::ClockPulse {
                sampled_at: Instant::now(),
            }) {
                Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => {}
                Err(mpsc::error::TrySendError::Closed(_)) => break,
            }
        }
    });
    let dispatcher_sessions = Arc::clone(&sessions);
    let dispatcher_engine = engine.clone();
    let (fatal_sender, mut fatal_receiver) = oneshot::channel();
    tokio::spawn(async move {
        while let Some(event) = engine_events.recv().await {
            match event {
                EngineEvent::Delivery(delivery) => {
                    match dispatcher_sessions.deliver(&delivery).await {
                        DeliveryDisposition::Enqueued | DeliveryDisposition::Obsolete => {
                            if dispatcher_engine
                                .sender
                                .send(EngineMessage::AcknowledgeOutbox(delivery.outbox_sequence))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        DeliveryDisposition::NoSession => {}
                    }
                }
                EngineEvent::PhaseChanged(transition) => {
                    dispatcher_sessions.phase_changed(&transition).await;
                }
                EngineEvent::CheckpointReady {
                    identity,
                    committed_sequence,
                    checkpoint,
                } => {
                    dispatcher_sessions
                        .checkpoint_ready(&identity, committed_sequence, &checkpoint)
                        .await
                }
                EngineEvent::EncounterReady {
                    identity,
                    committed_sequence,
                    encounter,
                } => {
                    dispatcher_sessions
                        .encounter_ready(&identity, committed_sequence, &encounter)
                        .await
                }
                EngineEvent::TrafficSnapshot {
                    identity,
                    committed_sequence,
                    snapshot,
                } => {
                    dispatcher_sessions
                        .traffic_snapshot(&identity, committed_sequence, &snapshot)
                        .await;
                }
                EngineEvent::TrafficMovement {
                    identity,
                    committed_sequence,
                    system_id,
                    observed_second,
                    contact,
                } => {
                    dispatcher_sessions
                        .traffic_movement(
                            &identity,
                            committed_sequence,
                            system_id,
                            observed_second,
                            &contact,
                        )
                        .await;
                }
                EngineEvent::UniverseReset => {
                    dispatcher_sessions.close_all_for_universe_reset().await;
                }
                EngineEvent::PlayerAccessChanged(access) => {
                    dispatcher_sessions.close_for_access_change(&access).await;
                }
                EngineEvent::Fatal(error) => {
                    let _ = fatal_sender.send(error);
                    return;
                }
            }
        }
    });

    let (incoming_sender, mut incoming_receiver) = mpsc::channel(CONNECTION_QUEUE_DEPTH);
    let mut accept_tasks = Vec::new();
    spawn_accept_tasks(
        game_listeners,
        ListenerRole::Game,
        &incoming_sender,
        &mut accept_tasks,
    );
    spawn_accept_tasks(
        admin_listeners,
        ListenerRole::Administrator,
        &incoming_sender,
        &mut accept_tasks,
    );
    spawn_accept_tasks(
        sysop_listeners,
        ListenerRole::Sysop,
        &incoming_sender,
        &mut accept_tasks,
    );
    drop(incoming_sender);
    let _accept_tasks = AcceptTasks(accept_tasks);

    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            result = incoming_receiver.recv() => {
                let connection = result.ok_or(ServerError::ListenerStopped)??;
                match connection {
                    AcceptedConnection::Game(socket, peer) => {
                        let connection_engine = engine.clone();
                        let connection_sessions = Arc::clone(&sessions);
                        let connection_registry = bbs_registry.clone();
                        let Ok(authentication_permit) = Arc::clone(
                            &pending_game_authentications,
                        )
                        .try_acquire_owned()
                        else {
                            let _ = socket
                                .into_std()
                                .and_then(|socket| socket.shutdown(Shutdown::Both));
                            continue;
                        };
                        tokio::spawn(async move {
                            if let Err(error) = handle_connection(
                                socket,
                                connection_engine,
                                connection_sessions,
                                connection_registry,
                                authentication_permit,
                            )
                            .await
                            {
                                eprintln!("connection {peer} closed: {error}");
                            }
                        });
                    }
                    AcceptedConnection::Administrator(socket, peer) => {
                        let connection_engine = engine.clone();
                        let connection_tls = admin_tls.clone();
                        let connection_sessions = Arc::clone(&sessions);
                        tokio::spawn(async move {
                            if let Err(error) = handle_admin_connection(
                                socket,
                                connection_engine,
                                connection_tls,
                                connection_sessions,
                            )
                            .await
                            {
                                eprintln!("administrator connection {peer} closed: {error}");
                            }
                        });
                    }
                    AcceptedConnection::Sysop(socket, peer) => {
                        let connection_engine = engine.clone();
                        let connection_registry = bbs_registry.clone();
                        tokio::spawn(async move {
                            if let Err(error) = handle_sysop_connection(
                                socket,
                                connection_engine,
                                connection_registry,
                            )
                            .await
                            {
                                eprintln!("sysop connection {peer} closed: {error}");
                            }
                        });
                    }
                }
            }
            result = &mut fatal_receiver => {
                let error = result.unwrap_or_else(|_| "engine event stream stopped".into());
                drop(engine);
                let _ = engine_thread.join();
                return Err(ServerError::Engine(error));
            }
            result = &mut shutdown => {
                result?;
                eprintln!("shutdown requested");
                sessions.close_all_for_shutdown().await;
                engine.shutdown().await;
                drop(engine);
                tokio::task::spawn_blocking(move || engine_thread.join())
                    .await
                    .map_err(|_| ServerError::EngineStopped)?
                    .map_err(|_| ServerError::EngineStopped)?;
                return Ok(());
            }
        }
    }
}

async fn handle_connection(
    socket: TcpStream,
    engine: EngineHandle,
    sessions: Arc<Sessions>,
    bbs_registry: BbsRegistry,
    authentication_permit: tokio::sync::OwnedSemaphorePermit,
) -> Result<(), ServerError> {
    socket.set_nodelay(true)?;
    let socket = socket.into_std()?;
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    socket.set_write_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    let socket = Arc::new(socket);
    let handshake_socket = Arc::clone(&socket);
    let credentials: Vec<_> = bbs_registry
        .tls_credentials()
        .into_iter()
        .map(|(identity, key)| PskCredential {
            identity: identity.into_bytes(),
            key,
        })
        .collect();
    let tls = tokio::task::spawn_blocking(move || {
        TlsServer::handshake_many(&*handshake_socket, &credentials)
    })
    .await
    .map_err(|_| ServerError::TlsWorkerStopped)?
    .map_err(|error| ServerError::Tls(error.to_string()))?;
    drop(authentication_permit);
    socket.set_read_timeout(None)?;
    socket.set_write_timeout(None)?;
    let tls = Arc::new(tls);
    let (outbound, mut outbound_receiver) = mpsc::channel::<Vec<u8>>(CONNECTION_QUEUE_DEPTH);
    let writer_tls = Arc::clone(&tls);
    let writer_task = tokio::spawn(async move {
        while let Some(frame) = outbound_receiver.recv().await {
            let tls = Arc::clone(&writer_tls);
            tokio::task::spawn_blocking(move || write_tls_frame(&tls, &frame))
                .await
                .map_err(|_| io::Error::other("TLS writer stopped"))??;
        }
        Ok::<(), io::Error>(())
    });

    let hello_frame = read_tls_frame_async(Arc::clone(&tls)).await?;
    let (client_version, hello) = decode_client_hello_with_version(&hello_frame)?;
    if client_version != PROTOCOL_VERSION {
        let reason = format!(
            "CT-RPC version {client_version} is no longer supported; upgrade your Cepheus Trader client (this server requires version {PROTOCOL_VERSION})"
        );
        let _ = outbound
            .send(encode_close_for_version(client_version, 0, &reason)?)
            .await;
        drop(outbound);
        let _ = writer_task.await;
        let _ = socket.shutdown(Shutdown::Both);
        return Ok(());
    }
    if tls
        .identity()
        .map_err(|error| ServerError::Tls(error.to_string()))?
        != hello.identity.bbs_id.to_string().as_bytes()
    {
        return Err(ServerError::BbsIdentityMismatch);
    }
    let opening = match engine.issue_session(hello.identity.clone()).await {
        Ok(opening) => opening,
        Err(ServerError::Engine(message)) if message.starts_with("player access denied:") => {
            let reason = message
                .strip_prefix("player access denied: ")
                .unwrap_or(&message);
            let _ = outbound.send(encode_close(0, reason)?).await;
            drop(outbound);
            let _ = writer_task.await;
            let _ = socket.shutdown(Shutdown::Both);
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    let epoch = opening.epoch;
    let (replaced_sender, mut replaced_receiver) = watch::channel(false);
    let active = ActiveSession {
        epoch,
        outbound: outbound.clone(),
        replaced: replaced_sender,
        socket: Some(Arc::clone(&socket)),
    };
    if let Some(previous) = sessions
        .replace_limited(hello.identity.clone(), active)
        .await?
    {
        if let Ok(event) = encode_session_replaced(previous.epoch) {
            let _ = previous.outbound.try_send(event);
        }
        let _ = previous.replaced.send(true);
        if let Some(socket) = previous.socket {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
    outbound
        .send(encode_server_hello(
            &hello.identity,
            epoch,
            opening.committed_sequence,
            opening.phase,
        )?)
        .await
        .map_err(|_| {
            ServerError::Io(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "connection writer stopped",
            ))
        })?;
    if let Some(snapshot) = opening.traffic_snapshot {
        outbound
            .send(encode_traffic_snapshot(
                epoch,
                opening.committed_sequence,
                &snapshot,
            )?)
            .await
            .map_err(|_| {
                ServerError::Io(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "connection writer stopped",
                ))
            })?;
    }
    if let Some(checkpoint) = opening.checkpoint {
        outbound
            .send(encode_checkpoint_ready(
                epoch,
                opening.committed_sequence,
                &checkpoint,
            )?)
            .await
            .map_err(|_| {
                ServerError::Io(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "connection writer stopped",
                ))
            })?;
    }
    if let Some(encounter) = opening.encounter {
        outbound
            .send(encode_encounter_ready(
                epoch,
                opening.committed_sequence,
                &encounter,
            )?)
            .await
            .map_err(|_| {
                ServerError::Io(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "connection writer stopped",
                ))
            })?;
    }

    let result = loop {
        let frame = tokio::select! {
            changed = replaced_receiver.changed() => {
                if changed.is_ok() && *replaced_receiver.borrow() {
                    break Ok(());
                }
                continue;
            }
            result = read_tls_frame_async(Arc::clone(&tls)) => {
                match result {
                    Ok(frame) => frame,
                    Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break Ok(()),
                    Err(error) => break Err(ServerError::Io(error)),
                }
            }
        };
        if decode_close(&frame)?.is_some() {
            break Ok(());
        }
        match decode_request(&frame) {
            Ok(request) if request.session_epoch == epoch => {
                if engine
                    .sender
                    .send(EngineMessage::Submit {
                        identity: hello.identity.clone(),
                        request,
                    })
                    .await
                    .is_err()
                {
                    break Err(ServerError::EngineStopped);
                }
            }
            Ok(_) => {
                let _ = outbound
                    .send(encode_close(epoch, "request used the wrong session epoch")?)
                    .await;
                break Ok(());
            }
            Err(error) => {
                let _ = outbound
                    .send(encode_close(epoch, &format!("malformed request: {error}"))?)
                    .await;
                break Ok(());
            }
        }
    };
    sessions.remove_if_current(&hello.identity, epoch).await;
    engine.close_session(hello.identity, epoch).await;
    let _ = socket.shutdown(Shutdown::Both);
    drop(outbound);
    let _ = writer_task.await;
    result
}

async fn handle_admin_connection(
    socket: TcpStream,
    engine: EngineHandle,
    tls_config: AdminTlsConfig,
    sessions: Arc<Sessions>,
) -> Result<(), ServerError> {
    socket.set_nodelay(true)?;
    let socket = socket.into_std()?;
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    socket.set_write_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    let socket = Arc::new(socket);
    let handshake_socket = Arc::clone(&socket);
    let key = Arc::clone(&tls_config.key);
    let tls = tokio::task::spawn_blocking(move || {
        TlsServer::handshake(&*handshake_socket, b"admin", &key)
    })
    .await
    .map_err(|_| ServerError::TlsWorkerStopped)?
    .map_err(|error| ServerError::Tls(error.to_string()))?;
    socket.set_read_timeout(None)?;
    socket.set_write_timeout(None)?;
    let tls = Arc::new(tls);

    loop {
        let frame = match read_tls_frame_async(Arc::clone(&tls)).await {
            Ok(frame) => frame,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(ServerError::Io(error)),
        };
        let request = match admin_wire::decode_request(&frame) {
            Ok(request) => request,
            Err(error) => {
                let close = admin_wire::encode_close(&format!("invalid request: {error}"))?;
                let writer = Arc::clone(&tls);
                let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
                return Ok(());
            }
        };
        let response = match &request.command {
            admin_wire::AdminCommand::AddBbs { name } => {
                match engine.add_bbs(request.command_id, name.clone()).await {
                    Ok(credential) => admin_wire::encode_bbs_added(&request, &credential)?,
                    Err(ServerError::Engine(message)) => {
                        admin_wire::encode_invalid_request(&request, &message)?
                    }
                    Err(error) => return Err(error),
                }
            }
            admin_wire::AdminCommand::InitializeUniverse => {
                let initialization = engine.initialize_universe(request.command_id).await?;
                admin_wire::encode_universe_initialized(&request, &initialization)?
            }
            admin_wire::AdminCommand::Status => {
                let status = engine.status().await?;
                admin_wire::encode_status(&request, &status, sessions.active_count().await)?
            }
            admin_wire::AdminCommand::LiveBackup { label } => {
                match engine
                    .live_backup(
                        (*tls_config.backup_root).clone(),
                        label.clone(),
                        request.command_id,
                    )
                    .await
                {
                    Ok(status) => admin_wire::encode_backup_complete(&request, label, &status)?,
                    Err(ServerError::Engine(message)) => {
                        admin_wire::encode_invalid_request(&request, &message)?
                    }
                    Err(error) => return Err(error),
                }
            }
        };
        let writer = Arc::clone(&tls);
        tokio::task::spawn_blocking(move || write_tls_frame(&writer, &response))
            .await
            .map_err(|_| ServerError::TlsWorkerStopped)??;
    }
}

async fn handle_sysop_connection(
    socket: TcpStream,
    engine: EngineHandle,
    bbs_registry: BbsRegistry,
) -> Result<(), ServerError> {
    socket.set_nodelay(true)?;
    let socket = socket.into_std()?;
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    socket.set_write_timeout(Some(AUTHENTICATION_TIMEOUT))?;
    let socket = Arc::new(socket);
    let handshake_socket = Arc::clone(&socket);
    let credentials: Vec<_> = bbs_registry
        .tls_credentials()
        .into_iter()
        .map(|(identity, key)| PskCredential {
            identity: identity.into_bytes(),
            key,
        })
        .collect();
    let tls = tokio::task::spawn_blocking(move || {
        TlsServer::handshake_many(&*handshake_socket, &credentials)
    })
    .await
    .map_err(|_| ServerError::TlsWorkerStopped)?
    .map_err(|error| ServerError::Tls(error.to_string()))?;
    socket.set_read_timeout(None)?;
    socket.set_write_timeout(None)?;
    let identity = tls
        .identity()
        .map_err(|error| ServerError::Tls(error.to_string()))?;
    let identity =
        std::str::from_utf8(&identity).map_err(|_| ServerError::InvalidBbsPskIdentity)?;
    let bbs_id = identity
        .parse::<u32>()
        .map_err(|_| ServerError::InvalidBbsPskIdentity)?;
    if bbs_id == 0 || bbs_id.to_string() != identity {
        return Err(ServerError::InvalidBbsPskIdentity);
    }
    let tls = Arc::new(tls);

    loop {
        let frame = match read_tls_frame_async(Arc::clone(&tls)).await {
            Ok(frame) => frame,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(error) => return Err(ServerError::Io(error)),
        };
        let request = match sysop_wire::decode_request(&frame) {
            Ok(request) => request,
            Err(error) => {
                let close = sysop_wire::encode_close(&format!("invalid request: {error}"))?;
                let writer = Arc::clone(&tls);
                let _ = tokio::task::spawn_blocking(move || write_tls_frame(&writer, &close)).await;
                return Ok(());
            }
        };
        let response = match &request.command {
            sysop_wire::SysopCommand::GetConfiguration => {
                let configuration = engine.bbs_configuration(bbs_id).await?;
                sysop_wire::encode_configuration(&request, &configuration)?
            }
            sysop_wire::SysopCommand::SetConfiguration {
                expected_revision,
                settings,
            } => match engine
                .configure_bbs(
                    bbs_id,
                    request.command_id,
                    *expected_revision,
                    settings.clone(),
                )
                .await?
            {
                ConfigureBbsResult::Updated(configuration) => {
                    sysop_wire::encode_configuration(&request, &configuration)?
                }
                ConfigureBbsResult::Stale(configuration) => {
                    sysop_wire::encode_stale_revision(&request, &configuration)?
                }
                ConfigureBbsResult::NoEligibleSite(configuration) => {
                    sysop_wire::encode_no_eligible_site(&request, &configuration)?
                }
            },
            sysop_wire::SysopCommand::GetPlayerAccess { player_id } => {
                let access = engine
                    .player_access(PlayerIdentity {
                        bbs_id,
                        player_id: *player_id,
                    })
                    .await?;
                sysop_wire::encode_player_access(&request, &access)?
            }
            sysop_wire::SysopCommand::SetPlayerAccess {
                player_id,
                expected_revision,
                state,
                reason,
            } => match engine
                .set_player_access(
                    bbs_id,
                    request.command_id,
                    *player_id,
                    *expected_revision,
                    *state,
                    reason.clone(),
                )
                .await?
            {
                SetPlayerAccessResult::Updated(access) => {
                    sysop_wire::encode_player_access(&request, &access)?
                }
                SetPlayerAccessResult::Stale(access) => {
                    sysop_wire::encode_player_access_stale(&request, &access)?
                }
                SetPlayerAccessResult::PermanentlyRemoved(access) => {
                    sysop_wire::encode_player_permanently_removed(&request, &access)?
                }
            },
            sysop_wire::SysopCommand::IssueDirective { player_id, kind } => {
                match engine
                    .issue_sysop_directive(bbs_id, request.command_id, *player_id, *kind)
                    .await
                {
                    Ok(directive) => sysop_wire::encode_directive_issued(&request, &directive)?,
                    Err(ServerError::Engine(message)) => {
                        sysop_wire::encode_invalid_request(&request, &message)?
                    }
                    Err(error) => return Err(error),
                }
            }
        };
        let writer = Arc::clone(&tls);
        tokio::task::spawn_blocking(move || write_tls_frame(&writer, &response))
            .await
            .map_err(|_| ServerError::TlsWorkerStopped)??;
    }
}

async fn read_tls_frame_async(tls: Arc<TlsServer>) -> io::Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || read_tls_frame(&tls))
        .await
        .map_err(|_| io::Error::other("TLS reader stopped"))?
}

fn read_tls_frame(tls: &TlsServer) -> io::Result<Vec<u8>> {
    let mut length = [0u8; 4];
    read_tls_exact(tls, &mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid CT-RPC frame length",
        ));
    }
    let mut bytes = vec![0; length];
    read_tls_exact(tls, &mut bytes)?;
    Ok(bytes)
}

fn read_tls_exact(tls: &TlsServer, mut data: &mut [u8]) -> io::Result<()> {
    while !data.is_empty() {
        let count = tls
            .receive(data)
            .map_err(|error| io::Error::new(io::ErrorKind::UnexpectedEof, error))?;
        data = &mut data[count..];
    }
    Ok(())
}

fn write_tls_frame(tls: &TlsServer, frame: &[u8]) -> io::Result<()> {
    let length = u32::try_from(frame.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "CT-RPC frame too large"))?;
    write_tls_all(tls, &length.to_be_bytes())?;
    write_tls_all(tls, frame)
}

fn write_tls_all(tls: &TlsServer, mut data: &[u8]) -> io::Result<()> {
    while !data.is_empty() {
        let count = tls
            .send(data)
            .map_err(|error| io::Error::new(io::ErrorKind::BrokenPipe, error))?;
        data = &data[count..];
    }
    Ok(())
}

#[cfg(test)]
async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Vec<u8>> {
    let length = reader.read_u32().await? as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid CT-RPC frame length",
        ));
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes).await?;
    Ok(bytes)
}

#[cfg(test)]
async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, frame: &[u8]) -> io::Result<()> {
    let length = u32::try_from(frame.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "CT-RPC frame too large"))?;
    writer.write_u32(length).await?;
    writer.write_all(frame).await?;
    writer.flush().await
}

#[cfg(test)]
mod tests {
    use tokio::io::{duplex, split};

    use super::*;

    #[tokio::test]
    async fn framing_round_trips() {
        let (left, right) = duplex(1024);
        let (_, mut left_writer) = split(left);
        let (mut right_reader, _) = split(right);
        let expected = vec![1, 2, 3, 4, 5];
        write_frame(&mut left_writer, &expected).await.unwrap();
        assert_eq!(read_frame(&mut right_reader).await.unwrap(), expected);
    }

    #[tokio::test]
    async fn framing_rejects_oversized_input_without_allocating_it() {
        let (left, right) = duplex(1024);
        let (_, mut left_writer) = split(left);
        let (mut right_reader, _) = split(right);
        left_writer
            .write_u32((MAX_FRAME_BYTES + 1) as u32)
            .await
            .unwrap();
        let error = read_frame(&mut right_reader).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn multiple_listeners_feed_one_accept_queue() {
        let addresses = [
            "127.0.0.1:0".parse().unwrap(),
            "127.0.0.1:0".parse().unwrap(),
        ];
        let listeners = bind_listeners(&addresses, "test").unwrap();
        let bound = listeners
            .iter()
            .map(|listener| listener.local_addr().unwrap())
            .collect::<Vec<_>>();
        let (sender, mut receiver) = mpsc::channel(CONNECTION_QUEUE_DEPTH);
        let mut tasks = Vec::new();
        spawn_accept_tasks(listeners, ListenerRole::Game, &sender, &mut tasks);
        drop(sender);
        let _tasks = AcceptTasks(tasks);

        let mut clients = Vec::new();
        for address in bound {
            clients.push(TcpStream::connect(address).await.unwrap());
        }
        for _ in 0..clients.len() {
            let accepted = tokio::time::timeout(Duration::from_secs(1), receiver.recv())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert!(matches!(accepted, AcceptedConnection::Game(_, _)));
        }
    }

    #[tokio::test]
    async fn ipv6_listener_does_not_claim_the_ipv4_port() {
        let address = "[::1]:0".parse().unwrap();
        let ipv6 = match bind_listener(address) {
            Ok(listener) => listener,
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::AddrNotAvailable | io::ErrorKind::Unsupported
                ) =>
            {
                return;
            }
            Err(error) => panic!("cannot create IPv6 test listener: {error}"),
        };
        let port = ipv6.local_addr().unwrap().port();
        let ipv4 = bind_listener(SocketAddr::from(([127, 0, 0, 1], port))).unwrap();
        assert_eq!(ipv4.local_addr().unwrap().port(), port);
    }

    #[tokio::test]
    async fn replacement_keeps_new_session_when_old_session_exits() {
        let sessions = Sessions::default();
        let identity = PlayerIdentity {
            bbs_id: 17,
            player_id: 42,
        };
        let (old_outbound, _) = mpsc::channel(1);
        let (old_replaced, _) = watch::channel(false);
        assert!(
            sessions
                .replace(
                    identity.clone(),
                    ActiveSession {
                        epoch: 1,
                        outbound: old_outbound,
                        replaced: old_replaced,
                        socket: None,
                    },
                )
                .await
                .is_none()
        );

        let (new_outbound, _) = mpsc::channel(1);
        let (new_replaced, _) = watch::channel(false);
        let previous = sessions
            .replace(
                identity.clone(),
                ActiveSession {
                    epoch: 2,
                    outbound: new_outbound,
                    replaced: new_replaced,
                    socket: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(previous.epoch, 1);

        sessions.remove_if_current(&identity, 1).await;
        assert_eq!(sessions.players.lock().await[&identity].epoch, 2);
    }

    #[tokio::test]
    async fn same_local_player_on_different_bbss_has_distinct_sessions() {
        let sessions = Sessions::default();
        let first = PlayerIdentity {
            bbs_id: 17,
            player_id: 42,
        };
        let second = PlayerIdentity {
            bbs_id: 23,
            player_id: 42,
        };
        for identity in [first.clone(), second.clone()] {
            let (outbound, _) = mpsc::channel(1);
            let (replaced, _) = watch::channel(false);
            assert!(
                sessions
                    .replace(
                        identity,
                        ActiveSession {
                            epoch: 1,
                            outbound,
                            replaced,
                            socket: None,
                        },
                    )
                    .await
                    .is_none()
            );
        }
        let players = sessions.players.lock().await;
        assert!(players.contains_key(&first));
        assert!(players.contains_key(&second));
    }
}
