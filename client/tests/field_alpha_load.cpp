#include "ct/bbs_credential.hpp"
#include "ct/protocol.hpp"
#include "ct/tls_connection.hpp"

#include <charconv>
#include <chrono>
#include <cstdint>
#include <iostream>
#include <limits>
#include <memory>
#include <stdexcept>
#include <string>
#include <string_view>
#include <thread>
#include <vector>

namespace {

uint32_t parse_positive(const char* text, const char* name) {
   uint32_t value = 0;
   const std::string input(text);
   const auto [end, error] =
      std::from_chars(input.data(), input.data() + input.size(), value);
   if(error != std::errc() || end != input.data() + input.size() || value == 0) {
      throw std::invalid_argument(std::string(name) + " must be a positive UInt32");
   }
   return value;
}

struct HeldSession {
   std::unique_ptr<ct::TlsConnection> connection;
   ct::PlayerIdentity identity;
   ct::PlayerPhase phase;
};

}  // namespace

int main(int argc, char** argv) {
   try {
      if(argc == 2 && std::string_view(argv[1]) == "--version") {
         std::cout << "ct-field-alpha-load " << CT_PRODUCT_VERSION << '\n';
         return 0;
      }
      if(argc < 7) {
         std::cerr
            << "usage: ct-field-alpha-load HOST PORT HOLD_SECONDS CYCLES "
               "SESSIONS_PER_BBS CREDENTIAL_FILE [CREDENTIAL_FILE ...]\n";
         return 2;
      }

      const std::string host(argv[1]);
      const std::string service(argv[2]);
      const auto hold_seconds = parse_positive(argv[3], "HOLD_SECONDS");
      const auto cycles = parse_positive(argv[4], "CYCLES");
      const auto sessions_per_bbs =
         parse_positive(argv[5], "SESSIONS_PER_BBS");
      const auto credential_count = static_cast<uint32_t>(argc - 6);
      if(sessions_per_bbs >
         std::numeric_limits<uint32_t>::max() / credential_count) {
         throw std::invalid_argument("requested session count overflows UInt32");
      }

      std::vector<ct::BbsCredentialFile> credentials;
      credentials.reserve(credential_count);
      for(int index = 6; index < argc; ++index) {
         credentials.push_back(ct::read_bbs_credential_file(argv[index]));
      }

      const auto requested_sessions = credential_count * sessions_per_bbs;
      for(uint32_t cycle = 1; cycle <= cycles; ++cycle) {
         std::vector<HeldSession> sessions;
         sessions.reserve(requested_sessions);
         const auto started = std::chrono::steady_clock::now();

         for(const auto& credential : credentials) {
            for(uint64_t player_number = 1;
                player_number <= sessions_per_bbs;
                ++player_number) {
               const auto player_id = static_cast<uint32_t>(player_number);
               auto connection = std::make_unique<ct::TlsConnection>(
                  host,
                  service,
                  std::to_string(credential.bbs_id),
                  credential.psk);
               const ct::PlayerIdentity identity{
                  .bbs_id = credential.bbs_id,
                  .player_id = player_id,
               };
               const auto hello = ct::exchange_hello(
                  *connection, identity, "ct-field-alpha-load/0.1", "en-US");
               sessions.push_back(HeldSession{
                  .connection = std::move(connection),
                  .identity = identity,
                  .phase = hello.phase,
               });
            }
         }

         const auto connected_at = std::chrono::steady_clock::now();
         const auto connect_milliseconds =
            std::chrono::duration_cast<std::chrono::milliseconds>(
               connected_at - started)
               .count();
         std::cout << "READY cycle=" << cycle << '/' << cycles
                   << " bbs=" << credential_count
                   << " sessions=" << sessions.size()
                   << " connect-ms=" << connect_milliseconds
                   << " hold-seconds=" << hold_seconds << '\n'
                   << std::flush;

         std::this_thread::sleep_for(std::chrono::seconds(hold_seconds));
         sessions.clear();
         std::cout << "DISCONNECTED cycle=" << cycle << '/' << cycles
                   << " sessions=" << requested_sessions << '\n'
                   << std::flush;

         if(cycle != cycles) {
            std::this_thread::sleep_for(std::chrono::seconds(1));
         }
      }

      return 0;
   } catch(const std::exception& error) {
      std::cerr << error.what() << '\n';
      return 1;
   }
}
