#include "ct/league_protocol.hpp"

#include "ct/tls_connection.hpp"
#include "ct_league.capnp.h"

#include <capnp/message.h>
#include <capnp/serialize.h>
#include <kj/array.h>

#include <algorithm>
#include <cstring>
#include <limits>
#include <span>

namespace ct {
namespace {

constexpr uint16_t PROTOCOL_VERSION = 1;
constexpr size_t MAX_FRAME_BYTES = 1024 * 1024;

void send_frame(TlsConnection& connection, const kj::ArrayPtr<const kj::byte> message) {
   if(message.size() == 0 || message.size() > MAX_FRAME_BYTES ||
      message.size() > std::numeric_limits<uint32_t>::max()) {
      throw std::runtime_error("invalid outgoing league frame size");
   }
   const auto size = static_cast<uint32_t>(message.size());
   connection.send(std::array<uint8_t, 4>{
      static_cast<uint8_t>(size >> 24), static_cast<uint8_t>(size >> 16),
      static_cast<uint8_t>(size >> 8), static_cast<uint8_t>(size)});
   connection.send(std::span(
      reinterpret_cast<const uint8_t*>(message.begin()), message.size()));
}

std::vector<uint8_t> receive_frame(TlsConnection& connection) {
   const auto header = connection.receive_exact(4);
   const auto size = (static_cast<uint32_t>(header[0]) << 24) |
                     (static_cast<uint32_t>(header[1]) << 16) |
                     (static_cast<uint32_t>(header[2]) << 8) |
                     static_cast<uint32_t>(header[3]);
   if(size == 0 || size > MAX_FRAME_BYTES) {
      throw std::runtime_error("invalid incoming league frame size");
   }
   return connection.receive_exact(size);
}

struct ResponseReader {
   kj::Array<capnp::word> words;
   capnp::FlatArrayMessageReader reader;
   league::Envelope::Reader envelope;
   league::Response::Reader response;

   ResponseReader(std::vector<uint8_t> frame,
                  const uint64_t request_id,
                  const std::array<uint8_t, 16>& command_id)
      : words(make_words(frame)),
        reader(words),
        envelope(reader.getRoot<league::Envelope>()),
        response(validate(request_id, command_id)) {}

   static kj::Array<capnp::word> make_words(const std::vector<uint8_t>& frame) {
      auto result = kj::heapArray<capnp::word>(
         (frame.size() + sizeof(capnp::word) - 1) / sizeof(capnp::word));
      std::memset(result.begin(), 0, result.asBytes().size());
      std::memcpy(result.asBytes().begin(), frame.data(), frame.size());
      return result;
   }

