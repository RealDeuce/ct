#include "ct/admin_protocol.hpp"
#include "ct/crypto.hpp"
#include "ct/tls_connection.hpp"

#include <algorithm>
#include <array>
#include <cstdlib>
#include <cstdint>
#include <exception>
#include <fstream>
#include <iostream>
#include <iterator>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

namespace {

constexpr std::string_view DEFAULT_HOST = "127.0.0.1";
constexpr std::string_view DEFAULT_PORT = "7324";
constexpr std::string_view DEFAULT_PSK_FILE = "server-data/admin.psk";
constexpr std::string_view UNIVERSE_CONFIRMATION = "INITIALIZE FEDERATION";

enum class Operation {
   AddBbs,
   InitializeUniverse,
   Status,
   LiveBackup,
};

struct Options {
   std::string host{DEFAULT_HOST};
   std::string port{DEFAULT_PORT};
   std::string psk_file{DEFAULT_PSK_FILE};
   Operation operation = Operation::AddBbs;
   std::string bbs_name;
   std::string backup_label;
   std::optional<std::array<uint8_t, 16>> command_id;
};

void print_usage(std::ostream& output) {
   output
      << "usage: cepheus-trader-admin [OPTIONS] add-bbs BBS_NAME\n"
         "       cepheus-trader-admin [OPTIONS] initialize-universe\n"
         "       cepheus-trader-admin [OPTIONS] status\n"
         "       cepheus-trader-admin [OPTIONS] live-backup LABEL\n"
         "\n"
         "Options:\n"
         "  --host HOST         administrator host (default: 127.0.0.1)\n"
         "  --port PORT         administrator port (default: 7324)\n"
         "  --psk-file PATH     administrator PSK (default: server-data/admin.psk)\n"
         "  --command-id HEX    reuse a 16-byte command ID when retrying\n"
         "  add-bbs requires an initialized universe\n"
         "  --version           show the product version\n"
         "  -h, --help          show this help\n";
}

std::string require_option_value(int& index,
                                 const int argc,
                                 char** argv,
                                 const std::string_view option) {
   ++index;
   if(index >= argc) {
      throw std::invalid_argument(std::string(option) + " needs a value");
   }
   return argv[index];
}

std::array<uint8_t, 16> decode_command_id(const std::string& encoded) {
   if(encoded.size() != 32 ||
      !std::all_of(encoded.begin(), encoded.end(), [](const unsigned char value) {
         return (value >= '0' && value <= '9') ||
                (value >= 'a' && value <= 'f') ||
                (value >= 'A' && value <= 'F');
      })) {
      throw std::invalid_argument(
         "--command-id must be exactly 32 hexadecimal characters");
   }
   const auto decoded = ct::hex_decode(encoded);
   std::array<uint8_t, 16> command_id{};
   std::copy(decoded.begin(), decoded.end(), command_id.begin());
   return command_id;
}

Options parse_options(const int argc, char** argv) {
   Options options;
   std::vector<std::string> positional;
   for(int index = 1; index < argc; ++index) {
      const std::string_view argument = argv[index];
      if(argument == "-h" || argument == "--help") {
         print_usage(std::cout);
         std::exit(0);
      } else if(argument == "--version") {
         std::cout << "cepheus-trader-admin " << CT_PRODUCT_VERSION << '\n';
         std::exit(0);
      }
      if(argument == "--host") {
         options.host = require_option_value(index, argc, argv, argument);
      } else if(argument == "--port") {
         options.port = require_option_value(index, argc, argv, argument);
      } else if(argument == "--psk-file") {
         options.psk_file = require_option_value(index, argc, argv, argument);
      } else if(argument == "--command-id") {
         options.command_id =
            decode_command_id(require_option_value(index, argc, argv, argument));
      } else if(argument.starts_with("--")) {
         throw std::invalid_argument("unknown option: " + std::string(argument));
      } else {
         positional.emplace_back(argument);
      }
   }
   if(positional.size() == 2 && positional[0] == "add-bbs") {
      options.operation = Operation::AddBbs;
      options.bbs_name = std::move(positional[1]);
   } else if(positional.size() == 1 &&
             positional[0] == "initialize-universe") {
      options.operation = Operation::InitializeUniverse;
   } else if(positional.size() == 1 && positional[0] == "status") {
      options.operation = Operation::Status;
   } else if(positional.size() == 2 && positional[0] == "live-backup") {
      options.operation = Operation::LiveBackup;
      options.backup_label = std::move(positional[1]);
   } else {
      throw std::invalid_argument(
         "expected an administrator operation; use --help for usage");
   }
   return options;
}

std::vector<uint8_t> read_key(const std::string& path) {
   std::ifstream input(path, std::ios::binary);
   if(!input) {
      throw std::runtime_error("cannot open administrator PSK file: " + path);
   }
   const auto begin = std::istreambuf_iterator<char>(input);
   const auto end = std::istreambuf_iterator<char>();
   std::vector<uint8_t> key(begin, end);
   if(key.size() != 32) {
      throw std::runtime_error("administrator PSK file must contain exactly 32 bytes");
   }
   return key;
}

std::array<uint8_t, 16> generate_command_id() {
   return ct::random_command_id();
}

void confirm_universe_initialization() {
   std::cerr
      << "WARNING: This permanently deletes the existing game universe,\n"
         "including all players, sessions, queued commands, generated systems,\n"
         "ships, markets, messages, and simulation state. BBS enrollment,\n"
         "credentials, and sysop-selected BBS settings are preserved.\n"
         "Type "
      << UNIVERSE_CONFIRMATION << " to continue: " << std::flush;
   std::string response;
   if(!std::getline(std::cin, response) || response != UNIVERSE_CONFIRMATION) {
      throw std::runtime_error("universe initialization cancelled");
   }
}

}  // namespace

