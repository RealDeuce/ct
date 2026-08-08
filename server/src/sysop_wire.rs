//! BBS-sysop protocol encoding.

use std::io::Cursor;

use capnp::message::{Builder, ReaderOptions};
use capnp::serialize;
use thiserror::Error;

use crate::ct_sysop_capnp::{ErrorCode, envelope, request};
use crate::store::{
    BbsConfiguration, BbsSettings, PlayerAccessRecord, PlayerAccessState, SysopDirectiveKind,
    SysopDirectiveRecord,
};
use crate::wire::{COMMAND_ID_BYTES, CloseCode, MAX_FRAME_BYTES};

pub const PROTOCOL_VERSION: u16 = 2;

pub const MAX_DISPLAY_NAME_BYTES: usize = 128;

#[derive(Debug, Error)]
pub enum SysopWireError {
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
    #[error("BBS and polity names must contain 1..={MAX_DISPLAY_NAME_BYTES} non-control bytes")]
    InvalidName,
    #[error("orientation values must be in the inclusive range 0..=100")]
    InvalidOrientation,
    #[error("player ID must be nonzero and the reason must contain at most 512 non-control bytes")]
    InvalidPlayerAccess,
    #[error("frame exceeds {MAX_FRAME_BYTES} bytes")]
    FrameTooLarge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SysopCommand {
    GetConfiguration,
    SetConfiguration {
        expected_revision: u64,
        settings: BbsSettings,
    },
    GetPlayerAccess {
        player_id: u32,
    },
    SetPlayerAccess {
        player_id: u32,
        expected_revision: u64,
        state: PlayerAccessState,
        reason: String,
    },
    IssueDirective {
        player_id: u32,
        kind: SysopDirectiveKind,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SysopRequest {
    pub request_id: u64,
    pub command_id: [u8; COMMAND_ID_BYTES],
    pub command: SysopCommand,
}

pub fn decode_protocol_version(bytes: &[u8]) -> Result<u16, SysopWireError> {
    let message = serialize::read_message(&mut Cursor::new(bytes), ReaderOptions::new())?;
    Ok(message
        .get_root::<envelope::Reader>()?
        .get_protocol_version())
}

pub fn decode_client_hello_with_version(bytes: &[u8]) -> Result<(u16, String), SysopWireError> {
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(SysopWireError::FrameTooLarge);
    }
    let message = serialize::read_message(&mut Cursor::new(bytes), ReaderOptions::new())?;
    let envelope = message.get_root::<envelope::Reader>()?;
    let version = envelope.get_protocol_version();
    let envelope::ClientHello(hello) = envelope.which()? else {
        return Err(SysopWireError::Expected("sysop client hello"));
    };
    let language = hello?
        .get_language_tag()?
        .to_str()
        .map_err(|_| SysopWireError::InvalidText)?
        .to_owned();
    Ok((version, language))
}

pub fn encode_server_hello(language_tag: &str) -> Result<Vec<u8>, SysopWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.init_server_hello().set_language_tag(language_tag);
    finish_message(&message)
}

pub fn decode_request(bytes: &[u8]) -> Result<SysopRequest, SysopWireError> {
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(SysopWireError::FrameTooLarge);
    }
    let mut options = ReaderOptions::new();
    options
        .traversal_limit_in_words(Some(MAX_FRAME_BYTES / 8))
        .nesting_limit(24);
    let message = serialize::read_message(&mut Cursor::new(bytes), options)?;
    let envelope = message.get_root::<envelope::Reader>()?;
    if envelope.get_protocol_version() != PROTOCOL_VERSION {
        return Err(SysopWireError::UnsupportedVersion(
            envelope.get_protocol_version(),
        ));
    }
    let request_id = envelope.get_request_id();
    let envelope::Request(request) = envelope.which()? else {
        return Err(SysopWireError::Expected("sysop request"));
    };
    let request = request?;
    let command_id = request
        .get_command_id()?
        .try_into()
        .map_err(|_| SysopWireError::InvalidCommandId)?;
    let command = match request.which()? {
        request::GetConfiguration(()) => SysopCommand::GetConfiguration,
        request::SetConfiguration(set) => {
            let set = set?;
            let settings = set.get_settings()?;
            let bbs_name = decode_name(settings.get_bbs_name()?)?;
            let polity_name = decode_name(settings.get_polity_name()?)?;
            let trade_combat = settings.get_trade_combat();
            let chaos_order = settings.get_chaos_order();
            if trade_combat > 100 || chaos_order > 100 {
                return Err(SysopWireError::InvalidOrientation);
            }
            SysopCommand::SetConfiguration {
                expected_revision: set.get_expected_revision(),
                settings: BbsSettings {
                    bbs_name,
                    polity_name,
                    trade_combat,
                    chaos_order,
                },
            }
        }
        request::GetPlayerAccess(target) => {
            let player_id = target?.get_player_id();
            if player_id == 0 {
                return Err(SysopWireError::InvalidPlayerAccess);
            }
            SysopCommand::GetPlayerAccess { player_id }
        }
        request::SetPlayerAccess(set) => {
            let set = set?;
            let player_id = set.get_player_id();
            let reason = set
                .get_reason()?
                .to_str()
                .map_err(|_| SysopWireError::InvalidText)?
                .to_owned();
            if player_id == 0 || reason.len() > 512 || reason.chars().any(char::is_control) {
                return Err(SysopWireError::InvalidPlayerAccess);
            }
            let state = match set.get_state()? {
                crate::ct_sysop_capnp::PlayerAccessState::Active => PlayerAccessState::Active,
                crate::ct_sysop_capnp::PlayerAccessState::Suspended => PlayerAccessState::Suspended,
                crate::ct_sysop_capnp::PlayerAccessState::Removed => PlayerAccessState::Removed,
            };
            SysopCommand::SetPlayerAccess {
                player_id,
                expected_revision: set.get_expected_revision(),
                state,
                reason,
            }
        }
        request::TaxPlayer(tax) => {
            let tax = tax?;
            if tax.get_player_id() == 0 || tax.get_credits() == 0 {
                return Err(SysopWireError::InvalidPlayerAccess);
            }
            SysopCommand::IssueDirective {
                player_id: tax.get_player_id(),
                kind: SysopDirectiveKind::Tax {
                    credits: tax.get_credits(),
                },
            }
        }
        request::DemotePlayer(demotion) => {
            let demotion = demotion?;
            if demotion.get_player_id() == 0 {
                return Err(SysopWireError::InvalidPlayerAccess);
            }
            SysopCommand::IssueDirective {
                player_id: demotion.get_player_id(),
                kind: SysopDirectiveKind::Demote {
                    naval_grade_index: demotion.get_naval_grade_index(),
                },
            }
        }
    };
    Ok(SysopRequest {
        request_id,
        command_id,
        command,
    })
}

