#pragma once

#include <array>
#include <cstdint>
#include <stdexcept>
#include <string>

namespace ct {

class TlsConnection;

class AdministratorRequestRejected : public std::runtime_error {
   public:
      using std::runtime_error::runtime_error;
};

struct BbsCredentials {
   uint32_t bbs_id;
   std::array<uint8_t, 32> psk;
   uint64_t committed_sequence;
};

struct UniverseInitialization {
   std::array<uint8_t, 16> universe_id;
   uint32_t polity_count;
   uint32_t system_count;
   uint32_t world_count;
   uint64_t committed_sequence;
};

struct ServerStatus {
   uint64_t committed_sequence;
   uint64_t game_second;
   uint64_t queued_inputs;
   uint32_t bbs_count;
   uint64_t player_count;
   uint64_t system_count;
   uint32_t active_sessions;
   uint64_t storage_format;
};

struct BackupComplete {
   std::string label;
   uint64_t committed_sequence;
   uint64_t game_second;
};

void exchange_admin_hello(TlsConnection& connection,
                          const std::string& language_tag);

BbsCredentials add_bbs(TlsConnection& connection,
                       const std::string& name,
                       const std::array<uint8_t, 16>& command_id,
                       uint64_t request_id);

UniverseInitialization initialize_universe(
   TlsConnection& connection,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

ServerStatus server_status(TlsConnection& connection, uint64_t request_id);

BackupComplete live_backup(TlsConnection& connection,
                           const std::string& label,
                           const std::array<uint8_t, 16>& command_id,
                           uint64_t request_id);

}  // namespace ct
