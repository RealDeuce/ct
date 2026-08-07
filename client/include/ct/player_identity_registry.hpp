#pragma once

#include <cstdint>
#include <optional>
#include <string>
#include <vector>

namespace ct {

struct LocalPlayerIdentity {
   uint32_t player_id;
   std::string name;
   std::optional<uint32_t> record_index;
   bool retired;
};

void create_player_identity_registry(const std::string& path, uint32_t bbs_id);

uint32_t resolve_player_identity(const std::string& path,
                                 uint32_t bbs_id,
                                 const std::string& name,
                                 std::optional<uint32_t> record_index);

std::vector<LocalPlayerIdentity> list_player_identities(
   const std::string& path,
   uint32_t bbs_id);

void rename_player_identity(const std::string& path,
                            uint32_t bbs_id,
                            uint32_t player_id,
                            const std::string& new_name);

void reindex_player_identity(const std::string& path,
                             uint32_t bbs_id,
                             uint32_t player_id,
                             std::optional<uint32_t> new_record_index);

void retire_player_identity(const std::string& path,
                            uint32_t bbs_id,
                            uint32_t player_id);

}  // namespace ct
