#include "ct/bbs_credential.hpp"
#include "ct/crypto.hpp"
#include "ct/league_protocol.hpp"
#include "ct/tls_connection.hpp"

#include <algorithm>
#include <array>
#include <charconv>
#include <cstdlib>
#include <exception>
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

enum class Operation { InitCredential, Status, SetName, AddBbs, DisableBbs, EnableBbs };

struct Options {
   std::string host{"127.0.0.1"};
   std::string port{"7326"};
   std::string credential_file;
   std::string bbs_credential_file;
   Operation operation{Operation::Status};
   std::string name;
   std::string reason;
   uint32_t bbs_id{0};
   std::optional<uint64_t> expected_revision;
   std::optional<std::array<uint8_t, 16>> command_id;
};

class EchoGuard final {
   public:
      EchoGuard() {
#ifdef _WIN32
         m_input = GetStdHandle(STD_INPUT_HANDLE);
         if(m_input != INVALID_HANDLE_VALUE && GetConsoleMode(m_input, &m_mode)) {
            SetConsoleMode(m_input, m_mode & ~ENABLE_ECHO_INPUT);
            m_active = true;
         }
#else
         if(tcgetattr(STDIN_FILENO, &m_mode) == 0) {
            auto hidden = m_mode;
            hidden.c_lflag &= static_cast<tcflag_t>(~ECHO);
            if(tcsetattr(STDIN_FILENO, TCSAFLUSH, &hidden) == 0) {
               m_active = true;
            }
         }
#endif
      }
      ~EchoGuard() {
         if(!m_active) return;
#ifdef _WIN32
         SetConsoleMode(m_input, m_mode);
#else
         tcsetattr(STDIN_FILENO, TCSAFLUSH, &m_mode);
#endif
      }

   private:
      bool m_active{false};
#ifdef _WIN32
      HANDLE m_input{INVALID_HANDLE_VALUE};
      DWORD m_mode{0};
#else
      termios m_mode{};
#endif
};

void usage(std::ostream& out) {
   out << "usage: cepheus-trader-league [OPTIONS] status\n"
          "       cepheus-trader-league [OPTIONS] set-name NAME\n"
          "       cepheus-trader-league [OPTIONS] add-bbs NAME\n"
          "       cepheus-trader-league [OPTIONS] disable-bbs BBS_ID REASON\n"
          "       cepheus-trader-league [OPTIONS] enable-bbs BBS_ID\n"
          "       cepheus-trader-league init-credential FILE\n\n"
          "Options:\n"
          "  --host HOST                 server host (default: 127.0.0.1)\n"
          "  --port PORT                 league port (default: 7326)\n"
          "  --credential FILE           League Coordinator credential\n"
          "  --bbs-credential FILE       new BBS credential output for add-bbs\n"
          "  --expected-revision NUMBER  required for set/enable/disable\n"
          "  --command-id HEX            reuse an exactly-once command ID\n";
}

std::string next(int& index, const int argc, char** argv, std::string_view option) {
   if(++index >= argc) throw std::invalid_argument(std::string(option) + " needs a value");
   return argv[index];
}

template <typename Integer>
Integer number(const std::string& text, const char* label, const bool nonzero = false) {
   Integer value{};
   const auto [end, error] = std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size() || (nonzero && value == 0)) {
      throw std::invalid_argument(std::string(label) + " is invalid");
   }
   return value;
}

std::array<uint8_t, 16> command_id(const std::string& text) {
   if(text.size() != 32 || !std::all_of(text.begin(), text.end(), [](unsigned char byte) {
      return std::isxdigit(byte) != 0;
   })) throw std::invalid_argument("--command-id must be 32 hexadecimal characters");
   const auto decoded = ct::hex_decode(text);
   std::array<uint8_t, 16> result{};
   std::copy(decoded.begin(), decoded.end(), result.begin());
   return result;
}