pub fn encode_directive_issued(
    request: &SysopRequest,
    directive: &SysopDirectiveRecord,
) -> Result<Vec<u8>, SysopWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(directive.committed_sequence);
    let mut issued = response.init_directive_issued();
    issued.set_player_id(directive.identity.player_id);
    issued.set_message_id(directive.message_id);
    issued.set_issued_second(directive.issued_second);
    match directive.kind {
        SysopDirectiveKind::Tax { credits } => issued.set_tax_credits(credits),
        SysopDirectiveKind::Demote { naval_grade_index } => {
            issued.set_naval_grade_index(naval_grade_index)
        }
    }
    finish_message(&message)
}

pub fn encode_player_access(
    request: &SysopRequest,
    access: &PlayerAccessRecord,
) -> Result<Vec<u8>, SysopWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(access.committed_sequence);
    set_player_access(response.init_player_access(), access);
    finish_message(&message)
}

pub fn encode_player_access_stale(
    request: &SysopRequest,
    access: &PlayerAccessRecord,
) -> Result<Vec<u8>, SysopWireError> {
    encode_access_error(
        request,
        access,
        ErrorCode::StaleRevision,
        "player access revision is stale",
    )
}

pub fn encode_player_permanently_removed(
    request: &SysopRequest,
    access: &PlayerAccessRecord,
) -> Result<Vec<u8>, SysopWireError> {
    encode_access_error(
        request,
        access,
        ErrorCode::PermanentlyRemoved,
        "a permanently removed player cannot be restored",
    )
}

fn encode_access_error(
    request: &SysopRequest,
    access: &PlayerAccessRecord,
    code: ErrorCode,
    message_text: &str,
) -> Result<Vec<u8>, SysopWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(access.committed_sequence);
    let mut error = response.init_error();
    error.set_code(code);
    error.set_message(message_text);
    error.set_current_revision(access.revision);
    finish_message(&message)
}

fn set_player_access(
    mut builder: crate::ct_sysop_capnp::player_access::Builder<'_>,
    access: &PlayerAccessRecord,
) {
    builder.set_player_id(access.identity.player_id);
    builder.set_revision(access.revision);
    builder.set_state(match access.state {
        PlayerAccessState::Active => crate::ct_sysop_capnp::PlayerAccessState::Active,
        PlayerAccessState::Suspended => crate::ct_sysop_capnp::PlayerAccessState::Suspended,
        PlayerAccessState::Removed => crate::ct_sysop_capnp::PlayerAccessState::Removed,
    });
    builder.set_reason(&access.reason);
}

