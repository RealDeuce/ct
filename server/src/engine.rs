//! Single-owner authoritative engine loop.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use thiserror::Error;

use crate::simulation::SimulationReport;
use crate::store::{
    BbsConfiguration, BbsCredential, BbsPlacementSeed, BbsSettings, ConfigureBbsResult, Delivery,
    LeagueCredential, LeagueStatus, OperationalStatus, PlayerAccessRecord, PlayerAccessState,
    PlayerTravelTransition, QueuedCommand, SetLeagueMemberAccessResult, SetLeagueNameResult,
    SetPlayerAccessResult, SimulationAdvance, Store, StoreError, SysopDirectiveKind,
    SysopDirectiveRecord,
};
use crate::universe::{INITIAL_SYSTEMS, UniverseInitialization};
use crate::wire::{COMMAND_ID_BYTES, CommandRequest, PlayerIdentity, PlayerPhase};

#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("cryptographic random source failed: {0}")]
    Random(#[from] getrandom::Error),
    #[error("player access denied: {0}")]
    PlayerAccessDenied(String),
}

pub struct Engine {
    store: Store,
    bbs_registry: BbsRegistry,
    league_registry: LeagueRegistry,
}

#[derive(Default)]
pub struct EngineBatch {
    pub deliveries: Vec<Delivery>,
    pub player_transitions: Vec<PlayerTravelTransition>,
}

#[derive(Clone, Default)]
pub struct BbsRegistry {
    credentials: Arc<RwLock<HashMap<u32, [u8; 32]>>>,
}

impl BbsRegistry {
    pub fn tls_credentials(&self) -> Vec<(String, Vec<u8>)> {
        self.credentials
            .read()
            .expect("BBS registry lock poisoned")
            .iter()
            .map(|(bbs_id, psk)| (bbs_id.to_string(), psk.to_vec()))
            .collect()
    }

    pub fn insert(&self, credential: &BbsCredential) {
        self.credentials
            .write()
            .expect("BBS registry lock poisoned")
            .insert(credential.bbs_id, credential.psk);
    }

    pub fn remove(&self, bbs_id: u32) {
        self.credentials
            .write()
            .expect("BBS registry lock poisoned")
            .remove(&bbs_id);
    }

    pub fn contains(&self, bbs_id: u32) -> bool {
        self.credentials
            .read()
            .expect("BBS registry lock poisoned")
            .contains_key(&bbs_id)
    }
}

#[derive(Clone, Default)]
pub struct LeagueRegistry {
    credentials: Arc<RwLock<HashMap<u32, [u8; 32]>>>,
}

impl LeagueRegistry {
    pub fn tls_credentials(&self) -> Vec<(String, Vec<u8>)> {
        self.credentials
            .read()
            .expect("league registry lock poisoned")
            .iter()
            .map(|(league_id, psk)| (league_id.to_string(), psk.to_vec()))
            .collect()
    }

    fn insert(&self, credential: &LeagueCredential) {
        self.credentials
            .write()
            .expect("league registry lock poisoned")
            .insert(credential.league_id, credential.psk);
    }
}

impl Engine {
    pub fn open(path: impl AsRef<Path>, bbs_registry: BbsRegistry) -> Result<Self, EngineError> {
        Self::open_with_registries(path, bbs_registry, LeagueRegistry::default())
    }

    pub fn open_with_registries(
        path: impl AsRef<Path>,
        bbs_registry: BbsRegistry,
        league_registry: LeagueRegistry,
    ) -> Result<Self, EngineError> {
        let store = Store::open(path)?;
        for credential in store.bbs_credentials()? {
            if store.bbs_enabled(credential.bbs_id)? {
                bbs_registry.insert(&credential);
            }
        }
        for credential in store.league_credentials()? {
            league_registry.insert(&credential);
        }
        let engine = Self {
            store,
            bbs_registry,
            league_registry,
        };
        engine.reset_if_unique_cargo_destroyed()?;
        Ok(engine)
    }

    pub fn issue_session(
        &self,
        identity: &PlayerIdentity,
    ) -> Result<(u64, u64, PlayerPhase), EngineError> {
        if !self.bbs_registry.contains(identity.bbs_id) {
            return Err(EngineError::PlayerAccessDenied(
                "home BBS access is disabled".into(),
            ));
        }
        let access = self.store.player_access(identity)?;
        match access.state {
            PlayerAccessState::Active => Ok(self.store.issue_session_epoch(identity)?),
            PlayerAccessState::Suspended => Err(EngineError::PlayerAccessDenied(format!(
                "account suspended{}",
                if access.reason.is_empty() {
                    String::new()
                } else {
                    format!(": {}", access.reason)
                }
            ))),
            PlayerAccessState::Removed => Err(EngineError::PlayerAccessDenied(format!(
                "account permanently removed{}",
                if access.reason.is_empty() {
                    String::new()
                } else {
                    format!(": {}", access.reason)
                }
            ))),
        }
    }

