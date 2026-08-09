#include "ct/bbs_credential.hpp"
#include "ct/crypto.hpp"
#include "ct/protocol.hpp"
#include "ct/tls_connection.hpp"

#include <algorithm>
#include <array>
#include <charconv>
#include <cstdint>
#include <exception>
#include <iostream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace {

uint32_t parse_id(const std::string& text, const char* name) {
   uint32_t value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size()) {
      throw std::invalid_argument(std::string(name) + " must be a UInt32");
   }
   return value;
}

}  // namespace

int main(int argc, char** argv) {
   try {
      if(argc == 2 && std::string_view(argv[1]) == "--version") {
         std::cout << "cepheus-trader-client " << CT_PRODUCT_VERSION << '\n';
         return 0;
      }
      if(argc != 5 &&
         (argc < 9 || std::string(argv[5]) != "create-player")) {
         std::cerr
            << "usage: cepheus-trader-client HOST PORT CREDENTIAL_FILE PLAYER_ID"
               " [create-player COMMAND_ID_HEX CAPTAIN_NAME SHIP_NAME"
               " [CREW_NAME ...]]\n";
         return 2;
      }
      auto credential = ct::read_bbs_credential_file(argv[3]);
      const auto player_id = parse_id(argv[4], "PLAYER_ID");
      ct::TlsConnection connection(
         argv[1],
         argv[2],
         std::to_string(credential.bbs_id),
         std::move(credential.psk));
      const ct::PlayerIdentity identity{
         .bbs_id = credential.bbs_id,
         .player_id = player_id,
      };
      const auto hello = ct::exchange_hello(
         connection,
         identity,
         "cepheus-trader-client/" CT_PRODUCT_VERSION,
         "en-US");
      const auto phase = [&hello] {
         switch(hello.phase) {
            case ct::PlayerPhase::Disconnected:
               return "disconnected";
            case ct::PlayerPhase::Jump:
               return "jump";
            case ct::PlayerPhase::Interplanetary:
               return "interplanetary";
            case ct::PlayerPhase::Encounter:
               return "encounter";
            case ct::PlayerPhase::OnPlanet:
               return "on-planet";
            case ct::PlayerPhase::NewUser:
               return "new-user";
            case ct::PlayerPhase::Docked:
               return "docked";
            case ct::PlayerPhase::Terminal:
               return "terminal";
            case ct::PlayerPhase::Other:
               return "other";
         }
         return "other";
      }();
      std::cout << "HELLO bbs=" << hello.identity.bbs_id
                << " player=" << hello.identity.player_id
                << " epoch=" << hello.assigned_epoch
                << " committed=" << hello.committed_sequence
                << " phase=" << phase
                << " language=" << hello.language_tag
                << " tls=" << connection.protocol_version() << '\n';
      if(argc > 5) {
         if(hello.phase != ct::PlayerPhase::NewUser) {
            throw std::runtime_error("CreatePlayer is only valid in the new-user phase");
         }
         const auto decoded_command_id = ct::hex_decode(argv[6]);
         if(decoded_command_id.size() != 16) {
            throw std::invalid_argument(
               "COMMAND_ID_HEX must encode exactly 16 bytes");
         }
         std::array<uint8_t, 16> command_id{};
         std::copy(
            decoded_command_id.begin(), decoded_command_id.end(), command_id.begin());
         auto query_id = command_id;
         query_id[0] ^= 0x41;
         const auto captain_options = ct::get_captain_creation_options(
            connection, hello.assigned_epoch, query_id, 1);
         query_id[0] ^= 0x03;
         const auto offers = ct::get_starting_ship_offers(
            connection, hello.assigned_epoch, query_id, 2);
         if(offers.offers.empty()) {
            throw std::runtime_error("server returned no starting offers");
         }
         const auto& offer = offers.offers.front();
         query_id[0] ^= 0x07;
         const auto crew_plan = ct::get_starting_crew_plan(
            connection,
            hello.assigned_epoch,
            offers.setup_revision,
            offer.offer_id,
            query_id,
            3);
         ct::PlayerCreation creation{
            .setup_revision = offers.setup_revision,
            .starting_offer_id = offer.offer_id,
            .captain = captain_options.default_captain,
            .ship_name = argv[8],
            .crew = {},
            .refit_option_ids = {1},
         };
         creation.captain.name = argv[7];
         for(size_t index = 0; index < crew_plan.slots.size(); ++index) {
            auto name = crew_plan.slots[index].default_crew.name;
            const auto argument = static_cast<int>(index) + 9;
            if(argument < argc) {
               name = argv[argument];
            }
            creation.crew.push_back(ct::InitialCrewDraft{
               .slot_id = crew_plan.slots[index].slot_id,
               .name = std::move(name),
               .training_skill =
                  crew_plan.slots[index].default_crew.training.skill,
            });
         }
         const auto created = ct::create_player(
            connection, hello.assigned_epoch, creation, command_id, 4);
         auto crew_query_id = command_id;
         crew_query_id[0] ^= 0x51;
         const auto managed_crew = ct::get_crew_management(
            connection, hello.assigned_epoch, crew_query_id, 5);
         if(managed_crew.members.size() != creation.crew.size() + 1 ||
            managed_crew.roles.empty()) {
            throw std::runtime_error(
               "server returned an incomplete Crew Management snapshot");
         }
         auto ship_query_id = command_id;
         ship_query_id[0] ^= 0x52;
         const auto ship_status = ct::get_ship_status(
            connection, hello.assigned_epoch, ship_query_id, 6);
         if(ship_status.ship_name != creation.ship_name ||
            ship_status.subsystems.empty()) {
            throw std::runtime_error(
               "server returned an incomplete Ship Status snapshot");
         }
         std::cout << "CREATED captain=" << created.creation.captain.name
                   << " ship=" << created.creation.ship_name
                   << " crew=" << created.creation.crew.size()
                   << " committed=" << created.committed_sequence
                   << " phase=docked\n";
      }
      return 0;
   } catch(const std::exception& error) {
      std::cerr << error.what() << '\n';
      return 1;
   }
}