int main(int argc, char** argv) {
   try {
      const auto options = parse_options(argc, argv);
      if(options.operation == Operation::InitializeUniverse) {
         confirm_universe_initialization();
      }
      auto key = read_key(options.psk_file);
      const auto generated_command_id = !options.command_id.has_value();
      const auto command_id = options.command_id.has_value()
                                 ? *options.command_id
                                 : generate_command_id();
      try {
         ct::TlsConnection connection(
            options.host, options.port, "admin", std::move(key));
         if(options.operation == Operation::AddBbs) {
            const auto credential =
               ct::add_bbs(connection, options.bbs_name, command_id, 1);
            std::cout << "BBS id=" << credential.bbs_id
                      << " committed=" << credential.committed_sequence
                      << " psk=" << ct::hex_encode(credential.psk) << '\n';
         } else if(options.operation == Operation::InitializeUniverse) {
            const auto initialized =
               ct::initialize_universe(connection, command_id, 1);
            std::cout
               << "universe-id=" << ct::hex_encode(initialized.universe_id)
               << " committed=" << initialized.committed_sequence
               << " polities=" << initialized.polity_count
               << " systems=" << initialized.system_count
               << " worlds=" << initialized.world_count << '\n';
         } else if(options.operation == Operation::Status) {
            const auto status = ct::server_status(connection, 1);
            std::cout << "committed=" << status.committed_sequence << '\n'
                      << "game-second=" << status.game_second << '\n'
                      << "queued-inputs=" << status.queued_inputs << '\n'
                      << "bbs-count=" << status.bbs_count << '\n'
                      << "player-count=" << status.player_count << '\n'
                      << "system-count=" << status.system_count << '\n'
                      << "active-sessions=" << status.active_sessions << '\n'
                      << "storage-format=" << status.storage_format << '\n';
         } else {
            const auto complete = ct::live_backup(
               connection, options.backup_label, command_id, 1);
            std::cout << "backup=" << complete.label
                      << " committed=" << complete.committed_sequence
                      << " game-second=" << complete.game_second << '\n';
         }
         return 0;
      } catch(const ct::AdministratorRequestRejected& error) {
         std::cerr << error.what() << '\n';
         return 1;
      } catch(const std::exception& error) {
         std::cerr << error.what() << '\n';
         if(generated_command_id) {
            std::cerr << "If the request may have reached the server, retry with "
                         "--command-id "
                      << ct::hex_encode(command_id) << '\n';
         }
         return 1;
      }
   } catch(const std::exception& error) {
      std::cerr << error.what() << '\n';
      return 2;
   }
}