    pub fn home_affiliation(
        &self,
        bbs_id: u32,
    ) -> Result<Option<crate::wire::InstitutionalAffiliation>, EngineError> {
        Ok(self.store.home_affiliation(bbs_id)?)
    }

    pub fn submit(
        &self,
        identity: PlayerIdentity,
        request: CommandRequest,
    ) -> Result<EngineBatch, EngineError> {
        // Close the preceding finite ingress batch and drain everything
        // already due at the current logical second before this command is
        // allowed to observe state.
        let mut batch = self.recover()?;
        self.store.enqueue(&QueuedCommand { identity, request })?;
        let following = self.recover()?;
        batch.deliveries.extend(following.deliveries);
        batch
            .player_transitions
            .extend(following.player_transitions);
        Ok(batch)
    }

    pub fn recover(&self) -> Result<EngineBatch, EngineError> {
        let mut deliveries = Vec::new();
        let mut player_transitions = Vec::new();
        while let Some(processed) = self.store.process_next_engine_input()? {
            if let Some(delivery) = processed.delivery {
                deliveries.push(delivery);
            }
            player_transitions.extend(processed.player_transitions);
            if self.reset_if_unique_cargo_destroyed()? {
                // Reset invalidates every delivery and transition gathered
                // from the replaced universe.
                deliveries.clear();
                player_transitions.clear();
                break;
            }
        }
        Ok(EngineBatch {
            deliveries,
            player_transitions,
        })
    }

    pub fn pending_outbox(&self) -> Result<Vec<Delivery>, EngineError> {
        Ok(self.store.pending_outbox()?)
    }

    pub fn pending_checkpoint(
        &self,
        identity: &PlayerIdentity,
    ) -> Result<Option<crate::wire::CheckpointSnapshot>, EngineError> {
        Ok(self.store.pending_checkpoint(identity)?)
    }
    pub fn pending_encounter(
        &self,
        identity: &PlayerIdentity,
    ) -> Result<Option<crate::wire::EncounterSnapshot>, EngineError> {
        Ok(self.store.pending_encounter(identity)?)
    }

    pub fn radio_unread_count(&self, identity: &PlayerIdentity) -> Result<(u64, u64), EngineError> {
        Ok(self.store.radio_unread_count(identity)?)
    }

    pub fn acknowledge_outbox(&self, sequence: u64) -> Result<bool, EngineError> {
        Ok(self.store.acknowledge_outbox(sequence)?)
    }

    pub fn add_bbs(
        &self,
        command_id: [u8; COMMAND_ID_BYTES],
        name: &str,
    ) -> Result<BbsCredential, EngineError> {
        let mut psk = [0; 32];
        getrandom::fill(&mut psk)?;
        let credential = self.store.add_bbs(&command_id, name, psk)?;
        self.bbs_registry.insert(&credential);
        Ok(credential)
    }

    pub fn add_league(
        &self,
        command_id: [u8; COMMAND_ID_BYTES],
    ) -> Result<LeagueCredential, EngineError> {
        let mut psk = [0; 32];
        getrandom::fill(&mut psk)?;
        let credential = self.store.add_league(&command_id, psk)?;
        self.league_registry.insert(&credential);
        Ok(credential)
    }

    pub fn league_status(&self, league_id: u32) -> Result<LeagueStatus, EngineError> {
        Ok(self.store.league_status(league_id)?)
    }

    pub fn set_league_name(
        &self,
        league_id: u32,
        command_id: [u8; COMMAND_ID_BYTES],
        expected_revision: u64,
        name: &str,
    ) -> Result<SetLeagueNameResult, EngineError> {
        Ok(self
            .store
            .set_league_name(league_id, &command_id, expected_revision, name)?)
    }

    pub fn add_league_bbs(
        &self,
        league_id: u32,
        command_id: [u8; COMMAND_ID_BYTES],
        name: &str,
    ) -> Result<BbsCredential, EngineError> {
        let mut psk = [0; 32];
        getrandom::fill(&mut psk)?;
        let credential = self
            .store
            .add_league_bbs(league_id, &command_id, name, psk)?;
        self.bbs_registry.insert(&credential);
        Ok(credential)
    }

