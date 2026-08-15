#include "ct/crypto.hpp"

#include "ct/client_transport.h"

#include <stdexcept>

namespace ct {
namespace {

[[noreturn]] void throw_transport_error() {
   ct_client_error_info info{};
   std::string message = "client transport failure";
   if(ct_client_last_error_info(&info) == CT_CLIENT_OK && info.message_bytes != 0) {
      std::vector<char> copied(info.message_bytes);
      if(ct_client_last_error_copy(copied.data(), copied.size()) == CT_CLIENT_OK) {
         message = copied.data();
      }
   }
   throw std::runtime_error(message);
}

uint8_t hex_nibble(const char value) {
   if(value >= '0' && value <= '9') {
      return static_cast<uint8_t>(value - '0');
   }
   if(value >= 'a' && value <= 'f') {
      return static_cast<uint8_t>(value - 'a' + 10);
   }
   if(value >= 'A' && value <= 'F') {
      return static_cast<uint8_t>(value - 'A' + 10);
   }
   throw std::invalid_argument("invalid hexadecimal input");
}

}  // namespace

class CommandIdGenerator::Impl {};

CommandIdGenerator::CommandIdGenerator() : impl_(std::make_unique<Impl>()) {}
CommandIdGenerator::~CommandIdGenerator() = default;
CommandIdGenerator::CommandIdGenerator(CommandIdGenerator&&) noexcept = default;
CommandIdGenerator& CommandIdGenerator::operator=(CommandIdGenerator&&) noexcept = default;

std::array<uint8_t, 16> CommandIdGenerator::next() {
   std::array<uint8_t, 16> result{};
   if(ct_client_randomize(result.data(), result.size()) != CT_CLIENT_OK) {
      throw_transport_error();
   }
   return result;
}

std::array<uint8_t, 16> random_command_id() {
   CommandIdGenerator generator;
   return generator.next();
}

std::array<uint8_t, 32> sha256(const std::span<const uint8_t> bytes) {
   std::array<uint8_t, 32> result{};
   if(ct_client_sha256(bytes.data(), bytes.size(), result.data()) != CT_CLIENT_OK) {
      throw_transport_error();
   }
   return result;
}

std::vector<uint8_t> hex_decode(const std::string_view encoded) {
   if(encoded.size() % 2 != 0) {
      throw std::invalid_argument("hexadecimal input has an odd length");
   }
   std::vector<uint8_t> result(encoded.size() / 2);
   for(size_t index = 0; index < result.size(); ++index) {
      result[index] = static_cast<uint8_t>(
         (hex_nibble(encoded[index * 2]) << 4) |
         hex_nibble(encoded[index * 2 + 1]));
   }
   return result;
}

std::string hex_encode(const std::span<const uint8_t> bytes) {
   static constexpr char DIGITS[] = "0123456789ABCDEF";
   std::string result(bytes.size() * 2, '\0');
   for(size_t index = 0; index < bytes.size(); ++index) {
      result[index * 2] = DIGITS[bytes[index] >> 4];
      result[index * 2 + 1] = DIGITS[bytes[index] & 0x0f];
   }
   return result;
}

void scrub_memory(std::string& value) noexcept {
   ct_client_scrub(value.data(), value.size());
}

void scrub_memory(std::vector<uint8_t>& value) noexcept {
   ct_client_scrub(value.data(), value.size());
}

void scrub_memory(const std::span<uint8_t> value) noexcept {
   ct_client_scrub(value.data(), value.size());
}

}  // namespace ct
