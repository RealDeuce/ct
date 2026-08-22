//! Read-only universe snapshots for the operator-hosted browser atlas.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use heed::byteorder::BE;
use heed::types::{Bytes, Str, U64};
use heed::{Database, EnvFlags, EnvOpenOptions};
use thiserror::Error;

use crate::celestial::derive_primary_world;
use crate::store::{
    STORAGE_FORMAT_VERSION, StoreError, SystemPublicationState, decode_stellar_system,
    decode_system_publication,
};
use crate::universe::INITIAL_SYSTEMS;

const INDEX_HTML: &str = include_str!("atlas_assets/index.html");
const ATLAS_CSS: &str = include_str!("atlas_assets/atlas.css");
const ATLAS_JS: &str = include_str!("atlas_assets/atlas.js");
const ATLAS_ROUTES_JS: &str = include_str!("atlas_assets/atlas-routes.js");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtlasVisibility {
    UniversallyKnown,
    Omniscient,
}

impl AtlasVisibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UniversallyKnown => "universally-known",
            Self::Omniscient => "omniscient",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtlasSystem {
    pub id: u64,
    pub name: String,
    pub world_name: Option<String>,
    pub position_parsecs: [f64; 3],
    pub polity_id: u64,
    pub starport: Option<char>,
    pub population: Option<u8>,
    pub tech_level: Option<u8>,
    pub universally_known_second: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtlasSnapshot {
    pub visibility: AtlasVisibility,
    pub game_second: u64,
    pub generated_unix_second: u64,
    pub systems: Vec<AtlasSystem>,
}

#[derive(Debug, Error)]
pub enum AtlasError {
    #[error("atlas database path does not exist: {0}")]
    MissingDatabase(PathBuf),
    #[error("atlas output already exists; choose a new directory: {0}")]
    OutputExists(PathBuf),
    #[error("atlas database is missing {0}")]
    MissingDatabaseTable(&'static str),
    #[error("atlas database metadata {0} is corrupt")]
    CorruptMetadata(&'static str),
    #[error(
        "atlas database storage format {actual} is incompatible with required format {required}"
    )]
    IncompatibleStorageFormat { actual: u64, required: u64 },
    #[error(transparent)]
    Heed(#[from] heed::Error),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Crypto(#[from] crate::crypto::CryptoError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("system count cannot be represented by the atlas format")]
    SystemCountOverflow,
    #[error("system {0} has a non-finite coordinate")]
    NonFiniteCoordinate(u64),
}

pub fn read_snapshot(
    data: impl AsRef<Path>,
    visibility: AtlasVisibility,
) -> Result<AtlasSnapshot, AtlasError> {
    let data = data.as_ref();
    if !data.is_dir() {
        return Err(AtlasError::MissingDatabase(data.to_owned()));
    }
    let mut options = EnvOpenOptions::new();
    options.max_dbs(80);
    // SAFETY: READ_ONLY is the only non-default LMDB flag. The atlas never
    // obtains a write transaction and uses the ordinary LMDB lock file while
    // the authoritative server may be running.
    let env = unsafe {
        options.flags(EnvFlags::READ_ONLY);
        options.open(data)?
    };
    let txn = env.read_txn()?;
    let meta: Database<Str, Bytes> = env
        .open_database(&txn, Some("metadata"))?
        .ok_or(AtlasError::MissingDatabaseTable("metadata"))?;
    let systems: Database<U64<BE>, Bytes> = env
        .open_database(&txn, Some("stellar-systems"))?
        .ok_or(AtlasError::MissingDatabaseTable("stellar-systems"))?;
    let publications: Option<Database<U64<BE>, Bytes>> =
        env.open_database(&txn, Some("system-publications"))?;
    let storage_format = metadata_u64(meta, &txn, "storage-format-version")?
        .ok_or(AtlasError::CorruptMetadata("storage-format-version"))?;
    if storage_format != STORAGE_FORMAT_VERSION {
        return Err(AtlasError::IncompatibleStorageFormat {
            actual: storage_format,
            required: STORAGE_FORMAT_VERSION,
        });
    }
    let game_second = metadata_u64(meta, &txn, "game-second")?.unwrap_or(0);
    // Storage-format-1 universes may predate the publication database. The
    // fixed Federation baseline was public from game second zero, so it is a
    // safe read-only fallback until the current server performs its backfill.
    let mut universally_known = INITIAL_SYSTEMS
        .iter()
        .map(|system| (system.id, 0))
        .collect::<std::collections::HashMap<_, _>>();
    if let Some(publications) = publications {
        for entry in publications.iter(&txn)? {
            let (system_id, bytes) = entry?;
            let record = decode_system_publication(bytes)?;
            if record.state == SystemPublicationState::UniversallyKnown {
                universally_known.insert(system_id, record.completed_second);
            }
        }
    }
    let included = universally_known.keys().copied().collect::<HashSet<_>>();
    let mut atlas_systems = Vec::new();
    for entry in systems.iter(&txn)? {
        let (system_id, encoded) = entry?;
        if visibility == AtlasVisibility::UniversallyKnown && !included.contains(&system_id) {
            continue;
        }
        let system = decode_stellar_system(encoded)?;
        if !system
            .position_parsecs
            .iter()
            .all(|value| value.is_finite())
        {
            return Err(AtlasError::NonFiniteCoordinate(system.id));
        }
        let world = derive_primary_world(&system)?;
        atlas_systems.push(AtlasSystem {
            id: system.id,
            name: system.name,
            world_name: Some(world.name),
            position_parsecs: system.position_parsecs,
            polity_id: system.polity_id,
            starport: Some(world.starport.code()),
            population: Some(world.population),
            tech_level: Some(world.tech_level),
            universally_known_second: universally_known.get(&system_id).copied(),
        });
    }
    atlas_systems.sort_by_key(|system| system.id);
    drop(txn);
    env.prepare_for_closing().wait();
    Ok(AtlasSnapshot {
        visibility,
        game_second,
        generated_unix_second: generated_unix_second(),
        systems: atlas_systems,
    })
}

pub fn initial_snapshot() -> AtlasSnapshot {
    AtlasSnapshot {
        visibility: AtlasVisibility::UniversallyKnown,
        game_second: 0,
        generated_unix_second: generated_unix_second(),
        systems: INITIAL_SYSTEMS
            .iter()
            .map(|system| AtlasSystem {
                id: system.id,
                name: system.name.into(),
                world_name: (system.id == 1).then(|| "Earth".into()),
                position_parsecs: system.position_parsecs,
                polity_id: 1,
                starport: (system.id == 1).then_some('A'),
                population: (system.id == 1).then_some(9),
                tech_level: (system.id == 1).then_some(13),
                universally_known_second: Some(0),
            })
            .collect(),
    }
}

pub fn write_site(snapshot: &AtlasSnapshot, output: impl AsRef<Path>) -> Result<(), AtlasError> {
    let output = output.as_ref();
    if output.exists() {
        return Err(AtlasError::OutputExists(output.to_owned()));
    }
    fs::create_dir_all(output)?;
    fs::write(output.join("index.html"), INDEX_HTML)?;
    fs::write(output.join("atlas.css"), ATLAS_CSS)?;
    fs::write(output.join("atlas-routes.js"), ATLAS_ROUTES_JS)?;
    fs::write(output.join("atlas.js"), ATLAS_JS)?;
    let file = fs::File::create(output.join("universe.json"))?;
    let mut writer = BufWriter::new(file);
    write_snapshot_json(snapshot, &mut writer)?;
    writer.flush()?;
    Ok(())
}

fn metadata_u64(
    database: Database<Str, Bytes>,
    txn: &heed::RoTxn<'_>,
    key: &'static str,
) -> Result<Option<u64>, AtlasError> {
    database
        .get(txn, key)?
        .map(|bytes| {
            let encoded: [u8; 8] = bytes
                .try_into()
                .map_err(|_| AtlasError::CorruptMetadata(key))?;
            Ok(u64::from_be_bytes(encoded))
        })
        .transpose()
}

fn generated_unix_second() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn write_snapshot_json(snapshot: &AtlasSnapshot, writer: &mut impl Write) -> io::Result<()> {
    let count = u64::try_from(snapshot.systems.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, AtlasError::SystemCountOverflow))?;
    write!(
        writer,
        "{{\"schemaVersion\":1,\"visibility\":\"{}\",\"gameSecond\":{},\"generatedUnixSecond\":{},\"systemCount\":{},\"axes\":{{\"x\":\"coreward\",\"y\":\"spinward\",\"z\":\"galactic north\",\"unit\":\"parsec\"}},\"systems\":[",
        snapshot.visibility.as_str(),
        snapshot.game_second,
        snapshot.generated_unix_second,
        count,
    )?;
    for (index, system) in snapshot.systems.iter().enumerate() {
        if index != 0 {
            writer.write_all(b",")?;
        }
        writer.write_all(b"{\"id\":")?;
        write!(writer, "{}", system.id)?;
        writer.write_all(b",\"name\":")?;
        write_json_string(writer, &system.name)?;
        writer.write_all(b",\"world\":")?;
        write_json_option_string(writer, system.world_name.as_deref())?;
        write!(
            writer,
            ",\"position\":[{:.9},{:.9},{:.9}],\"polityId\":{},\"starport\":",
            system.position_parsecs[0],
            system.position_parsecs[1],
            system.position_parsecs[2],
            system.polity_id,
        )?;
        let starport = system.starport.map(|value| value.to_string());
        write_json_option_string(writer, starport.as_deref())?;
        writer.write_all(b",\"population\":")?;
        write_json_option_number(writer, system.population)?;
        writer.write_all(b",\"techLevel\":")?;
        write_json_option_number(writer, system.tech_level)?;
        writer.write_all(b",\"universallyKnownSecond\":")?;
        match system.universally_known_second {
            Some(second) => write!(writer, "{second}")?,
            None => writer.write_all(b"null")?,
        }
        writer.write_all(b"}")?;
    }
    writer.write_all(b"]}\n")
}

fn write_json_option_number(writer: &mut impl Write, value: Option<u8>) -> io::Result<()> {
    match value {
        Some(value) => write!(writer, "{value}"),
        None => writer.write_all(b"null"),
    }
}

fn write_json_option_string(writer: &mut impl Write, value: Option<&str>) -> io::Result<()> {
    match value {
        Some(value) => write_json_string(writer, value),
        None => writer.write_all(b"null"),
    }
}

fn write_json_string(writer: &mut impl Write, value: &str) -> io::Result<()> {
    writer.write_all(b"\"")?;
    let mut escaped = String::new();
    for character in value.chars() {
        escaped.clear();
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value if value < '\u{20}' => {
                write!(&mut escaped, "\\u{:04x}", value as u32).expect("write string");
            }
            value => escaped.push(value),
        }
        writer.write_all(escaped.as_bytes())?;
    }
    writer.write_all(b"\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{BbsSettings, ConfigureBbsResult, Store};
    use crate::wire::COMMAND_ID_BYTES;

    #[test]
    fn initial_snapshot_json_has_axes_and_no_generation_seeds() {
        let snapshot = initial_snapshot();
        let mut json = Vec::new();
        write_snapshot_json(&snapshot, &mut json).unwrap();
        let json = String::from_utf8(json).unwrap();
        assert!(json.contains("\"visibility\":\"universally-known\""));
        assert!(json.contains("\"x\":\"coreward\""));
        assert!(json.contains("Alpha Centauri A"));
        assert!(!json.contains("seed"));
        assert_eq!(snapshot.systems.len(), INITIAL_SYSTEMS.len());
    }

    #[test]
    fn site_writer_refuses_to_replace_an_existing_directory() {
        let parent = tempfile::tempdir().unwrap();
        let output = parent.path().join("atlas");
        fs::create_dir(&output).unwrap();
        assert!(matches!(
            write_site(&initial_snapshot(), &output),
            Err(AtlasError::OutputExists(path)) if path == output
        ));
    }

    #[test]
    fn generated_site_contains_the_route_computer() {
        let parent = tempfile::tempdir().unwrap();
        let output = parent.path().join("atlas");
        write_site(&initial_snapshot(), &output).unwrap();
        assert!(output.join("atlas-routes.js").is_file());
        let html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(html.contains("id=\"route-origin\""));
        assert!(html.contains("id=\"route-destination\""));
        assert!(html.contains("src=\"atlas-routes.js\""));
    }

    #[test]
    fn read_only_snapshot_filters_systems_that_are_not_universally_known() {
        let directory = tempfile::tempdir().unwrap();
        {
            let store = Store::open(directory.path()).unwrap();
            let seeds = (0..INITIAL_SYSTEMS.len())
                .map(|index| [0x20_u8.wrapping_add(index as u8); 32])
                .collect::<Vec<_>>();
            store
                .initialize_universe(&[0x30; COMMAND_ID_BYTES], *b"CT-ATLAS-TEST-v1", &seeds, &[])
                .unwrap();
            let credential = store
                .add_bbs(&[0x31; COMMAND_ID_BYTES], "Atlas Test", [0x32; 32])
                .unwrap();
            assert!(matches!(
                store
                    .configure_bbs(
                        credential.bbs_id,
                        &[0x33; COMMAND_ID_BYTES],
                        0,
                        &BbsSettings {
                            bbs_name: "Atlas Test".into(),
                            polity_name: "Cartographers".into(),
                            trade_combat: 50,
                            chaos_order: 50,
                        },
                        [0x34; 32],
                    )
                    .unwrap(),
                ConfigureBbsResult::Updated(_)
            ));
        }

        let known = read_snapshot(directory.path(), AtlasVisibility::UniversallyKnown).unwrap();
        let omniscient = read_snapshot(directory.path(), AtlasVisibility::Omniscient).unwrap();
        assert_eq!(known.systems.len(), INITIAL_SYSTEMS.len());
        assert!(omniscient.systems.len() > known.systems.len());
        assert!(
            known
                .systems
                .iter()
                .all(|system| system.universally_known_second == Some(0))
        );
        assert!(
            omniscient
                .systems
                .iter()
                .any(|system| system.universally_known_second.is_none())
        );
    }
}
