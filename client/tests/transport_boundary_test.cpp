#include "ct/client_transport.h"
#include "ct/tls_connection.hpp"

#include <cstdint>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

void check(const bool condition) {
   if(!condition) {
      throw std::runtime_error("client transport boundary test failed");
   }
}

std::string last_error_message() {
   ct_client_error_info info{};
   check(ct_client_last_error_info(&info) == CT_CLIENT_OK);
   std::vector<char> message(info.message_bytes);
   check(ct_client_last_error_copy(message.data(), message.size()) == CT_CLIENT_OK);
   return message.data();
}

}  // namespace

int main() {
   check(ct_client_randomize(nullptr, 1) == CT_CLIENT_ERROR);
   ct_client_error_info info{};
   check(ct_client_last_error_info(&info) == CT_CLIENT_OK);
   check(info.code == CT_CLIENT_ERROR_INVALID_ARGUMENT);
   check(info.native_code == 0);
   check(last_error_message() == "random output buffer is null");

   ct_client_connection* connection = nullptr;
   std::vector<uint8_t> short_psk(31, 0x5a);
   check(ct_client_connection_create(
            "host-not-contacted",
            "1",
            "1",
            short_psk.data(),
            short_psk.size(),
            &connection) == CT_CLIENT_ERROR);
   check(connection == nullptr);
   check(last_error_message() == "PSK must contain at least 32 bytes");

   bool caught = false;
   try {
      ct::TlsConnection tls(
         "host-not-contacted", "1", "1", std::vector<uint8_t>(31, 0x5a));
   } catch(const std::runtime_error& error) {
      caught = std::string(error.what()).find(
         "PSK must contain at least 32 bytes") != std::string::npos;
   }
   check(caught);
   return 0;
}
