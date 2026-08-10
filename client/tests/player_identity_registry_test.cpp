#include "ct/player_identity_registry.hpp"

#include <botan/hash.h>

#include <chrono>
#include <filesystem>
#include <fstream>
#include <stdexcept>
#include <vector>

#ifndef _WIN32
#include <sys/stat.h>
#endif

namespace {

void check(const bool condition) {
   if(!condition) {
      throw std::runtime_error("player identity registry test failed");
   }
}

class ScratchDirectory final {
public:
   ScratchDirectory() {
      const auto suffix =
         std::chrono::steady_clock::now().time_since_epoch().count();
      path = std::filesystem::temp_directory_path() /
             ("cepheus-trader-identities-" + std::to_string(suffix));
      std::filesystem::create_directory(path);
   }
   ~ScratchDirectory() {
      std::error_code ignored;
      std::filesystem::remove_all(path, ignored);
   }
   std::filesystem::path path;
};

template<typename Function>
bool throws(Function&& function) {
   try {
      function();
      return false;
   } catch(const std::exception&) {
      return true;
   }
}

void append_u16(std::vector<uint8_t>& bytes, const uint16_t value) {
   bytes.push_back(static_cast<uint8_t>(value));
   bytes.push_back(static_cast<uint8_t>(value >> 8));
}

void append_u32(std::vector<uint8_t>& bytes, const uint32_t value) {
   for(unsigned shift = 0; shift != 32; shift += 8) {
      bytes.push_back(static_cast<uint8_t>(value >> shift));
   }
}

void write_version_one_registry(const std::filesystem::path& path) {
   std::vector<uint8_t> bytes{'C', 'T', 'I', 'D', 'M', 'A', 'P', 0};
   append_u16(bytes, 1);
   append_u16(bytes, 0);
   append_u32(bytes, 17);
   append_u32(bytes, 2);
   append_u32(bytes, 1);
   bytes.push_back(0);
   bytes.push_back(1);
   append_u16(bytes, 6);
   append_u32(bytes, 1);
   append_u32(bytes, 42);
   bytes.insert(bytes.end(), {'L', 'e', 'g', 'a', 'c', 'y'});
   auto hash = Botan::HashFunction::create_or_throw("SHA-256");
   hash->update(bytes);
   const auto digest = hash->final();
   bytes.insert(bytes.end(), digest.begin(), digest.end());
   std::ofstream output(path, std::ios::binary);
   output.write(reinterpret_cast<const char*>(bytes.data()),
                static_cast<std::streamsize>(bytes.size()));
   output.close();
#ifndef _WIN32
   chmod(path.c_str(), S_IRUSR | S_IWUSR);
#endif
}

}  // namespace

int main() {
   ScratchDirectory scratch;
   const auto legacy_path = scratch.path / "legacy.bin";
   write_version_one_registry(legacy_path);
   const auto legacy =
      ct::resolve_player_identity(legacy_path.string(), 17, "Legacy", 42);
   check(legacy.player_id == 1);
   check(legacy.help_level == ct::HelpLevel::Beginner);
   check(!legacy.orientation_shown);
   ct::mark_player_orientation_shown(legacy_path.string(), 17, 1);
   check(ct::resolve_player_identity(legacy_path.string(), 17, "Legacy", 42)
            .orientation_shown);

   const auto path = (scratch.path / "players.bin").string();
   ct::create_player_identity_registry(path, 17);
#ifndef _WIN32
   struct stat status {};
   check(stat(path.c_str(), &status) == 0);
   check((status.st_mode & (S_IRWXG | S_IRWXO)) == 0);
#endif

   const auto first = ct::resolve_player_identity(path, 17, "Jane Doe", 42);
   check(first.player_id == 1);
   check(first.help_level == ct::HelpLevel::Beginner);
   check(!first.orientation_shown);
   check(ct::resolve_player_identity(path, 17, "Jane Doe", 42).player_id ==
         first.player_id);
   check(throws([&] {
      (void)ct::resolve_player_identity(path, 17, "Jane Smith", 42);
   }));
   check(throws([&] {
      (void)ct::resolve_player_identity(path, 17, "Jane Doe", 43);
   }));

   ct::set_player_help_level(path, 17, first.player_id, ct::HelpLevel::Expert);
   ct::mark_player_orientation_shown(path, 17, first.player_id);
   auto updated = ct::resolve_player_identity(path, 17, "Jane Doe", 42);
   check(updated.help_level == ct::HelpLevel::Expert);
   check(updated.orientation_shown);
   ct::rename_player_identity(path, 17, first.player_id, "Jane Smith");
   ct::reindex_player_identity(path, 17, first.player_id, 43);
   check(ct::resolve_player_identity(path, 17, "Jane Smith", 43).player_id ==
         first.player_id);
   ct::retire_player_identity(path, 17, first.player_id);
   const auto replacement =
      ct::resolve_player_identity(path, 17, "Jane Smith", 43);
   check(replacement.player_id == 2);

   const auto entries = ct::list_player_identities(path, 17);
   check(entries.size() == 2);
   check(entries[0].retired);
   check(!entries[1].retired);
   check(throws([&] {
      (void)ct::list_player_identities(path, 18);
   }));
}
