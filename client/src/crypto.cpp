#include "ct/crypto.hpp"

#include <botan/auto_rng.h>
#include <botan/hex.h>
#include <botan/mem_ops.h>

#include <utility>

namespace ct {

class CommandIdGenerator::Impl {
public:
   Botan::AutoSeeded_RNG random;
};

CommandIdGenerator::CommandIdGenerator()
   : impl_(std::make_unique<Impl>()) {
}

CommandIdGenerator::~CommandIdGenerator() = default;

CommandIdGenerator::CommandIdGenerator(CommandIdGenerator&&) noexcept = default;

CommandIdGenerator& CommandIdGenerator::operator=(CommandIdGenerator&&) noexcept = default;

std::array<uint8_t, 16> CommandIdGenerator::next() {
   std::array<uint8_t, 16> result{};
   impl_->random.randomize(result);
   return result;
}

std::array<uint8_t, 16> random_command_id() {
   CommandIdGenerator generator;
   return generator.next();
}

std::vector<uint8_t> hex_decode(const std::string_view encoded) {
   return Botan::hex_decode(encoded);
}

std::string hex_encode(const std::span<const uint8_t> bytes) {
   return Botan::hex_encode(bytes);
}

void scrub_memory(std::string& value) noexcept {
   Botan::secure_scrub_memory(value.data(), value.size());
}

void scrub_memory(std::vector<uint8_t>& value) noexcept {
   Botan::secure_scrub_memory(value.data(), value.size());
}

}  // namespace ct
