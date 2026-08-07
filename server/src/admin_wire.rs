//! CT administrator protocol encoding.

use std::io::Cursor;

use capnp::message::{Builder, ReaderOptions};
use capnp::serialize;
use thiserror::Error;

use crate::ct_admin_capnp::{ErrorCode, envelope, request};
use crate::store::{BbsCredential, OperationalStatus};
use crate::universe::UniverseInitialization;
use crate::wire::{COMMAND_ID_BYTES, MAX_FRAME_BYTES};

const PROTOCOL_VERSION: u16 = 1;

pub const MAX_BBS_NAME_BYTES: usize = 128;

#[derive(Debug, Error)]
pub enum AdminWireError {
    #[error("Cap'n Proto message error: {0}")]
    Capnp(#[from] capnp::Error),
    #[error("unknown schema discriminant {0}")]
    NotInSchema(#[from] capnp::NotInSchema),
    #[error("unsupported protocol version {0}")]
    UnsupportedVersion(u16),
    #[error("expected {0}")]
    Expected(&'static str),
    #[error("invalid UTF-8 text")]
    InvalidText,
    #[error("command ID must contain exactly {COMMAND_ID_BYTES} bytes")]
    InvalidCommandId,
    #[error("BBS name must contain 1..={MAX_BBS_NAME_BYTES} bytes")]
    InvalidBbsName,
    #[error("backup label must contain 1..64 ASCII letters, digits, '-' or '_'")]
    InvalidBackupLabel,
    #[error("frame exceeds {MAX_FRAME_BYTES} bytes")]
    FrameTooLarge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminCommand {
    AddBbs { name: String },
    InitializeUniverse,
    Status,
    LiveBackup { label: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminRequest {
    pub request_id: u64,
    pub command_id: [u8; COMMAND_ID_BYTES],
    pub command: AdminCommand,
}

pub fn decode_request(bytes: &[u8]) -> Result<AdminRequest, AdminWireError> {
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(AdminWireError::FrameTooLarge);
    }
    let mut options = ReaderOptions::new();
    options
        .traversal_limit_in_words(Some(MAX_FRAME_BYTES / 8))
        .nesting_limit(16);
    let message = serialize::read_message(&mut Cursor::new(bytes), options)?;
    let envelope = message.get_root::<envelope::Reader>()?;
    if envelope.get_protocol_version() != PROTOCOL_VERSION {
        return Err(AdminWireError::UnsupportedVersion(
            envelope.get_protocol_version(),
        ));
    }
    let request_id = envelope.get_request_id();
    let envelope::Request(request) = envelope.which()? else {
        return Err(AdminWireError::Expected("administrator request"));
    };
    let request = request?;
    let command_id = request
        .get_command_id()?
        .try_into()
        .map_err(|_| AdminWireError::InvalidCommandId)?;
    let command = match request.which()? {
        request::AddBbs(add) => {
            let name = add?
                .get_name()?
                .to_str()
                .map_err(|_| AdminWireError::InvalidText)?
                .to_owned();
            if name.is_empty() || name.len() > MAX_BBS_NAME_BYTES {
                return Err(AdminWireError::InvalidBbsName);
            }
            AdminCommand::AddBbs { name }
        }
        request::InitializeUniverse(_) => AdminCommand::InitializeUniverse,
        request::Status(_) => AdminCommand::Status,
        request::LiveBackup(backup) => {
            let label = backup?
                .get_label()?
                .to_str()
                .map_err(|_| AdminWireError::InvalidText)?
                .to_owned();
            if !valid_backup_label(&label) {
                return Err(AdminWireError::InvalidBackupLabel);
            }
            AdminCommand::LiveBackup { label }
        }
    };
    Ok(AdminRequest {
        request_id,
        command_id,
        command,
    })
}

fn valid_backup_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 64
        && label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

pub fn encode_status(
    request: &AdminRequest,
    status: &OperationalStatus,
    active_sessions: u32,
) -> Result<Vec<u8>, AdminWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(status.committed_sequence);
    let mut wire = response.init_status();
    wire.set_committed_sequence(status.committed_sequence);
    wire.set_game_second(status.game_second);
    wire.set_queued_inputs(status.queued_inputs);
    wire.set_bbs_count(status.bbs_count);
    wire.set_player_count(status.player_count);
    wire.set_system_count(status.system_count);
    wire.set_active_sessions(active_sessions);
    wire.set_storage_format(status.storage_format);
    finish_message(&message)
}

pub fn encode_backup_complete(
    request: &AdminRequest,
    label: &str,
    status: &OperationalStatus,
) -> Result<Vec<u8>, AdminWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(status.committed_sequence);
    let mut complete = response.init_backup_complete();
    complete.set_label(label);
    complete.set_committed_sequence(status.committed_sequence);
    complete.set_game_second(status.game_second);
    finish_message(&message)
}

pub fn encode_bbs_added(
    request: &AdminRequest,
    credential: &BbsCredential,
) -> Result<Vec<u8>, AdminWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(credential.committed_sequence);
    let mut added = response.init_bbs_added();
    added.set_bbs_id(credential.bbs_id);
    added.set_psk(&credential.psk);
    finish_message(&message)
}

pub fn encode_universe_initialized(
    request: &AdminRequest,
    initialization: &UniverseInitialization,
) -> Result<Vec<u8>, AdminWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(initialization.committed_sequence);
    let mut initialized = response.init_universe_initialized();
    initialized.set_universe_id(&initialization.universe_id);
    initialized.set_polity_count(initialization.polity_count);
    initialized.set_system_count(initialization.system_count);
    initialized.set_world_count(initialization.world_count);
    finish_message(&message)
}

pub fn encode_invalid_request(
    request: &AdminRequest,
    message_text: &str,
) -> Result<Vec<u8>, AdminWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(0);
    let mut error = response.init_error();
    error.set_code(ErrorCode::InvalidRequest);
    error.set_message(message_text);
    finish_message(&message)
}

