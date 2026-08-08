#include "ct/bbs_config.hpp"
#include "ct/bbs_credential.hpp"
#include "ct/crypto.hpp"
#include "ct/player_identity_registry.hpp"
#include "ct/sysop_protocol.hpp"
#include "ct/tls_connection.hpp"

#include <algorithm>
#include <array>
#include <charconv>
#include <cerrno>
#include <cstring>
#include <cstdint>
#include <exception>
#include <filesystem>
#include <iostream>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

#ifdef _WIN32
#include <windows.h>
#else
#include <termios.h>
#include <unistd.h>
#endif

namespace {

uint32_t parse_bbs_id(const std::string& text) {
   uint32_t value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size() || value == 0) {
      throw std::invalid_argument("BBS ID must be a nonzero UInt32");
   }
   return value;
}

uint32_t parse_player_id(const std::string& text) {
   uint32_t value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size() || value == 0) {
      throw std::invalid_argument("PLAYER_ID must be a nonzero UInt32");
   }
   return value;
}

std::optional<uint32_t> parse_record_index(const std::string& text) {
   if(text == "none") {
      return std::nullopt;
   }
   uint32_t value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size()) {
      throw std::invalid_argument("RECORD_INDEX must be a UInt32 or 'none'");
   }
   return value;
}

uint64_t parse_credits(const std::string& text) {
   uint64_t value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size() || value == 0) {
      throw std::invalid_argument("CREDITS must be a nonzero UInt64");
   }
   return value;
}

uint8_t parse_grade(const std::string& text) {
   unsigned int value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size() || value > 255) {
      throw std::invalid_argument("NAVAL_GRADE_INDEX must be in 0..255");
   }
   return static_cast<uint8_t>(value);
}

uint64_t parse_revision(const std::string& text) {
   uint64_t value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size()) {
      throw std::invalid_argument("--expected-revision must be a UInt64");
   }
   return value;
}

std::string parse_port(const std::string& text, const char* option) {
   uint32_t value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size() ||
      value == 0 || value > 65535) {
      throw std::invalid_argument(
         std::string(option) + " must be an integer in 1..65535");
   }
   return text;
}

std::string parse_server(const std::string& text) {
   if(text.empty() || text.find_first_of("\r\n") != std::string::npos) {
      throw std::invalid_argument(
         "--server must be a nonempty hostname or address");
   }
   return text;
}

std::array<uint8_t, 16> parse_command_id(const std::string& text) {
   if(text.size() != 32 ||
      !std::all_of(text.begin(), text.end(), [](const unsigned char value) {
         return (value >= '0' && value <= '9') ||
                (value >= 'a' && value <= 'f') ||
                (value >= 'A' && value <= 'F');
      })) {
      throw std::invalid_argument(
         "--command-id must be exactly 32 hexadecimal characters");
   }
   const auto decoded = ct::hex_decode(text);
   std::array<uint8_t, 16> command_id{};
   std::copy(decoded.begin(), decoded.end(), command_id.begin());
   return command_id;
}

std::array<uint8_t, 16> generate_command_id() {
   return ct::random_command_id();
}

uint8_t parse_orientation(const std::string& text, const char* name) {
   unsigned int value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size() || value > 100) {
      throw std::invalid_argument(std::string(name) + " must be in 0..100");
   }
   return static_cast<uint8_t>(value);
}

class StdinEchoGuard final {
   public:
      explicit StdinEchoGuard(const bool disable) {
         if(!disable) {
            return;
         }
#ifdef _WIN32
         m_handle = GetStdHandle(STD_INPUT_HANDLE);
         if(m_handle == INVALID_HANDLE_VALUE ||
            GetConsoleMode(m_handle, &m_original_mode) == 0) {
            return;
         }
         if(SetConsoleMode(m_handle, m_original_mode & ~ENABLE_ECHO_INPUT) == 0) {
            throw std::runtime_error(
               "cannot disable stdin echo, Windows error " +
               std::to_string(GetLastError()));
         }
#else
         if(isatty(STDIN_FILENO) == 0) {
            return;
         }
         if(tcgetattr(STDIN_FILENO, &m_original_mode) != 0) {
            throw std::runtime_error(
               "cannot read terminal mode: " + std::string(std::strerror(errno)));
         }
         auto hidden = m_original_mode;
         hidden.c_lflag &= static_cast<tcflag_t>(~ECHO);
         if(tcsetattr(STDIN_FILENO, TCSAFLUSH, &hidden) != 0) {
            throw std::runtime_error(
               "cannot disable stdin echo: " + std::string(std::strerror(errno)));
         }
#endif
         m_active = true;
      }