    pub fn set_league_member_enabled(
        &self,
        league_id: u32,
        command_id: [u8; COMMAND_ID_BYTES],
        bbs_id: u32,
        expected_revision: u64,
        enabled: bool,
        reason: &str,
    ) -> Result<SetLeagueMemberAccessResult, EngineError> {
        let result = self.store.set_league_member_enabled(
            league_id,
            &command_id,
            bbs_id,
            expected_revision,
            enabled,
            reason,
        )?;
        if let SetLeagueMemberAccessResult::Updated(member) = &result {
            if member.enabled {
                if let Some(credential) = self
                    .store
                    .bbs_credentials()?
                    .into_iter()
                    .find(|credential| credential.bbs_id == bbs_id)
                {
                    self.bbs_registry.insert(&credential);
                }
            } else {
                self.bbs_registry.remove(bbs_id);
            }
        }
        Ok(result)
    }

    pub fn initialize_universe(
        &self,
        command_id: [u8; COMMAND_ID_BYTES],
    ) -> Result<UniverseInitialization, EngineError> {
        let mut universe_id = [0; 16];
        getrandom::fill(&mut universe_id)?;
        let mut system_seeds = vec![[0; 32]; INITIAL_SYSTEMS.len()];
        for seed in &mut system_seeds {
            getrandom::fill(seed)?;
        }
        let mut bbs_placement_seeds = Vec::new();
        for bbs_id in self.store.configured_bbs_ids()? {
            let mut seed = [0; 32];
            getrandom::fill(&mut seed)?;
            bbs_placement_seeds.push(BbsPlacementSeed { bbs_id, seed });
        }
        let mut initialization = self.store.initialize_universe(
            &command_id,
            universe_id,
            &system_seeds,
            &bbs_placement_seeds,
        )?;
        // Day-zero baselines are part of explicit universe provisioning, not
        // surprise catch-up work charged to the first player command.
        self.store.advance_simulation_to(0)?;
        initialization.committed_sequence = self.store.committed_sequence()?;
        Ok(initialization)
    }

    /// Internal simulation hook. It deliberately has no player/admin RPC.
    /// Ordinary loss or transfer must never call this method.
    pub fn record_unique_cargo_destruction(&self, object_id: u64) -> Result<bool, EngineError> {
        let recorded = self.store.record_unique_cargo_destruction(object_id)?;
        if recorded {
            self.reset_if_unique_cargo_destroyed()?;
        }
        Ok(recorded)
    }

    fn reset_if_unique_cargo_destroyed(&self) -> Result<bool, EngineError> {
        if !self.store.unique_cargo_reset_required()? {
            return Ok(false);
        }
        let mut command_id = [0; COMMAND_ID_BYTES];
        getrandom::fill(&mut command_id)?;
        self.initialize_universe(command_id)?;
        Ok(true)
    }

    pub fn bbs_configuration(&self, bbs_id: u32) -> Result<BbsConfiguration, EngineError> {
        Ok(self.store.bbs_configuration(bbs_id)?)
    }

    pub fn configure_bbs(
        &self,
        bbs_id: u32,
        command_id: [u8; COMMAND_ID_BYTES],
        expected_revision: u64,
        settings: &BbsSettings,
    ) -> Result<ConfigureBbsResult, EngineError> {
        let mut placement_seed = [0; 32];
        getrandom::fill(&mut placement_seed)?;
        Ok(self.store.configure_bbs(
            bbs_id,
            &command_id,
            expected_revision,
            settings,
            placement_seed,
        )?)
    }

    pub fn player_access(
        &self,
        identity: &PlayerIdentity,
    ) -> Result<PlayerAccessRecord, EngineError> {
        Ok(self.store.player_access(identity)?)
    }

    pub fn set_player_access(
        &self,
        bbs_id: u32,
        command_id: [u8; COMMAND_ID_BYTES],
        player_id: u32,
        expected_revision: u64,
        state: PlayerAccessState,
        reason: &str,
    ) -> Result<SetPlayerAccessResult, EngineError> {
        Ok(self.store.set_player_access(
            bbs_id,
            &command_id,
            player_id,
            expected_revision,
            state,
            reason,
        )?)
    }

    pub fn issue_sysop_directive(
        &self,
        bbs_id: u32,
        command_id: [u8; COMMAND_ID_BYTES],
        player_id: u32,
        kind: SysopDirectiveKind,
    ) -> Result<SysopDirectiveRecord, EngineError> {
        Ok(self
            .store
            .issue_sysop_directive(bbs_id, &command_id, player_id, kind)?)
    }

    pub fn committed_sequence(&self) -> Result<u64, EngineError> {
        Ok(self.store.committed_sequence()?)
    }

    pub fn operational_status(&self) -> Result<OperationalStatus, EngineError> {
        Ok(self.store.operational_status()?)
    }

