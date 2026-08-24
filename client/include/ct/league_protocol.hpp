#pragma once

#include <array>
#include <cstdint>
#include <stdexcept>
#include <string>
#include <vector>

namespace ct {

class TlsConnection;

class LeagueRequestRejected : public std::runtime_error {
   public:
      using std::runtime_error::runtime_error;
};

struct LeagueMemberStatus {
   uint32_t bbs_id;
   std::string bbs_name;
   bool enabled;
   std::string reason;
   uint64_t revision;
};

struct LeagueCoordinatorStatus {
   uint32_t league_id;
   std::string name;
   uint64_t revision;
   uint64_t committed_sequence;
   std::vector<LeagueMemberStatus> members;
   bool stale;
};

struct LeagueBbsCredentials {
   uint32_t bbs_id;
   std::array<uint8_t, 32> psk;
   uint64_t committed_sequence;
};

LeagueCoordinatorStatus league_status(TlsConnection& connection,
                                      uint64_t request_id);
LeagueCoordinatorStatus set_league_name(
   TlsConnection& connection,
   const std::string& name,
   uint64_t expected_revision,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);
LeagueBbsCredentials add_league_bbs(
   TlsConnection& connection,
   const std::string& name,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);
LeagueMemberStatus set_league_bbs_access(
   TlsConnection& connection,
   uint32_t bbs_id,
   uint64_t expected_revision,
   bool enabled,
   const std::string& reason,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id,
   bool& stale);

}  // namespace ct
