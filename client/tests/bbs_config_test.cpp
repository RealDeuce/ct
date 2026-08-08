#include "ct/bbs_config.hpp"

#include <chrono>
#include <filesystem>
#include <fstream>
#include <iterator>
#include <stdexcept>
#include <string>

#ifndef _WIN32
#include <sys/stat.h>
#endif

namespace {

void check(const bool condition) {
   if(!condition) {
      throw std::runtime_error("BBS configuration test failed");
   }
}

class ScratchDirectory final {
public:
   ScratchDirectory() {
      const auto suffix =
         std::chrono::steady_clock::now().time_since_epoch().count();
      path = std::filesystem::temp_directory_path() /
             ("cepheus-trader-bbs-config-" + std::to_string(suffix));
      if(!std::filesystem::create_directory(path)) {
         throw std::runtime_error("could not create test directory");
      }
   }

   ~ScratchDirectory() {
      std::error_code ignored;
      std::filesystem::remove_all(path, ignored);
   }

   ScratchDirectory(const ScratchDirectory&) = delete;
   ScratchDirectory& operator=(const ScratchDirectory&) = delete;

   std::filesystem::path path;
};

}  // namespace

int main() {
   ScratchDirectory scratch;
   const auto path = scratch.path / "cepheus-trader.config";
   const auto credential = scratch.path / "cepheus-trader.credential";
   const auto identities = scratch.path / "cepheus-trader.identities";

   const auto defaults = ct::default_bbs_config(path.string());
   check(defaults.server == "localhost");
   check(defaults.game_port == "7323");
   check(defaults.sysop_port == "7325");
   check(defaults.credential_path == credential.string());
   check(defaults.identity_registry_path == identities.string());
   check(defaults.identity_name_field == ct::IdentityNameField::RealName);
   check(defaults.terminal_profile == "auto");
   check(defaults.terminal_columns == 0);
   check(defaults.terminal_rows == 0);
   check(defaults.inactivity_timeout_seconds == 300);

   ct::create_default_bbs_config_file(path.string());
   const auto parsed = ct::read_bbs_config(path.string());
   check(parsed.server == defaults.server);
   check(parsed.game_port == defaults.game_port);
   check(parsed.sysop_port == defaults.sysop_port);
   check(parsed.credential_path == defaults.credential_path);
   check(parsed.identity_registry_path == defaults.identity_registry_path);
   check(parsed.inactivity_timeout_seconds == 300);

   std::ifstream input(path);
   const std::string original{
      std::istreambuf_iterator<char>(input),
      std::istreambuf_iterator<char>()};
   check(original.find("credential-file=cepheus-trader.credential\n") !=
         std::string::npos);
   check(original.find("identity-file=cepheus-trader.identities\n") !=
         std::string::npos);
   check(original.find("inactivity-timeout-seconds=300\n") !=
         std::string::npos);

   ct::set_bbs_inactivity_timeout(path.string(), 600);
   const auto adjusted = ct::read_bbs_config(path.string());
   check(adjusted.inactivity_timeout_seconds == 600);
   std::ifstream adjusted_input(path);
   const std::string adjusted_text{
      std::istreambuf_iterator<char>(adjusted_input),
      std::istreambuf_iterator<char>()};
   check(adjusted_text.find("inactivity-timeout-seconds=600\n") !=
         std::string::npos);

   const auto legacy_path = scratch.path / "legacy.conf";
   auto legacy_text = original;
   const std::string timeout_line = "inactivity-timeout-seconds=300\n";
   legacy_text.erase(legacy_text.find(timeout_line), timeout_line.size());
   {
      std::ofstream legacy_output(legacy_path);
      legacy_output << legacy_text;
   }
   check(ct::read_bbs_config(legacy_path.string()).inactivity_timeout_seconds ==
         300);
   ct::set_bbs_inactivity_timeout(legacy_path.string(), 0);
   check(ct::read_bbs_config(legacy_path.string()).inactivity_timeout_seconds ==
         0);

   bool refused_invalid_timeout = false;
   try {
      ct::set_bbs_inactivity_timeout(path.string(), 32768);
   } catch(const std::exception&) {
      refused_invalid_timeout = true;
   }
   check(refused_invalid_timeout);

   bool refused_overwrite = false;
   try {
      ct::create_default_bbs_config_file(path.string());
   } catch(const std::exception&) {
      refused_overwrite = true;
   }
   check(refused_overwrite);

   std::ifstream reread(path);
   const std::string after_refusal{
      std::istreambuf_iterator<char>(reread),
      std::istreambuf_iterator<char>()};
   check(after_refusal == adjusted_text);

   const auto custom_config = scratch.path / "installation" / "custom.conf";
   const auto custom_credential =
      scratch.path / "installation" / "secrets" / "bbs.bin";
   ct::create_bbs_config_file(custom_config.string(),
                              custom_credential.string(),
                              "game.example.net",
                              "17323",
                              "17325");
   const auto custom = ct::read_bbs_config(custom_config.string());
   check(custom.server == "game.example.net");
   check(custom.game_port == "17323");
   check(custom.sysop_port == "17325");
   check(custom.credential_path == custom_credential.string());
   std::ifstream custom_input(custom_config);
   const std::string custom_text{
      std::istreambuf_iterator<char>(custom_input),
      std::istreambuf_iterator<char>()};
   check(custom_text.find("credential-file=secrets/bbs.bin\n") !=
         std::string::npos);
#ifndef _WIN32
   check((std::filesystem::status(custom_config.parent_path()).permissions() &
          std::filesystem::perms::all) ==
         std::filesystem::perms::owner_all);
   check((std::filesystem::status(custom_credential.parent_path()).permissions() &
          std::filesystem::perms::all) ==
         std::filesystem::perms::owner_all);
#endif
}
