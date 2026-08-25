//! Browser-alert enrollment, shared storage, and best-effort Web Push delivery.

use std::fs;
use std::io;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use sha2::{Digest, Sha256};
use url::Url;
use web_push_native::jwt_simple::algorithms::ES256KeyPair;
use web_push_native::p256::elliptic_curve::sec1::ToEncodedPoint;
use web_push_native::{Auth, WebPushBuilder, p256::PublicKey};

use crate::wire::{COMMAND_ID_BYTES, PlayerIdentity};

const SCHEMA_VERSION: i64 = 1;
const ENROLLMENT_LIFETIME: Duration = Duration::from_secs(10 * 60);
const WORKER_QUEUE_DEPTH: usize = 128;
// Bound each delivery sweep to one network request so enrollment and alert
// persistence cannot sit behind a many-device run of 15-second HTTP timeouts.
const DELIVERY_BATCH: i64 = 1;
const MAX_ATTEMPTS: i64 = 8;
const MAX_DEVICE_COUNT: u32 = 5;

#[derive(Clone, Debug)]
pub struct WebPushConfig {
    pub public_url: Url,
    pub database_path: PathBuf,
    pub vapid_private_key_path: PathBuf,
    pub vapid_subject: String,
}

impl WebPushConfig {
    pub fn new(
        public_url: &str,
        database_path: PathBuf,
        vapid_private_key_path: PathBuf,
        vapid_subject: String,
    ) -> Result<Self, WebPushError> {
        let public_url = Url::parse(public_url)
            .map_err(|error| WebPushError::Configuration(error.to_string()))?;
        if public_url.scheme() != "https"
            || public_url.host_str().is_none()
            || public_url.username() != ""
            || public_url.password().is_some()
            || public_url.query().is_some()
            || public_url.fragment().is_some()
            || !public_url.path().ends_with('/')
        {
            return Err(WebPushError::Configuration(
                "browser-alert URL must be a canonical HTTPS directory URL without user information, query, or fragment".into(),
            ));
        }
        let valid_subject = vapid_subject
            .strip_prefix("mailto:")
            .is_some_and(|address| {
                !address.is_empty() && !address.contains('\r') && !address.contains('\n')
            })
            || Url::parse(&vapid_subject).is_ok_and(|value| {
                value.scheme() == "https"
                    && value.host_str().is_some()
                    && value.username().is_empty()
                    && value.password().is_none()
                    && value.fragment().is_none()
            });
        if !valid_subject {
            return Err(WebPushError::Configuration(
                "VAPID subject must be a mailto: contact or plain HTTPS URL".into(),
            ));
        }
        Ok(Self {
            public_url,
            database_path,
            vapid_private_key_path,
            vapid_subject,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebPushError {
    #[error("browser alerts are not configured")]
    Disabled,
    #[error("browser-alert configuration: {0}")]
    Configuration(String),
    #[error("browser-alert storage: {0}")]
    Storage(#[from] rusqlite::Error),
    #[error("browser-alert I/O: {0}")]
    Io(#[from] io::Error),
    #[error("browser-alert worker stopped")]
    WorkerStopped,
    #[error("browser-alert worker: {0}")]
    Worker(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserAlertStatus {
    pub configured: bool,
    pub active_devices: u32,
    pub maximum_devices: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserAlertEnrollment {
    pub url: String,
    pub expires_unix_second: u64,
    pub active_devices: u32,
    pub maximum_devices: u32,
}

#[derive(Clone, Debug)]
pub struct PushAlert {
    pub identity: PlayerIdentity,
    pub source_key: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub detail_json: String,
    pub created_unix_second: u64,
    pub expires_unix_second: u64,
    /// For `attention-soon`, the real-world instant when attention becomes
    /// necessary. Each subscription's lead time is subtracted from this.
    pub attention_due_unix_second: u64,
}

enum WorkerMessage {
    CreateEnrollment {
        identity: PlayerIdentity,
        command_id: [u8; COMMAND_ID_BYTES],
        reply: mpsc::Sender<Result<BrowserAlertEnrollment, String>>,
    },
    Status {
        identity: PlayerIdentity,
        reply: mpsc::Sender<Result<BrowserAlertStatus, String>>,
    },
    RevokeAll {
        identity: PlayerIdentity,
        reply: mpsc::Sender<Result<BrowserAlertStatus, String>>,
    },
    Alert {
        alert: PushAlert,
        reply: mpsc::Sender<Result<(), String>>,
    },
    Shutdown,
}

#[derive(Clone)]
pub struct WebPushHandle {
    sender: Option<SyncSender<WorkerMessage>>,
}

pub struct WebPushWorker {
    sender: SyncSender<WorkerMessage>,
    join: Option<thread::JoinHandle<()>>,
}

impl WebPushHandle {
    pub fn disabled() -> Self {
        Self { sender: None }
    }

    pub fn configured(&self) -> bool {
        self.sender.is_some()
    }

    pub fn create_enrollment(
        &self,
        identity: PlayerIdentity,
        command_id: [u8; COMMAND_ID_BYTES],
    ) -> Result<BrowserAlertEnrollment, WebPushError> {
        let sender = self.sender.as_ref().ok_or(WebPushError::Disabled)?;
        let (reply, receiver) = mpsc::channel();
        sender
            .send(WorkerMessage::CreateEnrollment {
                identity,
                command_id,
                reply,
            })
            .map_err(|_| WebPushError::WorkerStopped)?;
        receiver
            .recv()
            .map_err(|_| WebPushError::WorkerStopped)?
            .map_err(WebPushError::Worker)
    }

    pub fn status(&self, identity: PlayerIdentity) -> Result<BrowserAlertStatus, WebPushError> {
        let Some(sender) = &self.sender else {
            return Ok(BrowserAlertStatus {
                configured: false,
                active_devices: 0,
                maximum_devices: MAX_DEVICE_COUNT,
            });
        };
        let (reply, receiver) = mpsc::channel();
        sender
            .send(WorkerMessage::Status { identity, reply })
            .map_err(|_| WebPushError::WorkerStopped)?;
        receiver
            .recv()
            .map_err(|_| WebPushError::WorkerStopped)?
            .map_err(WebPushError::Worker)
    }

    pub fn revoke_all(&self, identity: PlayerIdentity) -> Result<BrowserAlertStatus, WebPushError> {
        let sender = self.sender.as_ref().ok_or(WebPushError::Disabled)?;
        let (reply, receiver) = mpsc::channel();
        sender
            .send(WorkerMessage::RevokeAll { identity, reply })
            .map_err(|_| WebPushError::WorkerStopped)?;
        receiver
            .recv()
            .map_err(|_| WebPushError::WorkerStopped)?
            .map_err(WebPushError::Worker)
    }

    /// Hand an alert to the dedicated worker. Callers must invoke this away
    /// from the authoritative engine thread; once accepted, its SQLite alert
    /// and delivery rows provide retry durability.
    pub fn enqueue(&self, alert: PushAlert) -> Result<(), WebPushError> {
        let sender = self.sender.as_ref().ok_or(WebPushError::Disabled)?;
        let (reply, receiver) = mpsc::channel();
        sender
            .send(WorkerMessage::Alert { alert, reply })
            .map_err(|_| WebPushError::WorkerStopped)?;
        receiver
            .recv()
            .map_err(|_| WebPushError::WorkerStopped)?
            .map_err(WebPushError::Worker)
    }
}

impl WebPushWorker {
    pub fn spawn(config: WebPushConfig) -> Result<(WebPushHandle, Self), WebPushError> {
        let private_key = read_private_key(&config.vapid_private_key_path)?;
        let key_pair = ES256KeyPair::from_bytes(&private_key)
            .map_err(|error| WebPushError::Configuration(error.to_string()))?;
        let public_key = encode_vapid_public_key(&key_pair)?;
        let connection = open_database(&config.database_path, &public_key, &config.public_url)?;
        drop(connection);

        let (sender, receiver) = mpsc::sync_channel(WORKER_QUEUE_DEPTH);
        let thread_sender = sender.clone();
        let join = thread::Builder::new()
            .name("ct-web-push".into())
            .spawn(move || worker_main(config, private_key, receiver))?;
        Ok((
            WebPushHandle {
                sender: Some(thread_sender),
            },
            Self {
                sender,
                join: Some(join),
            },
        ))
    }
}

impl Drop for WebPushWorker {
    fn drop(&mut self) {
        let _ = self.sender.send(WorkerMessage::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

pub fn initialize_vapid_key(path: &Path) -> Result<String, WebPushError> {
    let key_pair = ES256KeyPair::generate();
    let private_key = key_pair.to_bytes();
    let public_key = encode_vapid_public_key(&key_pair)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write as _;
    let mut file = options.open(path)?;
    file.write_all(URL_SAFE_NO_PAD.encode(private_key).as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(public_key)
}

fn read_private_key(path: &Path) -> Result<Vec<u8>, WebPushError> {
    let encoded = fs::read_to_string(path)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded.trim())
        .map_err(|error| WebPushError::Configuration(format!("invalid VAPID key: {error}")))?;
    if bytes.len() != 32 {
        return Err(WebPushError::Configuration(
            "VAPID private key must contain 32 bytes".into(),
        ));
    }
    Ok(bytes)
}

/// Encode the VAPID application-server key in the uncompressed SEC1 form
/// required by the Web Push API. jwt-simple's `to_bytes()` deliberately uses
/// compressed SEC1, which browsers reject as an `applicationServerKey`.
fn encode_vapid_public_key(key_pair: &ES256KeyPair) -> Result<String, WebPushError> {
    let compressed = key_pair.public_key().to_bytes();
    let public_key = PublicKey::from_sec1_bytes(&compressed).map_err(|error| {
        WebPushError::Configuration(format!("invalid VAPID public key: {error}"))
    })?;
    Ok(URL_SAFE_NO_PAD.encode(public_key.to_encoded_point(false).as_bytes()))
}

fn open_database(
    path: &Path,
    public_key: &str,
    public_url: &Url,
) -> Result<Connection, WebPushError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let connection = Connection::open(path)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.execute_batch(
        "BEGIN IMMEDIATE;
         CREATE TABLE IF NOT EXISTS schema_meta(version INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS settings(
           name TEXT PRIMARY KEY,
           value TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS pairing_tokens(
           token_hash TEXT PRIMARY KEY,
           bbs_id INTEGER NOT NULL,
           player_id INTEGER NOT NULL,
           command_id BLOB NOT NULL UNIQUE,
           expires_unix INTEGER NOT NULL,
           consumed_unix INTEGER
         );
         CREATE TABLE IF NOT EXISTS browser_sessions(
           id INTEGER PRIMARY KEY,
           credential_hash TEXT NOT NULL UNIQUE,
           bbs_id INTEGER NOT NULL,
           player_id INTEGER NOT NULL,
           created_unix INTEGER NOT NULL,
           revoked_unix INTEGER
         );
         CREATE TABLE IF NOT EXISTS subscriptions(
           id INTEGER PRIMARY KEY,
           session_id INTEGER NOT NULL REFERENCES browser_sessions(id),
           bbs_id INTEGER NOT NULL,
           player_id INTEGER NOT NULL,
           endpoint TEXT NOT NULL UNIQUE,
           p256dh TEXT NOT NULL,
           auth TEXT NOT NULL,
           locale TEXT NOT NULL DEFAULT 'en-US',
           attention_soon INTEGER NOT NULL DEFAULT 1,
           attention_now INTEGER NOT NULL DEFAULT 1,
           automation_applied INTEGER NOT NULL DEFAULT 1,
           lead_minutes INTEGER NOT NULL DEFAULT 5 CHECK(lead_minutes BETWEEN 1 AND 1440),
           created_unix INTEGER NOT NULL,
           updated_unix INTEGER NOT NULL,
           revoked_unix INTEGER,
           failure_count INTEGER NOT NULL DEFAULT 0,
           last_success_unix INTEGER
         );
         CREATE TABLE IF NOT EXISTS alerts(
           id INTEGER PRIMARY KEY,
           notice_ref TEXT NOT NULL UNIQUE,
           bbs_id INTEGER NOT NULL,
           player_id INTEGER NOT NULL,
           source_key TEXT NOT NULL,
           kind TEXT NOT NULL,
           title TEXT NOT NULL,
           body TEXT NOT NULL,
           detail_json TEXT NOT NULL,
           created_unix INTEGER NOT NULL,
           expires_unix INTEGER NOT NULL,
           UNIQUE(bbs_id, player_id, source_key, kind)
         );
         CREATE TABLE IF NOT EXISTS deliveries(
           alert_id INTEGER NOT NULL REFERENCES alerts(id) ON DELETE CASCADE,
           subscription_id INTEGER NOT NULL REFERENCES subscriptions(id),
           state TEXT NOT NULL DEFAULT 'pending',
           attempt_count INTEGER NOT NULL DEFAULT 0,
           next_attempt_unix INTEGER NOT NULL,
           last_status INTEGER,
           last_error TEXT,
           PRIMARY KEY(alert_id, subscription_id)
         );
         CREATE INDEX IF NOT EXISTS delivery_due ON deliveries(state, next_attempt_unix);
         DELETE FROM schema_meta;
         COMMIT;",
    )?;
    connection.execute(
        "INSERT INTO schema_meta(version) VALUES(?)",
        [SCHEMA_VERSION],
    )?;
    connection.execute(
        "INSERT INTO settings(name,value) VALUES('vapid_public_key',?1)
         ON CONFLICT(name) DO UPDATE SET value=excluded.value",
        [public_key],
    )?;
    connection.execute(
        "INSERT INTO settings(name,value) VALUES('public_url',?1)
         ON CONFLICT(name) DO UPDATE SET value=excluded.value",
        [public_url.as_str()],
    )?;
    Ok(connection)
}

fn worker_main(config: WebPushConfig, private_key: Vec<u8>, receiver: Receiver<WorkerMessage>) {
    let key_pair = match ES256KeyPair::from_bytes(&private_key) {
        Ok(value) => value,
        Err(error) => {
            crate::server::log(format_args!("browser-alert worker key failure: {error}"));
            return;
        }
    };
    let public_key = match encode_vapid_public_key(&key_pair) {
        Ok(public_key) => public_key,
        Err(error) => {
            eprintln!("browser-alert worker failed: {error}");
            return;
        }
    };
    let mut connection = match open_database(&config.database_path, &public_key, &config.public_url)
    {
        Ok(value) => value,
        Err(error) => {
            crate::server::log(format_args!(
                "browser-alert worker storage failure: {error}"
            ));
            return;
        }
    };
    loop {
        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(WorkerMessage::CreateEnrollment {
                identity,
                command_id,
                reply,
            }) => {
                let result =
                    create_enrollment(&mut connection, &config, &private_key, identity, command_id)
                        .map_err(|error| error.to_string());
                let _ = reply.send(result);
            }
            Ok(WorkerMessage::Status { identity, reply }) => {
                let result = status(&connection, identity).map_err(|error| error.to_string());
                let _ = reply.send(result);
            }
            Ok(WorkerMessage::RevokeAll { identity, reply }) => {
                let result =
                    revoke_all(&mut connection, identity).map_err(|error| error.to_string());
                let _ = reply.send(result);
            }
            Ok(WorkerMessage::Alert { alert, reply }) => {
                let result =
                    store_alert(&mut connection, &alert).map_err(|error| error.to_string());
                if let Err(error) = &result {
                    crate::server::log(format_args!("browser-alert enqueue failure: {error}"));
                }
                let _ = reply.send(result);
            }
            Ok(WorkerMessage::Shutdown) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if let Err(error) = deliver_due(&mut connection, &key_pair, &config.vapid_subject) {
            crate::server::log(format_args!(
                "browser-alert delivery sweep failure: {error}"
            ));
        }
        let _ = connection.execute(
            "DELETE FROM pairing_tokens WHERE expires_unix < ?1 OR
               (consumed_unix IS NOT NULL AND consumed_unix < ?1)",
            [unix_now() as i64 - 3600],
        );
        let _ = connection.execute(
            "DELETE FROM alerts WHERE expires_unix < ?1",
            [unix_now() as i64],
        );
    }
}

fn create_enrollment(
    connection: &mut Connection,
    config: &WebPushConfig,
    private_key: &[u8],
    identity: PlayerIdentity,
    command_id: [u8; COMMAND_ID_BYTES],
) -> Result<BrowserAlertEnrollment, WebPushError> {
    let existing: Option<(Option<i64>, u32, u32)> = connection
        .query_row(
            "SELECT consumed_unix,bbs_id,player_id FROM pairing_tokens WHERE command_id=?1",
            [command_id.as_slice()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    if let Some((consumed, bbs_id, player_id)) = existing {
        if bbs_id != identity.bbs_id || player_id != identity.player_id {
            return Err(WebPushError::Worker(
                "browser-alert enrollment command ID belongs to another captain".into(),
            ));
        }
        if consumed.is_some() {
            return Err(WebPushError::Worker(
                "this browser-alert enrollment command was already consumed".into(),
            ));
        }
    }
    let active = active_devices(connection, &identity)?;
    if active >= MAX_DEVICE_COUNT {
        return Err(WebPushError::Worker(
            "five browsers are already linked; revoke one before linking another".into(),
        ));
    }
    let mut digest = Sha256::new();
    digest.update(b"cepheus-trader/browser-alert-enrollment/v1\0");
    digest.update(private_key);
    digest.update(identity.bbs_id.to_be_bytes());
    digest.update(identity.player_id.to_be_bytes());
    digest.update(command_id);
    let token = URL_SAFE_NO_PAD.encode(&digest.finalize()[..16]);
    let token_hash = hex_sha256(&token);
    let expires = unix_now() + ENROLLMENT_LIFETIME.as_secs();
    connection.execute(
        "INSERT INTO pairing_tokens(token_hash,bbs_id,player_id,command_id,expires_unix,consumed_unix)
         VALUES(?1,?2,?3,?4,?5,NULL)
         ON CONFLICT(command_id) DO UPDATE SET token_hash=excluded.token_hash,
           expires_unix=excluded.expires_unix
         WHERE pairing_tokens.consumed_unix IS NULL",
        params![token_hash, identity.bbs_id, identity.player_id, command_id.as_slice(), expires],
    )?;
    let mut url = config.public_url.clone();
    url.set_fragment(Some(&token));
    Ok(BrowserAlertEnrollment {
        url: url.into(),
        expires_unix_second: expires,
        active_devices: active,
        maximum_devices: MAX_DEVICE_COUNT,
    })
}

fn status(
    connection: &Connection,
    identity: PlayerIdentity,
) -> Result<BrowserAlertStatus, WebPushError> {
    Ok(BrowserAlertStatus {
        configured: true,
        active_devices: active_devices(connection, &identity)?,
        maximum_devices: MAX_DEVICE_COUNT,
    })
}

fn active_devices(
    connection: &Connection,
    identity: &PlayerIdentity,
) -> Result<u32, rusqlite::Error> {
    connection.query_row(
        "SELECT COUNT(DISTINCT session_id) FROM subscriptions
         WHERE bbs_id=?1 AND player_id=?2 AND revoked_unix IS NULL",
        params![identity.bbs_id, identity.player_id],
        |row| row.get(0),
    )
}

fn revoke_all(
    connection: &mut Connection,
    identity: PlayerIdentity,
) -> Result<BrowserAlertStatus, WebPushError> {
    let now = unix_now();
    let transaction = connection.transaction()?;
    transaction.execute(
        "UPDATE subscriptions SET revoked_unix=?1 WHERE bbs_id=?2 AND player_id=?3 AND revoked_unix IS NULL",
        params![now, identity.bbs_id, identity.player_id],
    )?;
    transaction.execute(
        "UPDATE browser_sessions SET revoked_unix=?1 WHERE bbs_id=?2 AND player_id=?3 AND revoked_unix IS NULL",
        params![now, identity.bbs_id, identity.player_id],
    )?;
    transaction.commit()?;
    status(connection, identity)
}

fn store_alert(connection: &mut Connection, alert: &PushAlert) -> Result<(), WebPushError> {
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|error| WebPushError::Worker(error.to_string()))?;
    let notice_ref = URL_SAFE_NO_PAD.encode(random);
    let transaction = connection.transaction()?;
    transaction.execute(
        "INSERT INTO alerts(notice_ref,bbs_id,player_id,source_key,kind,title,body,detail_json,created_unix,expires_unix)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
         ON CONFLICT(bbs_id,player_id,source_key,kind) DO NOTHING",
        params![notice_ref, alert.identity.bbs_id, alert.identity.player_id, alert.source_key,
            alert.kind, alert.title, alert.body, alert.detail_json,
            alert.created_unix_second, alert.expires_unix_second],
    )?;
    let alert_id: Option<i64> = transaction
        .query_row(
            "SELECT id FROM alerts WHERE bbs_id=?1 AND player_id=?2 AND source_key=?3 AND kind=?4",
            params![
                alert.identity.bbs_id,
                alert.identity.player_id,
                alert.source_key,
                alert.kind
            ],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(alert_id) = alert_id {
        transaction.execute(
            "INSERT INTO deliveries(alert_id,subscription_id,next_attempt_unix)
             SELECT ?1,id,
               CASE WHEN ?5='attention-soon'
                 THEN MAX(?2, ?6 - (lead_minutes * 60)) ELSE ?2 END
             FROM subscriptions
             WHERE bbs_id=?3 AND player_id=?4 AND revoked_unix IS NULL AND
               CASE ?5 WHEN 'attention-soon' THEN attention_soon
                       WHEN 'attention-now' THEN attention_now
                       WHEN 'automation-applied' THEN automation_applied ELSE 0 END = 1
             ON CONFLICT(alert_id,subscription_id) DO NOTHING",
            params![
                alert_id,
                unix_now(),
                alert.identity.bbs_id,
                alert.identity.player_id,
                alert.kind,
                alert.attention_due_unix_second
            ],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

#[derive(Serialize)]
struct NotificationPayload<'a> {
    title: &'a str,
    body: &'a str,
    tag: String,
    notice: &'a str,
}

struct DueDelivery {
    alert_id: i64,
    subscription_id: i64,
    endpoint: String,
    p256dh: String,
    auth: String,
    notice_ref: String,
    title: String,
    body: String,
    expires: u64,
    attempts: i64,
}

fn deliver_due(
    connection: &mut Connection,
    key_pair: &ES256KeyPair,
    subject: &str,
) -> Result<(), WebPushError> {
    let now = unix_now();
    let deliveries = {
        let mut query = connection.prepare(
            "SELECT d.alert_id,d.subscription_id,s.endpoint,s.p256dh,s.auth,
                    a.notice_ref,a.title,a.body,a.expires_unix,d.attempt_count
             FROM deliveries d JOIN alerts a ON a.id=d.alert_id
             JOIN subscriptions s ON s.id=d.subscription_id
             WHERE d.state='pending' AND d.next_attempt_unix<=?1
               AND a.expires_unix>?1 AND s.revoked_unix IS NULL
               AND CASE a.kind WHEN 'attention-soon' THEN s.attention_soon
                       WHEN 'attention-now' THEN s.attention_now
                       WHEN 'automation-applied' THEN s.automation_applied ELSE 0 END = 1
             ORDER BY d.next_attempt_unix LIMIT ?2",
        )?;
        query
            .query_map(params![now, DELIVERY_BATCH], |row| {
                Ok(DueDelivery {
                    alert_id: row.get(0)?,
                    subscription_id: row.get(1)?,
                    endpoint: row.get(2)?,
                    p256dh: row.get(3)?,
                    auth: row.get(4)?,
                    notice_ref: row.get(5)?,
                    title: row.get(6)?,
                    body: row.get(7)?,
                    expires: row.get(8)?,
                    attempts: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
    };
    for delivery in deliveries {
        let payload = serde_json::to_vec(&NotificationPayload {
            title: &delivery.title,
            body: &delivery.body,
            tag: format!("ct-{}", delivery.alert_id),
            notice: &delivery.notice_ref,
        })
        .map_err(|error| WebPushError::Worker(error.to_string()))?;
        let ttl = delivery.expires.saturating_sub(now).max(1);
        match send_push(
            key_pair,
            subject,
            &delivery.endpoint,
            &delivery.p256dh,
            &delivery.auth,
            ttl,
            payload,
        ) {
            Ok(status) if status.is_success() => {
                connection.execute(
                    "UPDATE deliveries SET state='delivered',attempt_count=attempt_count+1,last_status=?1,last_error=NULL
                     WHERE alert_id=?2 AND subscription_id=?3",
                    params![status.as_u16(), delivery.alert_id, delivery.subscription_id],
                )?;
                connection.execute(
                    "UPDATE subscriptions SET failure_count=0,last_success_unix=?1 WHERE id=?2",
                    params![now, delivery.subscription_id],
                )?;
            }
            Ok(status) if status == StatusCode::NOT_FOUND || status == StatusCode::GONE => {
                connection.execute(
                    "UPDATE subscriptions SET revoked_unix=?1 WHERE id=?2",
                    params![now, delivery.subscription_id],
                )?;
                connection.execute(
                    "UPDATE deliveries SET state='expired',attempt_count=attempt_count+1,last_status=?1 WHERE alert_id=?2 AND subscription_id=?3",
                    params![status.as_u16(), delivery.alert_id, delivery.subscription_id],
                )?;
            }
            Ok(status) if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() => {
                retry_delivery(
                    connection,
                    &delivery,
                    now,
                    Some(status.as_u16()),
                    "push service temporarily unavailable",
                )?;
            }
            Ok(status) => {
                connection.execute(
                    "UPDATE deliveries SET state='failed',attempt_count=attempt_count+1,last_status=?1,last_error='permanent push response'
                     WHERE alert_id=?2 AND subscription_id=?3",
                    params![status.as_u16(), delivery.alert_id, delivery.subscription_id],
                )?;
            }
            Err(error) => retry_delivery(connection, &delivery, now, None, &error)?,
        }
    }
    connection.execute(
        "UPDATE deliveries SET state='expired' WHERE state='pending' AND alert_id IN
         (SELECT id FROM alerts WHERE expires_unix<=?1)",
        [now],
    )?;
    Ok(())
}

fn retry_delivery(
    connection: &Connection,
    delivery: &DueDelivery,
    now: u64,
    status: Option<u16>,
    error: &str,
) -> Result<(), rusqlite::Error> {
    let attempts = delivery.attempts + 1;
    let state = if attempts >= MAX_ATTEMPTS {
        "failed"
    } else {
        "pending"
    };
    let delay = 5u64.saturating_mul(1u64 << attempts.min(10));
    connection.execute(
        "UPDATE deliveries SET state=?1,attempt_count=?2,next_attempt_unix=?3,last_status=?4,last_error=?5
         WHERE alert_id=?6 AND subscription_id=?7",
        params![state, attempts, now.saturating_add(delay), status, truncate_error(error),
            delivery.alert_id, delivery.subscription_id],
    )?;
    connection.execute(
        "UPDATE subscriptions SET failure_count=failure_count+1 WHERE id=?1",
        [delivery.subscription_id],
    )?;
    Ok(())
}

fn send_push(
    key_pair: &ES256KeyPair,
    subject: &str,
    endpoint: &str,
    p256dh: &str,
    auth: &str,
    ttl: u64,
    payload: Vec<u8>,
) -> Result<StatusCode, String> {
    let url = Url::parse(endpoint).map_err(|error| error.to_string())?;
    if url.scheme() != "https" || url.username() != "" || url.password().is_some() {
        return Err("subscription endpoint is not a plain HTTPS URL".into());
    }
    let host = url.host_str().ok_or("subscription endpoint has no host")?;
    let port = url
        .port_or_known_default()
        .ok_or("subscription endpoint has no port")?;
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("cannot resolve push endpoint: {error}"))?
        .filter(|address| is_public_ip(address.ip()))
        .collect::<Vec<_>>();
    let address = addresses
        .first()
        .copied()
        .ok_or("push endpoint resolves only to non-public addresses")?;
    let client = Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(15))
        .resolve(host, address)
        .build()
        .map_err(|error| error.to_string())?;
    let public_bytes = URL_SAFE_NO_PAD
        .decode(p256dh)
        .map_err(|error| error.to_string())?;
    let auth_bytes = URL_SAFE_NO_PAD
        .decode(auth)
        .map_err(|error| error.to_string())?;
    if auth_bytes.len() != 16 {
        return Err("subscription auth secret is not 16 bytes".into());
    }
    let public =
        PublicKey::from_sec1_bytes(&public_bytes).map_err(|_| "invalid subscription P-256 key")?;
    let auth = Auth::clone_from_slice(&auth_bytes);
    let request = WebPushBuilder::new(
        endpoint
            .parse()
            .map_err(|error| format!("invalid endpoint: {error}"))?,
        public,
        auth,
    )
    .with_valid_duration(Duration::from_secs(ttl))
    .with_vapid(key_pair, subject)
    .build(payload)
    .map_err(|error| error.to_string())?;
    let mut builder = client.post(endpoint);
    for (name, value) in request.headers() {
        builder = builder.header(name, value);
    }
    builder
        .body(request.into_body())
        .send()
        .map(|response| response.status())
        .map_err(|error| error.to_string())
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_broadcast()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && octets[0] != 0
                && !(octets[0] == 100 && (64..=127).contains(&octets[1]))
                && !(octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
                && !(octets[0] >= 224)
        }
        IpAddr::V6(ip) => {
            !ip.is_loopback()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !(ip.segments()[0] & 0xfe00 == 0xfc00)
                && !(ip.segments()[0] & 0xffc0 == 0xfe80)
        }
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn hex_sha256(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn truncate_error(error: &str) -> String {
    error.chars().take(240).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_address_filter_rejects_local_networks() {
        assert!(!is_public_ip("127.0.0.1".parse().unwrap()));
        assert!(!is_public_ip("10.2.3.4".parse().unwrap()));
        assert!(!is_public_ip("::1".parse().unwrap()));
        assert!(is_public_ip("8.8.8.8".parse().unwrap()));
        assert!(is_public_ip("2606:4700:4700::1111".parse().unwrap()));
    }

    #[test]
    fn vapid_public_key_uses_browser_sec1_encoding() {
        let key_pair = ES256KeyPair::generate();
        let encoded = encode_vapid_public_key(&key_pair).unwrap();
        let decoded = URL_SAFE_NO_PAD.decode(encoded).unwrap();
        assert_eq!(decoded.len(), 65);
        assert_eq!(decoded[0], 0x04);
    }

    #[test]
    fn schema_and_enrollment_are_idempotent() {
        let directory = tempfile::tempdir().unwrap();
        let key_path = directory.path().join("vapid.key");
        initialize_vapid_key(&key_path).unwrap();
        let config = WebPushConfig::new(
            "https://example.test/ct-alerts/",
            directory.path().join("push.sqlite3"),
            key_path.clone(),
            "mailto:sysop@example.test".into(),
        )
        .unwrap();
        let private = read_private_key(&key_path).unwrap();
        let key_pair = ES256KeyPair::from_bytes(&private).unwrap();
        let mut database = open_database(
            &config.database_path,
            &encode_vapid_public_key(&key_pair).unwrap(),
            &config.public_url,
        )
        .unwrap();
        let identity = PlayerIdentity {
            bbs_id: 2,
            player_id: 7,
        };
        let first =
            create_enrollment(&mut database, &config, &private, identity.clone(), [9; 16]).unwrap();
        let second =
            create_enrollment(&mut database, &config, &private, identity, [9; 16]).unwrap();
        assert_eq!(first.url, second.url);
        assert_eq!(first.maximum_devices, 5);
        database
            .execute(
                "UPDATE pairing_tokens SET consumed_unix=?1 WHERE command_id=?2",
                params![unix_now(), [9_u8; 16].as_slice()],
            )
            .unwrap();
        let replay = create_enrollment(
            &mut database,
            &config,
            &private,
            PlayerIdentity {
                bbs_id: 2,
                player_id: 7,
            },
            [9; 16],
        )
        .unwrap_err();
        assert!(replay.to_string().contains("already consumed"));
    }

    #[test]
    fn attention_warning_uses_each_subscription_lead_time() {
        let directory = tempfile::tempdir().unwrap();
        let key_path = directory.path().join("vapid.key");
        initialize_vapid_key(&key_path).unwrap();
        let config = WebPushConfig::new(
            "https://example.test/ct-alerts/",
            directory.path().join("push.sqlite3"),
            key_path.clone(),
            "mailto:sysop@example.test".into(),
        )
        .unwrap();
        let private = read_private_key(&key_path).unwrap();
        let key_pair = ES256KeyPair::from_bytes(&private).unwrap();
        let mut database = open_database(
            &config.database_path,
            &encode_vapid_public_key(&key_pair).unwrap(),
            &config.public_url,
        )
        .unwrap();
        database
            .execute(
                "INSERT INTO browser_sessions(id,credential_hash,bbs_id,player_id,created_unix) \
                 VALUES(1,'credential',2,7,1)",
                [],
            )
            .unwrap();
        database
            .execute(
                "INSERT INTO subscriptions(session_id,bbs_id,player_id,endpoint,p256dh,auth,\
                 lead_minutes,created_unix,updated_unix) \
                 VALUES(1,2,7,'https://push.example.test/one','key','auth',12,1,1)",
                [],
            )
            .unwrap();
        let now = unix_now();
        let due = now + 3_600;
        store_alert(
            &mut database,
            &PushAlert {
                identity: PlayerIdentity {
                    bbs_id: 2,
                    player_id: 7,
                },
                source_key: "leg:1".into(),
                kind: "attention-soon".into(),
                title: "Bridge watch reminder".into(),
                body: "The ship will wait for orders.".into(),
                detail_json: "{}".into(),
                created_unix_second: now,
                expires_unix_second: due + 3_600,
                attention_due_unix_second: due,
            },
        )
        .unwrap();
        let scheduled: u64 = database
            .query_row("SELECT next_attempt_unix FROM deliveries", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(scheduled, due - 12 * 60);
    }
}
