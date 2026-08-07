#pragma once

#include <array>
#include <cstdint>
#include <memory>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace ct {

class CommandIdGenerator {
public:
   CommandIdGenerator();
   ~CommandIdGenerator();

   CommandIdGenerator(CommandIdGenerator&&) noexcept;
   CommandIdGenerator& operator=(CommandIdGenerator&&) noexcept;

   CommandIdGenerator(const CommandIdGenerator&) = delete;
   CommandIdGenerator& operator=(const CommandIdGenerator&) = delete;

   std::array<uint8_t, 16> next();

private:
   class Impl;
   std::unique_ptr<Impl> impl_;
};

std::array<uint8_t, 16> random_command_id();
std::vector<uint8_t> hex_decode(std::string_view encoded);
std::string hex_encode(std::span<const uint8_t> bytes);
void scrub_memory(std::string& value) noexcept;
void scrub_memory(std::vector<uint8_t>& value) noexcept;

}  // namespace ct
