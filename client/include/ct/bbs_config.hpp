#pragma once

#include <string>

namespace ct {

inline constexpr unsigned int DEFAULT_INACTIVITY_TIMEOUT_SECONDS = 300;
inline constexpr unsigned int MAX_INACTIVITY_TIMEOUT_SECONDS = 32767;

enum class IdentityNameField {
   RealName,
   Handle,
};

struct BbsConfig {
   std::string server;
   std::string game_port;
   std::string sysop_port;
   std::string credential_path;
   std::string identity_registry_path;
   IdentityNameField identity_name_field;
   std::string terminal_profile;
   unsigned int terminal_columns;
   unsigned int terminal_rows;
   unsigned int inactivity_timeout_seconds;
};

BbsConfig read_bbs_config(const std::string& path);
BbsConfig default_bbs_config(const std::string& path);
void create_bbs_installation_directories(const std::string& config_path,
                                         const std::string& credential_path);
void create_bbs_config_file(const std::string& path,
                            const std::string& credential_path);
void create_bbs_config_file(const std::string& path,
                            const std::string& credential_path,
                            const std::string& server,
                            const std::string& game_port,
                            const std::string& sysop_port);
void create_default_bbs_config_file(const std::string& path);
void set_bbs_inactivity_timeout(const std::string& path,
                                unsigned int timeout_seconds);

}  // namespace ct
