#include "ct/sysop_protocol.hpp"

#include "ct/tls_connection.hpp"
#include "ct_sysop.capnp.h"

#include <capnp/message.h>
#include <capnp/serialize.h>
#include <kj/array.h>

#include <algorithm>
#include <array>
#include <cstring>
#include <limits>
#include <span>
#include <stdexcept>
#include <vector>

namespace ct {
namespace {

constexpr uint16_t PROTOCOL_VERSION = 1;
constexpr size_t MAX_FRAME_BYTES = 1024 * 1024;
constexpr size_t MAX_NAME_BYTES = 128;

void send_frame(TlsConnection& connection, const kj::ArrayPtr<const kj::byte> message) {
   if(message.size() == 0 || message.size() > MAX_FRAME_BYTES ||
      message.size() > std::numeric_limits<uint32_t>::max()) {
      throw std::runtime_error("invalid outgoing sysop frame size");
   }
   const auto size = static_cast<uint32_t>(message.size());
   const std::array<uint8_t, 4> header = {
      static_cast<uint8_t>(size >> 24),
      static_cast<uint8_t>(size >> 16),
      static_cast<uint8_t>(size >> 8),
      static_cast<uint8_t>(size),
   };
   connection.send(header);
   connection.send(
      std::span(reinterpret_cast<const uint8_t*>(message.begin()), message.size()));
}

std::vector<uint8_t> receive_frame(TlsConnection& connection) {
   const auto header = connection.receive_exact(4);
   const auto size = (static_cast<uint32_t>(header[0]) << 24) |
                     (static_cast<uint32_t>(header[1]) << 16) |
                     (static_cast<uint32_t>(header[2]) << 8) |
                     static_cast<uint32_t>(header[3]);
   if(size == 0 || size > MAX_FRAME_BYTES) {
      throw std::runtime_error("invalid incoming sysop frame size");
   }
   return connection.receive_exact(size);
}

void validate_name(const std::string& name, const char* description) {
   if(name.empty() || name.size() > MAX_NAME_BYTES ||
      std::any_of(name.begin(), name.end(), [](const unsigned char byte) {
         return byte < 0x20 || byte == 0x7f;
      })) {
      throw std::invalid_argument(
         std::string(description) + " must contain 1..128 non-control bytes");
   }
}

SysopConfiguration parse_response(TlsConnection& connection,
                                  const uint64_t request_id,
                                  const std::array<uint8_t, 16>& command_id) {
   const auto frame = receive_frame(connection);
   const auto word_count =
      (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
   auto response_words = kj::heapArray<capnp::word>(word_count);
   std::memset(response_words.begin(), 0, response_words.asBytes().size());
   std::memcpy(response_words.asBytes().begin(), frame.data(), frame.size());
   capnp::FlatArrayMessageReader reader(response_words);
   const auto envelope = reader.getRoot<sysop::Envelope>();
   if(envelope.getProtocolVersion() != PROTOCOL_VERSION ||
      envelope.getRequestId() != request_id ||
      !envelope.isResponse()) {
      throw std::runtime_error("invalid sysop response envelope");
   }
   const auto response = envelope.getResponse();
   const auto returned_command_id = response.getCommandId();
   if(returned_command_id.size() != command_id.size() ||
      !std::equal(returned_command_id.begin(),
                  returned_command_id.end(),
                  command_id.begin())) {
      throw std::runtime_error("sysop response command ID mismatch");
   }
   if(response.isError()) {
      const auto error = response.getError();
      if(error.getCode() == sysop::ErrorCode::STALE_REVISION) {
         throw StaleConfiguration(error.getCurrentRevision());
      }
      throw std::runtime_error(error.getMessage().cStr());
   }
   if(!response.isConfiguration()) {
      throw std::runtime_error("expected a BBS-configuration response");
   }
   const auto configuration = response.getConfiguration();
   const auto settings = configuration.getSettings();
   return SysopConfiguration{
      .bbs_id = configuration.getBbsId(),
      .revision = configuration.getRevision(),
      .configured = configuration.getConfigured(),
      .settings =
         {
            .bbs_name = settings.getBbsName().cStr(),
            .polity_name = settings.getPolityName().cStr(),
            .trade_combat = settings.getTradeCombat(),
            .chaos_order = settings.getChaosOrder(),
         },
      .committed_sequence = response.getCommittedSequence(),
   };
}

PlayerAccess parse_access_response(
   TlsConnection& connection,
   const uint64_t request_id,
   const std::array<uint8_t, 16>& command_id) {
   const auto frame = receive_frame(connection);
   const auto word_count =
      (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
   auto response_words = kj::heapArray<capnp::word>(word_count);
   std::memset(response_words.begin(), 0, response_words.asBytes().size());
   std::memcpy(response_words.asBytes().begin(), frame.data(), frame.size());
   capnp::FlatArrayMessageReader reader(response_words);
   const auto envelope = reader.getRoot<sysop::Envelope>();
   if(envelope.getProtocolVersion() != PROTOCOL_VERSION ||
      envelope.getRequestId() != request_id || !envelope.isResponse()) {
      throw std::runtime_error("invalid sysop response envelope");
   }
   const auto response = envelope.getResponse();
   const auto returned_command_id = response.getCommandId();
   if(returned_command_id.size() != command_id.size() ||
      !std::equal(returned_command_id.begin(), returned_command_id.end(),
                  command_id.begin())) {
      throw std::runtime_error("sysop response command ID mismatch");
   }
   if(response.isError()) {
      const auto error = response.getError();
      if(error.getCode() == sysop::ErrorCode::STALE_REVISION) {
         throw StaleConfiguration(error.getCurrentRevision());
      }
      throw std::runtime_error(error.getMessage().cStr());
   }
   if(!response.isPlayerAccess()) {
      throw std::runtime_error("expected a player-access response");
   }
   const auto access = response.getPlayerAccess();
   const auto state = [&access]() -> PlayerAccessState {
      switch(access.getState()) {
         case sysop::PlayerAccessState::ACTIVE:
            return PlayerAccessState::Active;
         case sysop::PlayerAccessState::SUSPENDED:
            return PlayerAccessState::Suspended;
         case sysop::PlayerAccessState::REMOVED:
            return PlayerAccessState::Removed;
      }
      throw std::runtime_error("unknown player-access state");
   }();
   return PlayerAccess{
      .player_id = access.getPlayerId(),
      .revision = access.getRevision(),
      .state = state,
      .reason = access.getReason().cStr(),
      .committed_sequence = response.getCommittedSequence(),
   };
}

DirectiveIssued parse_directive_response(
   TlsConnection& connection,
   const uint64_t request_id,
   const std::array<uint8_t, 16>& command_id) {
   const auto frame = receive_frame(connection);
   const auto word_count =
      (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
   auto words = kj::heapArray<capnp::word>(word_count);
   std::memset(words.begin(), 0, words.asBytes().size());
   std::memcpy(words.asBytes().begin(), frame.data(), frame.size());
   capnp::FlatArrayMessageReader reader(words);
   const auto envelope = reader.getRoot<sysop::Envelope>();
   if(envelope.getProtocolVersion() != PROTOCOL_VERSION ||
      envelope.getRequestId() != request_id || !envelope.isResponse()) {
      throw std::runtime_error("invalid sysop directive response");
   }
   const auto response = envelope.getResponse();
   const auto returned_command_id = response.getCommandId();
   if(returned_command_id.size() != command_id.size() ||
      !std::equal(returned_command_id.begin(), returned_command_id.end(),
                  command_id.begin())) {
      throw std::runtime_error("sysop directive response command ID mismatch");
   }
   if(response.isError()) {
      throw std::runtime_error(response.getError().getMessage().cStr());
   }
   if(!response.isDirectiveIssued()) {
      throw std::runtime_error("expected a directive-issued response");
   }
   const auto issued = response.getDirectiveIssued();
   const bool is_tax = issued.isTaxCredits();
   return DirectiveIssued{
      .player_id = issued.getPlayerId(),
      .message_id = issued.getMessageId(),
      .issued_second = issued.getIssuedSecond(),
      .is_tax = is_tax,
      .value = is_tax ? issued.getTaxCredits() : issued.getNavalGradeIndex(),
      .committed_sequence = response.getCommittedSequence(),
   };
}

}  // namespace

StaleConfiguration::StaleConfiguration(const uint64_t current_revision) :
      std::runtime_error(
         "BBS configuration revision is stale; current revision is " +
         std::to_string(current_revision)),
      m_current_revision(current_revision) {}

uint64_t StaleConfiguration::current_revision() const noexcept {
   return m_current_revision;
}

SysopConfiguration get_bbs_configuration(TlsConnection& connection,
                                         const uint64_t request_id) {
   const std::array<uint8_t, 16> command_id{};
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<sysop::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   request.setGetConfiguration();
   const auto words = capnp::messageToFlatArray(message);
   send_frame(connection, words.asBytes());
   return parse_response(connection, request_id, command_id);
}

SysopConfiguration set_bbs_configuration(
   TlsConnection& connection,
   const uint64_t expected_revision,
   const SysopSettings& settings,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id) {
   validate_name(settings.bbs_name, "BBS name");
   validate_name(settings.polity_name, "polity name");
   if(settings.trade_combat > 100 || settings.chaos_order > 100) {
      throw std::invalid_argument("orientation values must be in 0..100");
   }
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<sysop::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   auto set = request.initSetConfiguration();
   set.setExpectedRevision(expected_revision);
   auto wire_settings = set.initSettings();
   wire_settings.setBbsName(settings.bbs_name);
   wire_settings.setPolityName(settings.polity_name);
   wire_settings.setTradeCombat(settings.trade_combat);
   wire_settings.setChaosOrder(settings.chaos_order);
   const auto words = capnp::messageToFlatArray(message);
   send_frame(connection, words.asBytes());
   return parse_response(connection, request_id, command_id);
}

PlayerAccess get_player_access(TlsConnection& connection,
                               const uint32_t player_id,
                               const uint64_t request_id) {
   if(player_id == 0) {
      throw std::invalid_argument("player ID must be nonzero");
   }
   const std::array<uint8_t, 16> command_id{};
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<sysop::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   request.initGetPlayerAccess().setPlayerId(player_id);
   const auto words = capnp::messageToFlatArray(message);
   send_frame(connection, words.asBytes());
   return parse_access_response(connection, request_id, command_id);
}

PlayerAccess set_player_access(
   TlsConnection& connection,
   const uint32_t player_id,
   const uint64_t expected_revision,
   const PlayerAccessState state,
   const std::string& reason,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id) {
   if(player_id == 0 || reason.size() > 512 ||
      std::any_of(reason.begin(), reason.end(), [](const unsigned char byte) {
         return byte < 0x20 || byte == 0x7f;
      })) {
      throw std::invalid_argument("invalid player access target or reason");
   }
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<sysop::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   auto set = request.initSetPlayerAccess();
   set.setPlayerId(player_id);
   set.setExpectedRevision(expected_revision);
   set.setState([state] {
      switch(state) {
         case PlayerAccessState::Active:
            return sysop::PlayerAccessState::ACTIVE;
         case PlayerAccessState::Suspended:
            return sysop::PlayerAccessState::SUSPENDED;
         case PlayerAccessState::Removed:
            return sysop::PlayerAccessState::REMOVED;
      }
      return sysop::PlayerAccessState::ACTIVE;
   }());
   set.setReason(reason);
   const auto words = capnp::messageToFlatArray(message);
   send_frame(connection, words.asBytes());
   return parse_access_response(connection, request_id, command_id);
}

DirectiveIssued tax_player(
   TlsConnection& connection,
   const uint32_t player_id,
   const uint64_t credits,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id) {
   if(player_id == 0 || credits == 0) {
      throw std::invalid_argument("player ID and tax must be nonzero");
   }
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<sysop::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   auto tax = request.initTaxPlayer();
   tax.setPlayerId(player_id);
   tax.setCredits(credits);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   return parse_directive_response(connection, request_id, command_id);
}

DirectiveIssued demote_player(
   TlsConnection& connection,
   const uint32_t player_id,
   const uint8_t naval_grade_index,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id) {
   if(player_id == 0) {
      throw std::invalid_argument("player ID must be nonzero");
   }
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<sysop::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   auto demotion = request.initDemotePlayer();
   demotion.setPlayerId(player_id);
   demotion.setNavalGradeIndex(naval_grade_index);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   return parse_directive_response(connection, request_id, command_id);
}

}  // namespace ct
