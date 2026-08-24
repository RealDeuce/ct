//! League Coordinator protocol encoding.

use std::io::Cursor;

use capnp::message::{Builder, ReaderOptions};
use capnp::serialize;
use thiserror::Error;

use crate::ct_league_capnp::{ErrorCode, envelope, request};
use crate::store::{BbsCredential, LeagueMember, LeagueStatus};
use crate::wire::{COMMAND_ID_BYTES, MAX_FRAME_BYTES};

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_NAME_BYTES: usize = 128;
pub const MAX_REASON_BYTES: usize = 512;

#[derive(Debug, Error)]
pub enum LeagueWireError {
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
    #[error("name must contain 1..={MAX_NAME_BYTES} non-control bytes")]
    InvalidName,
    #[error("member request is invalid")]
    InvalidMember,
    #[error("frame exceeds {MAX_FRAME_BYTES} bytes")]
    FrameTooLarge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LeagueCommand {
    Status,
    SetName {
        expected_revision: u64,
        name: String,
    },
    AddBbs {
        name: String,
    },
    SetBbsAccess {
        bbs_id: u32,
        expected_revision: u64,
        enabled: bool,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeagueRequest {
    pub request_id: u64,
    pub command_id: [u8; COMMAND_ID_BYTES],
    pub command: LeagueCommand,
}

pub fn decode_protocol_version(bytes: &[u8]) -> Result<u16, LeagueWireError> {
    let message = serialize::read_message(&mut Cursor::new(bytes), ReaderOptions::new())?;
    Ok(message
        .get_root::<envelope::Reader>()?
        .get_protocol_version())
}

pub fn decode_request(bytes: &[u8]) -> Result<LeagueRequest, LeagueWireError> {
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(LeagueWireError::FrameTooLarge);
    }
    let message = serialize::read_message(&mut Cursor::new(bytes), ReaderOptions::new())?;
    let envelope = message.get_root::<envelope::Reader>()?;
    if envelope.get_protocol_version() != PROTOCOL_VERSION {
        return Err(LeagueWireError::UnsupportedVersion(
            envelope.get_protocol_version(),
        ));
    }
    let request_id = envelope.get_request_id();
    let envelope::Request(request) = envelope.which()? else {
        return Err(LeagueWireError::Expected("league request"));
    };
    let request = request?;
    let command_id = request
        .get_command_id()?
        .try_into()
        .map_err(|_| LeagueWireError::InvalidCommandId)?;
    let command = match request.which()? {
        request::Status(()) => LeagueCommand::Status,
        request::SetName(value) => {
            let value = value?;
            LeagueCommand::SetName {
                expected_revision: value.get_expected_revision(),
                name: decode_name(value.get_name()?)?,
            }
        }
        request::AddBbs(value) => LeagueCommand::AddBbs {
            name: decode_name(value?.get_name()?)?,
        },
        request::DisableBbs(value) | request::EnableBbs(value) => {
            let enabled = matches!(request.which()?, request::EnableBbs(_));
            let value = value?;
            let reason = value
                .get_reason()?
                .to_str()
                .map_err(|_| LeagueWireError::InvalidText)?
                .to_owned();
            if value.get_bbs_id() == 0
                || reason.len() > MAX_REASON_BYTES
                || reason.chars().any(char::is_control)
                || (!enabled && reason.is_empty())
            {
                return Err(LeagueWireError::InvalidMember);
            }
            LeagueCommand::SetBbsAccess {
                bbs_id: value.get_bbs_id(),
                expected_revision: value.get_expected_revision(),
                enabled,
                reason,
            }
        }
    };
    Ok(LeagueRequest {
        request_id,
        command_id,
        command,
    })
}

fn decode_name(value: capnp::text::Reader<'_>) -> Result<String, LeagueWireError> {
    let name = value
        .to_str()
        .map_err(|_| LeagueWireError::InvalidText)?
        .to_owned();
    if name.is_empty()
        || name.len() > MAX_NAME_BYTES
        || name.chars().any(char::is_control)
        || name.trim() != name
    {
        return Err(LeagueWireError::InvalidName);
    }
    Ok(name)
}

fn set_member(mut wire: crate::ct_league_capnp::league_member::Builder<'_>, member: &LeagueMember) {
    wire.set_bbs_id(member.bbs_id);
    wire.set_bbs_name(&member.bbs_name);
    wire.set_enabled(member.enabled);
    wire.set_reason(&member.reason);
    wire.set_revision(member.revision);
}

fn set_status(mut wire: crate::ct_league_capnp::league_status::Builder<'_>, status: &LeagueStatus) {
    wire.set_league_id(status.league_id);
    wire.set_name(&status.name);
    wire.set_revision(status.revision);
    let mut members = wire.init_members(status.members.len() as u32);
    for (index, member) in status.members.iter().enumerate() {
        set_member(members.reborrow().get(index as u32), member);
    }
}

pub fn encode_status(
    request: &LeagueRequest,
    status: &LeagueStatus,
) -> Result<Vec<u8>, LeagueWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(status.committed_sequence);
    set_status(response.init_status(), status);
    finish_message(&message)
}

pub fn encode_name_set(
    request: &LeagueRequest,
    status: &LeagueStatus,
    stale: bool,
) -> Result<Vec<u8>, LeagueWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(status.committed_sequence);
    if stale {
        set_status(response.reborrow().init_stale(), status);
    } else {
        set_status(response.reborrow().init_name_set(), status);
    }
    finish_message(&message)
}

pub fn encode_bbs_added(
    request: &LeagueRequest,
    credential: &BbsCredential,
) -> Result<Vec<u8>, LeagueWireError> {
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

pub fn encode_member_updated(
    request: &LeagueRequest,
    member: &LeagueMember,
    stale: bool,
) -> Result<Vec<u8>, LeagueWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(member.committed_sequence);
    if stale {
        set_member(response.reborrow().init_stale_member(), member);
    } else {
        set_member(response.reborrow().init_member_updated(), member);
    }
    finish_message(&message)
}

pub fn encode_error(request: &LeagueRequest, text: &str) -> Result<Vec<u8>, LeagueWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(0);
    let mut error = response.init_error();
    error.set_code(ErrorCode::InvalidRequest);
    error.set_message(text);
    finish_message(&message)
}

pub fn encode_close(
    code: crate::ct_league_capnp::CloseCode,
    text: &str,
) -> Result<Vec<u8>, LeagueWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    let mut close = envelope.init_close();
    close.set_code(code);
    close.set_message(text);
    finish_message(&message)
}

fn finish_message(
    message: &Builder<capnp::message::HeapAllocator>,
) -> Result<Vec<u8>, LeagueWireError> {
    let mut bytes = Vec::new();
    serialize::write_message(&mut bytes, message)?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(LeagueWireError::FrameTooLarge);
    }
    Ok(bytes)
}
