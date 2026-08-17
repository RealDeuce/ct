#pragma once

#include <cstdint>
#include <optional>
#include <string>
#include <vector>

namespace ct {

enum class HelpLevel : uint8_t {
   Beginner,
   Expert,
};

enum class FirstWatchDisposition : uint8_t {
   NotOffered,
   Active,
   Hidden,
   LocallyComplete,
};

constexpr uint16_t FIRST_WATCH_PRESENTATION_VERSION = 1;

struct FirstWatchPreferenceState {
   FirstWatchDisposition disposition = FirstWatchDisposition::NotOffered;
   uint16_t presentation_version = FIRST_WATCH_PRESENTATION_VERSION;
   uint32_t seen = 0;
};

struct LocalPlayerIdentity {
   uint32_t player_id;
   std::string name;
   std::optional<uint32_t> record_index;
   bool retired;
   HelpLevel help_level;
   bool orientation_shown;
   bool page_pauses;
   FirstWatchPreferenceState first_watch;
   bool first_watch_preferences_recovered;
};

void create_player_identity_registry(const std::string& path, uint32_t bbs_id);

LocalPlayerIdentity resolve_player_identity(
   const std::string& path,
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

void set_player_help_level(const std::string& path,
                           uint32_t bbs_id,
                           uint32_t player_id,
                           HelpLevel help_level);

void set_player_page_pauses(const std::string& path,
                            uint32_t bbs_id,
                            uint32_t player_id,
                            bool enabled);

void mark_player_orientation_shown(const std::string& path,
                                   uint32_t bbs_id,
                                   uint32_t player_id);

void set_player_first_watch_state(const std::string& path,
                                  uint32_t bbs_id,
                                  uint32_t player_id,
                                  FirstWatchPreferenceState state);

}  // namespace ct