    pub fn live_backup(
        &self,
        backup_root: &Path,
        label: &str,
        command_id: &[u8; COMMAND_ID_BYTES],
    ) -> Result<OperationalStatus, EngineError> {
        Ok(self.store.live_backup(backup_root, label, command_id)?)
    }

    pub fn advance_simulation_to(
        &self,
        target_second: u64,
    ) -> Result<SimulationAdvance, EngineError> {
        self.recover()?;
        Ok(self.store.advance_simulation_to(target_second)?)
    }

    pub fn advance_simulation_toward(
        &self,
        target_second: u64,
        maximum_events: u64,
    ) -> Result<SimulationAdvance, EngineError> {
        let recovered = self.recover()?;
        let mut advance = self
            .store
            .advance_simulation_toward(target_second, Some(maximum_events))?;
        if !recovered.player_transitions.is_empty() {
            let mut transitions = recovered.player_transitions;
            transitions.extend(advance.player_transitions);
            advance.player_transitions = transitions;
        }
        Ok(advance)
    }

    pub fn game_second(&self) -> Result<u64, EngineError> {
        Ok(self.store.game_second()?)
    }

    pub fn traffic_snapshot(
        &self,
        identity: &PlayerIdentity,
    ) -> Result<Option<crate::traffic::TrafficSnapshot>, EngineError> {
        Ok(self.store.traffic_snapshot(identity)?)
    }

    pub fn active_ship_id(&self, identity: &PlayerIdentity) -> Result<Option<u64>, EngineError> {
        Ok(self
            .store
            .player_record(identity)?
            .map(|player| player.ship_id))
    }

    pub fn traffic_movements(
        &self,
        system_id: u64,
        after_second: u64,
        through_second: u64,
    ) -> Result<Vec<crate::traffic::TrafficContact>, EngineError> {
        Ok(self
            .store
            .traffic_movements(system_id, after_second, through_second)?)
    }

    pub fn simulation_report(&self) -> Result<SimulationReport, EngineError> {
        Ok(self.store.simulation_report()?)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn unique_cargo_destruction_resets_game_state_and_preserves_bbs_control() {
        let dir = TempDir::new().unwrap();
        let engine = Engine::open_with_registries(
            dir.path(),
            BbsRegistry::default(),
            LeagueRegistry::default(),
        )
        .unwrap();
        let initial = engine.initialize_universe([1; COMMAND_ID_BYTES]).unwrap();
        let credential = engine.add_bbs([2; COMMAND_ID_BYTES], "Test BBS").unwrap();
        let before = engine.bbs_configuration(credential.bbs_id).unwrap();

        assert!(engine.record_unique_cargo_destruction(1).unwrap());
        assert!(!engine.store.unique_cargo_reset_required().unwrap());
        let after = engine.bbs_configuration(credential.bbs_id).unwrap();
        assert_eq!(after, before);
        assert_ne!(
            engine.store.committed_sequence().unwrap(),
            initial.committed_sequence
        );
        assert!(!engine.record_unique_cargo_destruction(999).unwrap());
    }

    #[test]
    fn league_member_disable_removes_and_enable_restores_live_bbs_authentication() {
        let dir = TempDir::new().unwrap();
        let bbs_registry = BbsRegistry::default();
        let engine = Engine::open_with_registries(
            dir.path(),
            bbs_registry.clone(),
            LeagueRegistry::default(),
        )
        .unwrap();
        engine.initialize_universe([3; COMMAND_ID_BYTES]).unwrap();
        let league = engine.add_league([4; COMMAND_ID_BYTES]).unwrap();
        let bbs = engine
            .add_league_bbs(league.league_id, [5; COMMAND_ID_BYTES], "League BBS")
            .unwrap();
        assert!(bbs_registry.contains(bbs.bbs_id));

        let disabled = engine
            .set_league_member_enabled(
                league.league_id,
                [6; COMMAND_ID_BYTES],
                bbs.bbs_id,
                0,
                false,
                "paused",
            )
            .unwrap();
        assert!(matches!(
            disabled,
            SetLeagueMemberAccessResult::Updated(ref member) if !member.enabled
        ));
        assert!(!bbs_registry.contains(bbs.bbs_id));
        assert!(matches!(
            engine.issue_session(&PlayerIdentity {
                bbs_id: bbs.bbs_id,
                player_id: 1,
            }),
            Err(EngineError::PlayerAccessDenied(_))
        ));

        engine
            .set_league_member_enabled(
                league.league_id,
                [7; COMMAND_ID_BYTES],
                bbs.bbs_id,
                1,
                true,
                "",
            )
            .unwrap();
        assert!(bbs_registry.contains(bbs.bbs_id));
    }
}