pub fn encode_close(reason: &str) -> Result<Vec<u8>, AdminWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.init_close().set_reason(reason);
    finish_message(&message)
}

fn finish_message(
    message: &Builder<capnp::message::HeapAllocator>,
) -> Result<Vec<u8>, AdminWireError> {
    let mut bytes = Vec::new();
    serialize::write_message(&mut bytes, message)?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(AdminWireError::FrameTooLarge);
    }
    Ok(bytes)
}

#[cfg(test)]
pub fn encode_request(request: &AdminRequest) -> Result<Vec<u8>, AdminWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut builder = envelope.init_request();
    builder.set_command_id(&request.command_id);
    match &request.command {
        AdminCommand::AddBbs { name } => builder.init_add_bbs().set_name(name),
        AdminCommand::InitializeUniverse => {
            builder.init_initialize_universe();
        }
        AdminCommand::Status => builder.set_status(()),
        AdminCommand::LiveBackup { label } => builder.init_live_backup().set_label(label),
    }
    finish_message(&message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn administrator_commands_round_trip() {
        let request = AdminRequest {
            request_id: 7,
            command_id: [0xa5; COMMAND_ID_BYTES],
            command: AdminCommand::AddBbs {
                name: "Dark Star".into(),
            },
        };
        assert_eq!(
            decode_request(&encode_request(&request).unwrap()).unwrap(),
            request
        );
        let initialize = AdminRequest {
            request_id: 8,
            command_id: [0x5a; COMMAND_ID_BYTES],
            command: AdminCommand::InitializeUniverse,
        };
        assert_eq!(
            decode_request(&encode_request(&initialize).unwrap()).unwrap(),
            initialize
        );
        for command in [
            AdminCommand::Status,
            AdminCommand::LiveBackup {
                label: "nightly-20260803".into(),
            },
        ] {
            let request = AdminRequest {
                request_id: 9,
                command_id: [0x77; COMMAND_ID_BYTES],
                command,
            };
            assert_eq!(
                decode_request(&encode_request(&request).unwrap()).unwrap(),
                request
            );
        }
    }
}
