#pragma once

#include <array>
#include <cstdint>
#include <stdexcept>
#include <string>

namespace ct {

class TlsConnection;

struct SysopSettings {
   std::string bbs_name;
   std::string polity_name;
   uint8_t trade_combat;
   uint8_t chaos_order;

   bool operator==(const SysopSettings&) const = default;
};

struct SysopConfiguration {
   uint32_t bbs_id;
   uint64_t revision;
   bool configured;
   SysopSettings settings;
   uint64_t committed_sequence;

   bool operator==(const SysopConfiguration&) const = default;
};

enum class PlayerAccessState {
   Active,
   Suspended,
   Removed,
};

struct PlayerAccess {
   uint32_t player_id;
   uint64_t revision;
   PlayerAccessState state;
   std::string reason;
   uint64_t committed_sequence;
};

struct DirectiveIssued {
   uint32_t player_id;
   uint64_t message_id;
   uint64_t issued_second;
   bool is_tax;
   uint64_t value;
   uint64_t committed_sequence;
};

class StaleConfiguration final : public std::runtime_error {
   public:
      explicit StaleConfiguration(uint64_t current_revision);
      uint64_t current_revision() const noexcept;

   private:
      uint64_t m_current_revision;
};

SysopConfiguration get_bbs_configuration(TlsConnection& connection,
                                         uint64_t request_id);

SysopConfiguration set_bbs_configuration(
   TlsConnection& connection,
   uint64_t expected_revision,
   const SysopSettings& settings,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

PlayerAccess get_player_access(TlsConnection& connection,
                               uint32_t player_id,
                               uint64_t request_id);

PlayerAccess set_player_access(
   TlsConnection& connection,
   uint32_t player_id,
   uint64_t expected_revision,
   PlayerAccessState state,
   const std::string& reason,
   const std::array<uint8_t, 16>& command_id,
   uint64_t request_id);

DirectiveIssued tax_player(TlsConnection& connection,
                           uint32_t player_id,
                           uint64_t credits,
                           const std::array<uint8_t, 16>& command_id,
                           uint64_t request_id);

DirectiveIssued demote_player(TlsConnection& connection,
                              uint32_t player_id,
                              uint8_t naval_grade_index,
                              const std::array<uint8_t, 16>& command_id,
                              uint64_t request_id);

}  // namespace ct