      ~StdinEchoGuard() {
         if(!m_active) {
            return;
         }
#ifdef _WIN32
         SetConsoleMode(m_handle, m_original_mode);
#else
         tcsetattr(STDIN_FILENO, TCSAFLUSH, &m_original_mode);
#endif
      }

      StdinEchoGuard(const StdinEchoGuard&) = delete;
      StdinEchoGuard& operator=(const StdinEchoGuard&) = delete;

   private:
      bool m_active = false;
#ifdef _WIN32
      HANDLE m_handle = INVALID_HANDLE_VALUE;
      DWORD m_original_mode = 0;
#else
      struct termios m_original_mode {};
#endif
};

bool stdin_is_terminal() {
#ifdef _WIN32
   DWORD mode = 0;
   const auto handle = GetStdHandle(STD_INPUT_HANDLE);
   return handle != INVALID_HANDLE_VALUE && GetConsoleMode(handle, &mode) != 0;
#else
   return isatty(STDIN_FILENO) != 0;
#endif
}

std::string read_required_line(const char* description, const bool secret = false) {
   const bool terminal = stdin_is_terminal();
   if(terminal) {
      std::cerr << description << ": " << std::flush;
   }
   StdinEchoGuard echo_guard(secret);
   std::string line;
   if(!std::getline(std::cin, line)) {
      throw std::runtime_error(std::string("missing ") + description + " on stdin");
   }
   if(terminal && secret) {
      std::cerr << '\n';
   }
   if(line.empty()) {
      throw std::runtime_error(std::string(description) + " must not be empty");
   }
   return line;
}

void print_configuration(const ct::SysopConfiguration& configuration) {
   std::cout << "bbs-id=" << configuration.bbs_id << '\n'
             << "revision=" << configuration.revision << '\n'
             << "committed=" << configuration.committed_sequence << '\n'
             << "configured=" << (configuration.configured ? "yes" : "no") << '\n'
             << "bbs-name=" << configuration.settings.bbs_name << '\n'
             << "polity-name=" << configuration.settings.polity_name << '\n'
             << "trade-combat=" << static_cast<unsigned int>(
                   configuration.settings.trade_combat) << '\n'
             << "chaos-order=" << static_cast<unsigned int>(
                   configuration.settings.chaos_order) << '\n';
}

const char* access_state_name(const ct::PlayerAccessState state) {
   switch(state) {
      case ct::PlayerAccessState::Active:
         return "active";
      case ct::PlayerAccessState::Suspended:
         return "suspended";
      case ct::PlayerAccessState::Removed:
         return "removed";
   }
   return "unknown";
}

void print_player_access(const ct::PlayerAccess& access) {
   std::cout << "player-id=" << access.player_id << '\n'
             << "revision=" << access.revision << '\n'
             << "committed=" << access.committed_sequence << '\n'
             << "state=" << access_state_name(access.state) << '\n'
             << "reason=" << access.reason << '\n';
}

std::filesystem::path comparable_path(const std::string& path) {
   return std::filesystem::absolute(std::filesystem::path(path))
      .lexically_normal();
}