pub fn encode_configuration(
    request: &SysopRequest,
    configuration: &BbsConfiguration,
) -> Result<Vec<u8>, SysopWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(configuration.committed_sequence);
    set_configuration(response.init_configuration(), configuration);
    finish_message(&message)
}

pub fn encode_stale_revision(
    request: &SysopRequest,
    configuration: &BbsConfiguration,
) -> Result<Vec<u8>, SysopWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(configuration.committed_sequence);
    let mut error = response.init_error();
    error.set_code(ErrorCode::StaleRevision);
    error.set_message("BBS configuration revision is stale");
    error.set_current_revision(configuration.revision);
    finish_message(&message)
}

pub fn encode_no_eligible_site(
    request: &SysopRequest,
    configuration: &BbsConfiguration,
) -> Result<Vec<u8>, SysopWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut response = envelope.init_response();
    response.set_command_id(&request.command_id);
    response.set_committed_sequence(configuration.committed_sequence);
    let mut error = response.init_error();
    error.set_code(ErrorCode::NoEligibleSite);
    error.set_message("no eligible unexplored site can contain a BBS polity");
    error.set_current_revision(configuration.revision);
    finish_message(&message)
}

pub fn encode_invalid_request(
    request: &SysopRequest,
    message_text: &str,
) -> Result<Vec<u8>, SysopWireError> {
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

pub fn encode_close(
    code: CloseCode,
    text: &str,
    languages: &[&str],
) -> Result<Vec<u8>, SysopWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    let mut close = envelope.init_close();
    close.set_code(match code {
        CloseCode::Unspecified => crate::ct_sysop_capnp::CloseCode::Unspecified,
        CloseCode::UnsupportedVersion => crate::ct_sysop_capnp::CloseCode::UnsupportedVersion,
        CloseCode::MalformedHello => crate::ct_sysop_capnp::CloseCode::MalformedHello,
        CloseCode::UnsupportedLanguage => crate::ct_sysop_capnp::CloseCode::UnsupportedLanguage,
        CloseCode::AccessDenied => crate::ct_sysop_capnp::CloseCode::AccessDenied,
        CloseCode::InvalidRequest => crate::ct_sysop_capnp::CloseCode::InvalidRequest,
        CloseCode::StaleSession => crate::ct_sysop_capnp::CloseCode::StaleSession,
        CloseCode::ServerStopping => crate::ct_sysop_capnp::CloseCode::ServerStopping,
        CloseCode::SessionReplaced => crate::ct_sysop_capnp::CloseCode::SessionReplaced,
        CloseCode::InternalFailure => crate::ct_sysop_capnp::CloseCode::InternalFailure,
    });
    close.set_message(text);
    let mut supported = close
        .reborrow()
        .init_supported_language_tags(languages.len() as u32);
    for (index, language) in languages.iter().enumerate() {
        supported.set(index as u32, language);
    }
    finish_message(&message)
}

pub fn encode_legacy_close(protocol_version: u16, reason: &str) -> Result<Vec<u8>, SysopWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<crate::ct_sysop_capnp::legacy_v1_envelope::Builder>();
    envelope.set_protocol_version(protocol_version);
    envelope.init_close().set_reason(reason);
    finish_message(&message)
}

fn decode_name(name: capnp::text::Reader<'_>) -> Result<String, SysopWireError> {
    let name = name
        .to_str()
        .map_err(|_| SysopWireError::InvalidText)?
        .to_owned();
    if name.is_empty() || name.len() > MAX_DISPLAY_NAME_BYTES || name.chars().any(char::is_control)
    {
        return Err(SysopWireError::InvalidName);
    }
    Ok(name)
}

fn set_configuration(
    mut builder: crate::ct_sysop_capnp::bbs_configuration::Builder<'_>,
    configuration: &BbsConfiguration,
) {
    builder.set_bbs_id(configuration.bbs_id);
    builder.set_revision(configuration.revision);
    builder.set_configured(configuration.configured);
    let mut settings = builder.init_settings();
    settings.set_bbs_name(&configuration.settings.bbs_name);
    settings.set_polity_name(&configuration.settings.polity_name);
    settings.set_trade_combat(configuration.settings.trade_combat);
    settings.set_chaos_order(configuration.settings.chaos_order);
}

fn finish_message(
    message: &Builder<capnp::message::HeapAllocator>,
) -> Result<Vec<u8>, SysopWireError> {
    let mut bytes = Vec::new();
    serialize::write_message(&mut bytes, message)?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(SysopWireError::FrameTooLarge);
    }
    Ok(bytes)
}