Options parse(const int argc, char** argv) {
   Options options;
   std::vector<std::string> positional;
   for(int index = 1; index < argc; ++index) {
      const std::string_view argument = argv[index];
      if(argument == "--help" || argument == "-h") { usage(std::cout); std::exit(0); }
      if(argument == "--version") {
         std::cout << "cepheus-trader-league " << CT_PRODUCT_VERSION << '\n';
         std::exit(0);
      }
      if(argument == "--host") options.host = next(index, argc, argv, argument);
      else if(argument == "--port") options.port = next(index, argc, argv, argument);
      else if(argument == "--credential") options.credential_file = next(index, argc, argv, argument);
      else if(argument == "--bbs-credential") options.bbs_credential_file = next(index, argc, argv, argument);
      else if(argument == "--expected-revision") options.expected_revision = number<uint64_t>(next(index, argc, argv, argument), "revision");
      else if(argument == "--command-id") options.command_id = command_id(next(index, argc, argv, argument));
      else if(argument.starts_with("--")) throw std::invalid_argument("unknown option: " + std::string(argument));
      else positional.emplace_back(argument);
   }
   if(positional.size() == 2 && positional[0] == "init-credential") {
      options.operation = Operation::InitCredential; options.credential_file = positional[1];
      return options;
   }
   if(options.credential_file.empty()) throw std::invalid_argument("--credential FILE is required");
   if(positional.size() == 1 && positional[0] == "status") options.operation = Operation::Status;
   else if(positional.size() == 2 && positional[0] == "set-name") { options.operation = Operation::SetName; options.name = positional[1]; }
   else if(positional.size() == 2 && positional[0] == "add-bbs") { options.operation = Operation::AddBbs; options.name = positional[1]; }
   else if(positional.size() == 3 && positional[0] == "disable-bbs") { options.operation = Operation::DisableBbs; options.bbs_id = number<uint32_t>(positional[1], "BBS ID", true); options.reason = positional[2]; }
   else if(positional.size() == 2 && positional[0] == "enable-bbs") { options.operation = Operation::EnableBbs; options.bbs_id = number<uint32_t>(positional[1], "BBS ID", true); }
   else throw std::invalid_argument("expected a league operation; use --help");
   if(options.operation != Operation::Status && !options.expected_revision.has_value() && options.operation != Operation::AddBbs)
      throw std::invalid_argument("--expected-revision is required");
   if(options.operation == Operation::AddBbs && options.bbs_credential_file.empty())
      throw std::invalid_argument("add-bbs requires --bbs-credential FILE");
   return options;
}

void init_credential(const std::string& path) {
   std::string id_text;
   std::cerr << "League ID: " << std::flush;
   if(!std::getline(std::cin, id_text)) throw std::runtime_error("could not read League ID");
   std::cerr << "League PSK (64 hexadecimal characters): " << std::flush;
   std::string psk_text;
   {
      EchoGuard guard;
      if(!std::getline(std::cin, psk_text)) throw std::runtime_error("could not read League PSK");
   }
   std::cerr << '\n';
   if(psk_text.size() != 64) throw std::invalid_argument("League PSK must contain 64 hexadecimal characters");
   auto psk = ct::hex_decode(psk_text);
   ct::create_league_credential_file(path, number<uint32_t>(id_text, "League ID", true), psk);
   ct::scrub_memory(psk);
   std::fill(psk_text.begin(), psk_text.end(), '\0');
}

void print_status(const ct::LeagueCoordinatorStatus& status) {
   std::cout << "league-id=" << status.league_id << '\n'
             << "name=" << status.name << '\n'
             << "revision=" << status.revision << '\n'
             << "committed=" << status.committed_sequence << '\n';
   for(const auto& member : status.members) {
      std::cout << "bbs=" << member.bbs_id << " name=" << member.bbs_name
                << " enabled=" << (member.enabled ? "yes" : "no")
                << " revision=" << member.revision;
      if(!member.reason.empty()) std::cout << " reason=" << member.reason;
      std::cout << '\n';
   }
}

}  // namespace

int main(int argc, char** argv) {
   std::optional<std::array<uint8_t, 16>> retry_command_id;
   try {
      const auto options = parse(argc, argv);
      if(options.operation == Operation::InitCredential) {
         init_credential(options.credential_file);
         std::cout << "credential-created=" << options.credential_file << '\n';
         return 0;
      }
      auto credential = ct::read_league_credential_file(options.credential_file);
      ct::TlsConnection connection(options.host, options.port,
                                   std::to_string(credential.league_id),
                                   std::move(credential.psk));
      const auto id = options.command_id.value_or(ct::random_command_id());
      if(!options.command_id.has_value() && options.operation != Operation::Status) {
         retry_command_id = id;
      }
      if(options.operation == Operation::Status) print_status(ct::league_status(connection, 1));
      else if(options.operation == Operation::SetName) {
         const auto status = ct::set_league_name(connection, options.name,
            *options.expected_revision, id, 1);
         print_status(status);
         if(status.stale) {
            retry_command_id.reset();
            return 1;
         }
      } else if(options.operation == Operation::AddBbs) {
         const auto bbs = ct::add_league_bbs(connection, options.name, id, 1);
         ct::create_bbs_credential_file(options.bbs_credential_file, bbs.bbs_id, bbs.psk);
         std::cout << "bbs-id=" << bbs.bbs_id << " committed=" << bbs.committed_sequence
                   << " credential=" << options.bbs_credential_file << '\n';
      } else {
         bool stale = false;
         const auto member = ct::set_league_bbs_access(connection, options.bbs_id,
            *options.expected_revision, options.operation == Operation::EnableBbs,
            options.reason, id, 1, stale);
         std::cout << "bbs-id=" << member.bbs_id << " enabled="
                   << (member.enabled ? "yes" : "no") << " revision=" << member.revision << '\n';
         if(stale) {
            retry_command_id.reset();
            return 1;
         }
      }
      retry_command_id.reset();
      return 0;
   } catch(const std::exception& error) {
      std::cerr << error.what() << '\n';
      if(retry_command_id) {
         std::cerr << "If the request may have reached the server, retry with "
                      "--command-id "
                   << ct::hex_encode(*retry_command_id) << '\n';
      }
      return 2;
   }
}