int init_credential(const std::string& config_path,
                    const std::optional<std::string>& requested_path,
                    const std::optional<std::string>& requested_server,
                    const std::optional<std::string>& requested_game_port,
                    const std::optional<std::string>& requested_sysop_port) {
   std::error_code existence_error;
   const bool config_exists =
      std::filesystem::exists(config_path, existence_error);
   if(existence_error) {
      throw std::runtime_error(
         "cannot inspect BBS configuration file: " + config_path + ": " +
         existence_error.message());
   }

   ct::BbsConfig config = config_exists
                            ? ct::read_bbs_config(config_path)
                            : ct::default_bbs_config(config_path);
   if(requested_path.has_value()) {
      if(config_exists &&
         comparable_path(config.credential_path) !=
            comparable_path(*requested_path)) {
         throw std::invalid_argument(
            "credential path does not match 'credential-file' in " +
            config_path + ": " + config.credential_path);
      }
      config.credential_path = *requested_path;
   }
   const auto apply_initial_value =
      [config_exists, &config_path](const std::optional<std::string>& requested,
                                    std::string& value,
                                    const char* option) {
         if(!requested.has_value()) {
            return;
         }
         if(config_exists && value != *requested) {
            throw std::invalid_argument(
               std::string(option) + " does not match the value in " +
               config_path + ": " + value);
         }
         value = *requested;
      };
   apply_initial_value(requested_server, config.server, "--server");
   apply_initial_value(requested_game_port, config.game_port, "--game-port");
   apply_initial_value(requested_sysop_port, config.sysop_port, "--sysop-port");

   const auto bbs_id = parse_bbs_id(read_required_line("BBS ID"));
   auto psk_text = read_required_line("BBS PSK", true);
   std::vector<uint8_t> psk;
   try {
      psk = ct::hex_decode(psk_text);
   } catch(...) {
      ct::scrub_memory(psk_text);
      throw;
   }
   ct::scrub_memory(psk_text);
   if(psk.size() != ct::BBS_PSK_BYTES) {
      ct::scrub_memory(psk);
      throw std::invalid_argument("BBS PSK must encode exactly 32 bytes");
   }
   try {
      ct::create_bbs_installation_directories(
         config_path, config.credential_path);
      if(!config_exists) {
         ct::create_bbs_config_file(
            config_path,
            config.credential_path,
            config.server,
            config.game_port,
            config.sysop_port);
      }
      ct::create_bbs_credential_file(config.credential_path, bbs_id, psk);
      ct::create_player_identity_registry(config.identity_registry_path, bbs_id);
   } catch(...) {
      ct::scrub_memory(psk);
      throw;
   }
   ct::scrub_memory(psk);
   if(!config_exists) {
      std::cout << "installation configuration created: " << config_path << '\n';
   }
   std::cout << "credential created for BBS id=" << bbs_id << ": "
             << config.credential_path << '\n'
             << "identity registry created: "
             << config.identity_registry_path << '\n'
             << "next: cepheus-trader-sysop --config " << config_path
             << " set-config BBS_NAME POLITY_NAME TRADE_COMBAT CHAOS_ORDER\n";
   return 0;
}

struct CommandLine {
   std::string config_path = "cepheus-trader.conf";
   std::optional<std::string> server;
   std::optional<std::string> game_port;
   std::optional<std::string> sysop_port;
   std::optional<uint64_t> expected_revision;
   std::optional<std::array<uint8_t, 16>> command_id;
   std::vector<std::string> positional;
};

CommandLine parse_command_line(const int argc, char** argv) {
   CommandLine result;
   for(int index = 1; index < argc; ++index) {
      const std::string_view argument = argv[index];
      if(argument == "--config") {
         if(++index >= argc) {
            throw std::invalid_argument("--config needs a path");
         }
         result.config_path = argv[index];
      } else if(argument == "--server") {
         if(++index >= argc) {
            throw std::invalid_argument("--server needs a hostname or address");
         }
         result.server = parse_server(argv[index]);
      } else if(argument == "--game-port") {
         if(++index >= argc) {
            throw std::invalid_argument("--game-port needs a value");
         }
         result.game_port = parse_port(argv[index], "--game-port");
      } else if(argument == "--sysop-port") {
         if(++index >= argc) {
            throw std::invalid_argument("--sysop-port needs a value");
         }
         result.sysop_port = parse_port(argv[index], "--sysop-port");
      } else if(argument == "--expected-revision") {
         if(++index >= argc) {
            throw std::invalid_argument("--expected-revision needs a value");
         }
         result.expected_revision = parse_revision(argv[index]);
      } else if(argument == "--command-id") {
         if(++index >= argc) {
            throw std::invalid_argument("--command-id needs a value");
         }
         result.command_id = parse_command_id(argv[index]);
      } else if(argument.starts_with("--")) {
         throw std::invalid_argument("unknown option: " + std::string(argument));
      } else {
         result.positional.emplace_back(argument);
      }
   }
   return result;
}

void print_usage() {
   std::cerr
      << "usage: cepheus-trader-sysop [--config FILE] [--server HOST]"
         " [--game-port PORT] [--sysop-port PORT]\n"
         "       init-credential [CREDENTIAL_FILE]\n"
         "       cepheus-trader-sysop [--config FILE] get-config\n"
         "       cepheus-trader-sysop [--config FILE] set-config"
         " BBS_NAME POLITY_NAME TRADE_COMBAT CHAOS_ORDER\n"
         "       cepheus-trader-sysop [--config FILE] identity-list\n"
         "       cepheus-trader-sysop [--config FILE] identity-rename"
         " PLAYER_ID NEW_NAME\n"
         "       cepheus-trader-sysop [--config FILE] identity-reindex"
         " PLAYER_ID RECORD_INDEX|none\n"
         "       cepheus-trader-sysop [--config FILE] identity-retire"
         " PLAYER_ID RETIRE\n"
         "       cepheus-trader-sysop [--config FILE] player-access PLAYER_ID\n"
         "       cepheus-trader-sysop [--config FILE] suspend-player"
         " PLAYER_ID REASON\n"
         "       cepheus-trader-sysop [--config FILE] resume-player"
         " PLAYER_ID REASON\n"
         "       cepheus-trader-sysop [--config FILE] remove-player"
         " PLAYER_ID REMOVE REASON\n"
         "       cepheus-trader-sysop [--config FILE] tax-player"
         " PLAYER_ID CREDITS\n"
         "       cepheus-trader-sysop [--config FILE] demote-player"
         " PLAYER_ID NAVAL_GRADE_INDEX\n"
         "       retry options: --expected-revision REVISION"
         " --command-id 32_HEX_CHARACTERS\n"
         "       --version prints the product version\n"
         "init stdin: BBS_ID followed by the 64-hex-digit PSK,"
         " one per line\n";
}

}  // namespace