   league::Response::Reader validate(
      const uint64_t request_id,
      const std::array<uint8_t, 16>& command_id) {
      if(envelope.getProtocolVersion() != PROTOCOL_VERSION ||
         envelope.getRequestId() != request_id || !envelope.isResponse()) {
         if(envelope.isClose()) {
            throw std::runtime_error(envelope.getClose().getMessage().cStr());
         }
         throw std::runtime_error("invalid league response envelope");
      }
      const auto value = envelope.getResponse();
      const auto returned = value.getCommandId();
      if(returned.size() != command_id.size() ||
         !std::equal(returned.begin(), returned.end(), command_id.begin())) {
         throw std::runtime_error("league response command ID mismatch");
      }
      if(value.isError()) {
         throw LeagueRequestRejected(value.getError().getMessage().cStr());
      }
      return value;
   }
};

LeagueMemberStatus decode_member(league::LeagueMember::Reader member) {
   return LeagueMemberStatus{
      .bbs_id = member.getBbsId(),
      .bbs_name = member.getBbsName().cStr(),
      .enabled = member.getEnabled(),
      .reason = member.getReason().cStr(),
      .revision = member.getRevision(),
   };
}

LeagueCoordinatorStatus decode_status(league::LeagueStatus::Reader status,
                                      const uint64_t sequence,
                                      const bool stale) {
   LeagueCoordinatorStatus result{
      .league_id = status.getLeagueId(),
      .name = status.getName().cStr(),
      .revision = status.getRevision(),
      .committed_sequence = sequence,
      .members = {},
      .stale = stale,
   };
   for(const auto member : status.getMembers()) {
      result.members.push_back(decode_member(member));
   }
   return result;
}

template <typename Fill>
ResponseReader exchange(TlsConnection& connection,
                        const uint64_t request_id,
                        const std::array<uint8_t, 16>& command_id,
                        Fill fill) {
   capnp::MallocMessageBuilder message;
   auto envelope = message.initRoot<league::Envelope>();
   envelope.setProtocolVersion(PROTOCOL_VERSION);
   envelope.setRequestId(request_id);
   auto request = envelope.initRequest();
   request.setCommandId(kj::arrayPtr(command_id.data(), command_id.size()));
   fill(request);
   send_frame(connection, capnp::messageToFlatArray(message).asBytes());
   return ResponseReader(receive_frame(connection), request_id, command_id);
}

}  // namespace

LeagueCoordinatorStatus league_status(TlsConnection& connection,
                                      const uint64_t request_id) {
   const std::array<uint8_t, 16> command_id{};
   auto reply = exchange(connection, request_id, command_id,
                         [](auto request) { request.setStatus(); });
   if(!reply.response.isStatus()) {
      throw std::runtime_error("expected league status response");
   }
   return decode_status(reply.response.getStatus(),
                        reply.response.getCommittedSequence(), false);
}

LeagueCoordinatorStatus set_league_name(
   TlsConnection& connection, const std::string& name,
   const uint64_t expected_revision,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id) {
   auto reply = exchange(connection, request_id, command_id, [&](auto request) {
      auto set = request.initSetName();
      set.setExpectedRevision(expected_revision);
      set.setName(name);
   });
   if(reply.response.isNameSet()) {
      return decode_status(reply.response.getNameSet(),
                           reply.response.getCommittedSequence(), false);
   }
   if(reply.response.isStale()) {
      return decode_status(reply.response.getStale(),
                           reply.response.getCommittedSequence(), true);
   }
   throw std::runtime_error("expected league-name response");
}

LeagueBbsCredentials add_league_bbs(
   TlsConnection& connection, const std::string& name,
   const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id) {
   auto reply = exchange(connection, request_id, command_id, [&](auto request) {
      request.initAddBbs().setName(name);
   });
   if(!reply.response.isBbsAdded()) {
      throw std::runtime_error("expected league BBS-added response");
   }
   const auto added = reply.response.getBbsAdded();
   const auto psk = added.getPsk();
   if(psk.size() != 32) {
      throw std::runtime_error("server returned an invalid BBS PSK");
   }
   LeagueBbsCredentials result{
      .bbs_id = added.getBbsId(),
      .psk = {},
      .committed_sequence = reply.response.getCommittedSequence(),
   };
   std::copy(psk.begin(), psk.end(), result.psk.begin());
   return result;
}

LeagueMemberStatus set_league_bbs_access(
   TlsConnection& connection, const uint32_t bbs_id,
   const uint64_t expected_revision, const bool enabled,
   const std::string& reason, const std::array<uint8_t, 16>& command_id,
   const uint64_t request_id, bool& stale) {
   auto reply = exchange(connection, request_id, command_id, [&](auto request) {
      auto set = enabled ? request.initEnableBbs() : request.initDisableBbs();
      set.setBbsId(bbs_id);
      set.setExpectedRevision(expected_revision);
      set.setReason(reason);
   });
   stale = reply.response.isStaleMember();
   if(stale) {
      return decode_member(reply.response.getStaleMember());
   }
   if(!reply.response.isMemberUpdated()) {
      throw std::runtime_error("expected league member response");
   }
   return decode_member(reply.response.getMemberUpdated());
}

}  // namespace ct