#[cfg(test)]
pub fn encode_request(request: &SysopRequest) -> Result<Vec<u8>, SysopWireError> {
    let mut message = Builder::new_default();
    let mut envelope = message.init_root::<envelope::Builder>();
    envelope.set_protocol_version(PROTOCOL_VERSION);
    envelope.set_request_id(request.request_id);
    let mut builder = envelope.init_request();
    builder.set_command_id(&request.command_id);
    match &request.command {
        SysopCommand::GetConfiguration => builder.set_get_configuration(()),
        SysopCommand::SetConfiguration {
            expected_revision,
            settings,
        } => {
            let mut set = builder.init_set_configuration();
            set.set_expected_revision(*expected_revision);
            let mut wire_settings = set.init_settings();
            wire_settings.set_bbs_name(&settings.bbs_name);
            wire_settings.set_polity_name(&settings.polity_name);
            wire_settings.set_trade_combat(settings.trade_combat);
            wire_settings.set_chaos_order(settings.chaos_order);
        }
        SysopCommand::GetPlayerAccess { player_id } => {
            builder.init_get_player_access().set_player_id(*player_id);
        }
        SysopCommand::SetPlayerAccess {
            player_id,
            expected_revision,
            state,
            reason,
        } => {
            let mut set = builder.init_set_player_access();
            set.set_player_id(*player_id);
            set.set_expected_revision(*expected_revision);
            set.set_state(match state {
                PlayerAccessState::Active => crate::ct_sysop_capnp::PlayerAccessState::Active,
                PlayerAccessState::Suspended => crate::ct_sysop_capnp::PlayerAccessState::Suspended,
                PlayerAccessState::Removed => crate::ct_sysop_capnp::PlayerAccessState::Removed,
            });
            set.set_reason(reason);
        }
        SysopCommand::IssueDirective { player_id, kind } => match kind {
            SysopDirectiveKind::Tax { credits } => {
                let mut tax = builder.init_tax_player();
                tax.set_player_id(*player_id);
                tax.set_credits(*credits);
            }
            SysopDirectiveKind::Demote { naval_grade_index } => {
                let mut demotion = builder.init_demote_player();
                demotion.set_player_id(*player_id);
                demotion.set_naval_grade_index(*naval_grade_index);
            }
        },
    }
    finish_message(&message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sysop_hello_carries_language_and_version() {
        let mut message = Builder::new_default();
        let mut envelope = message.init_root::<envelope::Builder>();
        envelope.set_protocol_version(PROTOCOL_VERSION);
        envelope.init_client_hello().set_language_tag("en-US");
        let frame = finish_message(&message).unwrap();
        assert_eq!(
            decode_client_hello_with_version(&frame).unwrap(),
            (PROTOCOL_VERSION, "en-US".into())
        );
    }

    #[test]
    fn set_configuration_round_trips() {
        let request = SysopRequest {
            request_id: 7,
            command_id: [0x5a; COMMAND_ID_BYTES],
            command: SysopCommand::SetConfiguration {
                expected_revision: 3,
                settings: BbsSettings {
                    bbs_name: "Dark Star".into(),
                    polity_name: "Far Reach".into(),
                    trade_combat: 65,
                    chaos_order: 25,
                },
            },
        };
        assert_eq!(
            decode_request(&encode_request(&request).unwrap()).unwrap(),
            request
        );
    }

    #[test]
    fn set_configuration_rejects_out_of_range_orientation() {
        let request = SysopRequest {
            request_id: 8,
            command_id: [0x6b; COMMAND_ID_BYTES],
            command: SysopCommand::SetConfiguration {
                expected_revision: 0,
                settings: BbsSettings {
                    bbs_name: "Dark Star".into(),
                    polity_name: "Far Reach".into(),
                    trade_combat: 101,
                    chaos_order: 25,
                },
            },
        };
        assert!(matches!(
            decode_request(&encode_request(&request).unwrap()),
            Err(SysopWireError::InvalidOrientation)
        ));
    }

    #[test]
    fn player_access_commands_round_trip() {
        for command in [
            SysopCommand::GetPlayerAccess { player_id: 77 },
            SysopCommand::SetPlayerAccess {
                player_id: 77,
                expected_revision: 4,
                state: PlayerAccessState::Suspended,
                reason: "Local conduct review".into(),
            },
        ] {
            let request = SysopRequest {
                request_id: 9,
                command_id: [0x44; COMMAND_ID_BYTES],
                command,
            };
            assert_eq!(
                decode_request(&encode_request(&request).unwrap()).unwrap(),
                request
            );
        }
    }
}