int main(int argc, char** argv) {
   try {
      if(argc == 2 && std::string_view(argv[1]) == "--version") {
         std::cout << "cepheus-trader-sysop " << CT_PRODUCT_VERSION << '\n';
         return 0;
      }
      const auto command_line = parse_command_line(argc, argv);
      const bool init =
         (command_line.positional.size() == 1 ||
          command_line.positional.size() == 2) &&
         command_line.positional[0] == "init-credential" &&
         !command_line.expected_revision.has_value() &&
         !command_line.command_id.has_value();
      if(init) {
         return init_credential(
            command_line.config_path,
            command_line.positional.size() == 2
               ? std::optional<std::string>(command_line.positional[1])
               : std::nullopt,
            command_line.server,
            command_line.game_port,
            command_line.sysop_port);
      }
      if(command_line.server || command_line.game_port ||
         command_line.sysop_port) {
         throw std::invalid_argument(
            "--server, --game-port, and --sysop-port are valid only with "
            "init-credential");
      }
      const bool identity_command =
         !command_line.positional.empty() &&
         command_line.positional[0].starts_with("identity-") &&
         !command_line.expected_revision.has_value() &&
         !command_line.command_id.has_value();
      if(identity_command) {
         const auto config = ct::read_bbs_config(command_line.config_path);
         const auto credential =
            ct::read_bbs_credential_file(config.credential_path);
         if(command_line.positional[0] == "identity-list" &&
            command_line.positional.size() == 1) {
            for(const auto& entry : ct::list_player_identities(
                   config.identity_registry_path, credential.bbs_id)) {
               std::cout << entry.player_id << '\t'
                         << (entry.retired ? "retired" : "active") << '\t';
               if(entry.record_index) {
                  std::cout << *entry.record_index;
               } else {
                  std::cout << "none";
               }
               std::cout << '\t' << entry.name << '\n';
            }
            return 0;
         }
         if(command_line.positional[0] == "identity-rename" &&
            command_line.positional.size() == 3) {
            ct::rename_player_identity(
               config.identity_registry_path,
               credential.bbs_id,
               parse_player_id(command_line.positional[1]),
               command_line.positional[2]);
            return 0;
         }
         if(command_line.positional[0] == "identity-reindex" &&
            command_line.positional.size() == 3) {
            ct::reindex_player_identity(
               config.identity_registry_path,
               credential.bbs_id,
               parse_player_id(command_line.positional[1]),
               parse_record_index(command_line.positional[2]));
            return 0;
         }
         if(command_line.positional[0] == "identity-retire" &&
            command_line.positional.size() == 3 &&
            command_line.positional[2] == "RETIRE") {
            ct::retire_player_identity(
               config.identity_registry_path,
               credential.bbs_id,
               parse_player_id(command_line.positional[1]));
            return 0;
         }
         print_usage();
         return 2;
      }
      const bool get_config =
         command_line.positional.size() == 1 &&
         command_line.positional[0] == "get-config" &&
         !command_line.expected_revision.has_value() &&
         !command_line.command_id.has_value();
      const bool set_config =
         command_line.positional.size() == 5 &&
         command_line.positional[0] == "set-config";
      const bool get_access =
         command_line.positional.size() == 2 &&
         command_line.positional[0] == "player-access";
      const bool suspend_player =
         command_line.positional.size() == 3 &&
         command_line.positional[0] == "suspend-player";
      const bool resume_player =
         command_line.positional.size() == 3 &&
         command_line.positional[0] == "resume-player";
      const bool remove_player =
         command_line.positional.size() == 4 &&
         command_line.positional[0] == "remove-player" &&
         command_line.positional[2] == "REMOVE";
      const bool set_access = suspend_player || resume_player || remove_player;
      const bool tax_player =
         command_line.positional.size() == 3 &&
         command_line.positional[0] == "tax-player";
      const bool demote_player =
         command_line.positional.size() == 3 &&
         command_line.positional[0] == "demote-player";
      const bool directive_command = tax_player || demote_player;
      const bool has_complete_retry =
         command_line.expected_revision.has_value() ==
         command_line.command_id.has_value();
      if(!directive_command && !has_complete_retry) {
         throw std::invalid_argument(
            "--expected-revision and --command-id must be supplied together");
      }
      if(directive_command && command_line.expected_revision) {
         throw std::invalid_argument(
            "tax-player and demote-player do not use --expected-revision");
      }
      if(!get_config && !set_config && !get_access && !set_access &&
         !tax_player && !demote_player) {
         print_usage();
         return 2;
      }
      std::optional<ct::SysopSettings> settings;
      if(set_config) {
         settings = ct::SysopSettings{
            .bbs_name = command_line.positional[1],
            .polity_name = command_line.positional[2],
            .trade_combat =
               parse_orientation(command_line.positional[3], "TRADE_COMBAT"),
            .chaos_order =
               parse_orientation(command_line.positional[4], "CHAOS_ORDER"),
         };
      }
      const bool generated_retry = !command_line.command_id.has_value();
      auto config = ct::read_bbs_config(command_line.config_path);
      auto credential = ct::read_bbs_credential_file(config.credential_path);
      ct::TlsConnection connection(
         config.server,
         config.sysop_port,
         std::to_string(credential.bbs_id),
         std::move(credential.psk));
      ct::exchange_sysop_hello(connection, "en");
      if(get_access) {
         if(command_line.expected_revision || command_line.command_id) {
            throw std::invalid_argument(
               "player-access does not accept retry options");
         }
         print_player_access(ct::get_player_access(
            connection, parse_player_id(command_line.positional[1]), 1));
         return 0;
      }
      if(set_access) {
         const auto player_id = parse_player_id(command_line.positional[1]);
         const auto current = ct::get_player_access(connection, player_id, 1);
         const auto expected_revision = command_line.expected_revision.value_or(
            current.revision);
         const auto command_id = command_line.command_id.value_or(
            generate_command_id());
         const auto state = suspend_player
                               ? ct::PlayerAccessState::Suspended
                               : resume_player
                                    ? ct::PlayerAccessState::Active
                                    : ct::PlayerAccessState::Removed;
         const auto& reason = command_line.positional[remove_player ? 3 : 2];
         print_player_access(ct::set_player_access(
            connection,
            player_id,
            expected_revision,
            state,
            reason,
            command_id,
            2));
         return 0;
      }
      if(tax_player || demote_player) {
         const auto player_id = parse_player_id(command_line.positional[1]);
         const auto command_id = command_line.command_id.value_or(
            generate_command_id());
         ct::DirectiveIssued issued;
         if(tax_player) {
            issued = ct::tax_player(
               connection,
               player_id,
               parse_credits(command_line.positional[2]),
               command_id,
               1);
         } else {
            issued = ct::demote_player(
               connection,
               player_id,
               parse_grade(command_line.positional[2]),
               command_id,
               1);
         }
         std::cout << "player-id=" << issued.player_id << '\n'
                   << "message-id=" << issued.message_id << '\n'
                   << "issued-second=" << issued.issued_second << '\n'
                   << (issued.is_tax ? "tax-credits=" : "naval-grade-index=")
                   << issued.value << '\n'
                   << "committed=" << issued.committed_sequence << '\n';
         return 0;
      }
      if(get_config) {
         print_configuration(ct::get_bbs_configuration(connection, 1));
         return 0;
      }
      const auto expected_revision = command_line.expected_revision.has_value()
                                        ? *command_line.expected_revision
                                        : ct::get_bbs_configuration(connection, 1).revision;
      const auto command_id = command_line.command_id.has_value()
                                 ? *command_line.command_id
                                 : generate_command_id();
      try {
         print_configuration(ct::set_bbs_configuration(
            connection,
            expected_revision,
            *settings,
            command_id,
            generated_retry ? 2 : 1));
         return 0;
      } catch(const std::exception& error) {
         std::cerr << error.what() << '\n';
         if(generated_retry) {
            std::cerr
               << "If the update may have reached the server, retry with "
                  "--expected-revision "
               << expected_revision << " --command-id "
               << ct::hex_encode(command_id) << '\n';
         }
         return 1;
      }
   } catch(const std::exception& error) {
      std::cerr << error.what() << '\n';
      return 1;
   }
}
