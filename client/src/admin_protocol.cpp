#include "ct/admin_protocol.hpp"

#include "ct/protocol.hpp"
#include "ct/tls_connection.hpp"
#include "ct_admin.capnp.h"

#include <capnp/message.h>
#include <capnp/serialize.h>
#include <kj/array.h>

#include <algorithm>
#include <array>
#include <cctype>
#include <cstring>
#include <limits>
#include <span>
#include <stdexcept>
#include <vector>

namespace ct {
namespace {

constexpr uint16_t PROTOCOL_VERSION = 2;
constexpr size_t MAX_FRAME_BYTES = 1024 * 1024;

void send_frame(TlsConnection& connection, const kj::ArrayPtr<const kj::byte> message) {
   if(message.size() == 0 || message.size() > MAX_FRAME_BYTES ||
      message.size() > std::numeric_limits<uint32_t>::max()) {
      throw std::runtime_error("invalid outgoing administrator frame size");
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
      throw std::runtime_error("invalid incoming administrator frame size");
   }
   return connection.receive_exact(size);
}

}  // namespace

void exchange_admin_hello(TlsConnection& connection,
                          const std::string& language_tag) {
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<admin::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.initClientHello().setLanguageTag(language_tag);
   const auto words = capnp::messageToFlatArray(message);
   send_frame(connection, words.asBytes());

   const auto frame = receive_frame(connection);
   const auto word_count =
      (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
   auto response_words = kj::heapArray<capnp::word>(word_count);
   std::memset(response_words.begin(), 0, response_words.asBytes().size());
   std::memcpy(response_words.asBytes().begin(), frame.data(), frame.size());
   capnp::FlatArrayMessageReader reader(response_words);
   const auto response = reader.getRoot<admin::Envelope>();
   if(response.isClose()) {
      const auto close = response.getClose();
      if(close.hasMessage() && close.getMessage().size() != 0) {
         throw std::runtime_error(close.getMessage().cStr());
      }
      const auto legacy = reader.getRoot<admin::LegacyV1Envelope>();
      if(legacy.isClose()) {
         throw std::runtime_error(legacy.getClose().getReason().cStr());
      }
      throw std::runtime_error("administrator connection closed during negotiation");
   }
   if(response.getProtocolVersion() != PROTOCOL_VERSION) {
      const auto legacy = reader.getRoot<admin::LegacyV1Envelope>();
      if(legacy.isClose()) {
         throw std::runtime_error(legacy.getClose().getReason().cStr());
      }
      throw std::runtime_error("server selected an unsupported administrator protocol version");
   }
   if(!response.isServerHello() ||
      !language_selection_matches(
         language_tag,
         response.getServerHello().getLanguageTag().cStr())) {
      throw std::runtime_error("expected a valid administrator ServerHello");
   }
}

BbsCredentials add_bbs(TlsConnection& connection,
                       const std::string& name,
                       const std::array<uint8_t, 16>& command_id,
                       const uint64_t request_id) {
   if(name.empty() || name.size() > 128) {
      throw std::invalid_argument("BBS name must contain 1..128 bytes");
   }
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<admin::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   request.initAddBbs().setName(name);
   const auto words = capnp::messageToFlatArray(message);
   send_frame(connection, words.asBytes());

   const auto frame = receive_frame(connection);
   const auto word_count =
      (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
   auto response_words = kj::heapArray<capnp::word>(word_count);
   std::memset(response_words.begin(), 0, response_words.asBytes().size());
   std::memcpy(response_words.asBytes().begin(), frame.data(), frame.size());
   capnp::FlatArrayMessageReader reader(response_words);
   const auto response_envelope = reader.getRoot<admin::Envelope>();
   if(response_envelope.getProtocolVersion() != PROTOCOL_VERSION ||
      response_envelope.getRequestId() != request_id ||
      !response_envelope.isResponse()) {
      throw std::runtime_error("invalid administrator response envelope");
   }
   const auto response = response_envelope.getResponse();
   const auto returned_command_id = response.getCommandId();
   if(returned_command_id.size() != command_id.size() ||
      !std::equal(returned_command_id.begin(),
                  returned_command_id.end(),
                  command_id.begin())) {
      throw std::runtime_error("administrator response command ID mismatch");
   }
   if(response.isError()) {
      const auto error = response.getError();
      if(error.getCode() == admin::ErrorCode::INVALID_REQUEST) {
         throw AdministratorRequestRejected(error.getMessage().cStr());
      }
      throw std::runtime_error(error.getMessage().cStr());
   }
   if(!response.isBbsAdded()) {
      throw std::runtime_error("expected a BBS-added response");
   }
   const auto added = response.getBbsAdded();
   const auto psk = added.getPsk();
   if(psk.size() != 32) {
      throw std::runtime_error("server returned an invalid BBS PSK");
   }
   BbsCredentials result{
      .bbs_id = added.getBbsId(),
      .psk = {},
      .committed_sequence = response.getCommittedSequence(),
   };
   std::copy(psk.begin(), psk.end(), result.psk.begin());
   return result;
}

UniverseInitialization initialize_universe(
   TlsConnection& connection,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id) {
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<admin::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   request.initInitializeUniverse();
   const auto words = capnp::messageToFlatArray(message);
   send_frame(connection, words.asBytes());

   const auto frame = receive_frame(connection);
   const auto word_count =
      (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
   auto response_words = kj::heapArray<capnp::word>(word_count);
   std::memset(response_words.begin(), 0, response_words.asBytes().size());
   std::memcpy(response_words.asBytes().begin(), frame.data(), frame.size());
   capnp::FlatArrayMessageReader reader(response_words);
   const auto response_envelope = reader.getRoot<admin::Envelope>();
   if(response_envelope.getProtocolVersion() != PROTOCOL_VERSION ||
      response_envelope.getRequestId() != request_id ||
      !response_envelope.isResponse()) {
      throw std::runtime_error("invalid administrator response envelope");
   }
   const auto response = response_envelope.getResponse();
   const auto returned_command_id = response.getCommandId();
   if(returned_command_id.size() != command_id.size() ||
      !std::equal(returned_command_id.begin(),
                  returned_command_id.end(),
                  command_id.begin())) {
      throw std::runtime_error("administrator response command ID mismatch");
   }
   if(response.isError()) {
      const auto error = response.getError();
      if(error.getCode() == admin::ErrorCode::INVALID_REQUEST) {
         throw AdministratorRequestRejected(error.getMessage().cStr());
      }
      throw std::runtime_error(error.getMessage().cStr());
   }
   if(!response.isUniverseInitialized()) {
      throw std::runtime_error("expected a universe-initialized response");
   }
   const auto initialized = response.getUniverseInitialized();
   const auto universe_id = initialized.getUniverseId();
   if(universe_id.size() != 16) {
      throw std::runtime_error("server returned an invalid universe ID");
   }
   UniverseInitialization result{
      .universe_id = {},
      .polity_count = initialized.getPolityCount(),
      .system_count = initialized.getSystemCount(),
      .world_count = initialized.getWorldCount(),
      .committed_sequence = response.getCommittedSequence(),
   };
   std::copy(universe_id.begin(), universe_id.end(), result.universe_id.begin());
   return result;
}

ServerStatus server_status(TlsConnection& connection, const uint64_t request_id) {
   const std::array<uint8_t, 16> command_id{};
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<admin::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   request.setStatus();
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());

   const auto frame = receive_frame(connection);
   const auto word_count =
      (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
   auto words = kj::heapArray<capnp::word>(word_count);
   std::memset(words.begin(), 0, words.asBytes().size());
   std::memcpy(words.asBytes().begin(), frame.data(), frame.size());
   capnp::FlatArrayMessageReader reader(words);
   const auto response_envelope = reader.getRoot<admin::Envelope>();
   if(response_envelope.getProtocolVersion() != PROTOCOL_VERSION ||
      response_envelope.getRequestId() != request_id ||
      !response_envelope.isResponse()) {
      throw std::runtime_error("invalid administrator status response");
   }
   const auto response = response_envelope.getResponse();
   const auto returned_command_id = response.getCommandId();
   if(returned_command_id.size() != command_id.size() ||
      !std::equal(returned_command_id.begin(), returned_command_id.end(),
                  command_id.begin())) {
      throw std::runtime_error("administrator status command ID mismatch");
   }
   if(response.isError()) {
      throw AdministratorRequestRejected(response.getError().getMessage().cStr());
   }
   if(!response.isStatus()) {
      throw std::runtime_error("expected a server-status response");
   }
   const auto status = response.getStatus();
   return ServerStatus{
      .committed_sequence = status.getCommittedSequence(),
      .game_second = status.getGameSecond(),
      .queued_inputs = status.getQueuedInputs(),
      .bbs_count = status.getBbsCount(),
      .player_count = status.getPlayerCount(),
      .system_count = status.getSystemCount(),
      .active_sessions = status.getActiveSessions(),
      .storage_format = status.getStorageFormat(),
   };
}

BackupComplete live_backup(
   TlsConnection& connection,
   const std::string& label,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id) {
   if(label.empty() || label.size() > 64 ||
      !std::all_of(label.begin(), label.end(), [](const unsigned char byte) {
         return std::isalnum(byte) != 0 || byte == '-' || byte == '_';
      })) {
      throw std::invalid_argument("invalid backup label");
   }
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<admin::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   request.initLiveBackup().setLabel(label);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());

   const auto frame = receive_frame(connection);
   const auto word_count =
      (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word);
   auto words = kj::heapArray<capnp::word>(word_count);
   std::memset(words.begin(), 0, words.asBytes().size());
   std::memcpy(words.asBytes().begin(), frame.data(), frame.size());
   capnp::FlatArrayMessageReader reader(words);
   const auto response_envelope = reader.getRoot<admin::Envelope>();
   if(response_envelope.getProtocolVersion() != PROTOCOL_VERSION ||
      response_envelope.getRequestId() != request_id ||
      !response_envelope.isResponse()) {
      throw std::runtime_error("invalid administrator backup response");
   }
   const auto response = response_envelope.getResponse();
   const auto returned_command_id = response.getCommandId();
   if(returned_command_id.size() != command_id.size() ||
      !std::equal(returned_command_id.begin(), returned_command_id.end(),
                  command_id.begin())) {
      throw std::runtime_error("administrator backup command ID mismatch");
   }
   if(response.isError()) {
      throw AdministratorRequestRejected(response.getError().getMessage().cStr());
   }
   if(!response.isBackupComplete()) {
      throw std::runtime_error("expected a backup-complete response");
   }
   const auto complete = response.getBackupComplete();
   return BackupComplete{
      .label = complete.getLabel().cStr(),
      .committed_sequence = complete.getCommittedSequence(),
      .game_second = complete.getGameSecond(),
   };
}

}  // namespace ct
