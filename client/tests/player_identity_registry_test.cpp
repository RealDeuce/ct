#include "ct/player_identity_registry.hpp"

#include <chrono>
#include <filesystem>
#include <stdexcept>

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

}  // namespace

int main() {
   ScratchDirectory scratch;
   const auto path = (scratch.path / "players.bin").string();
   ct::create_player_identity_registry(path, 17);
#ifndef _WIN32
   struct stat status {};
   check(stat(path.c_str(), &status) == 0);
   check((status.st_mode & (S_IRWXG | S_IRWXO)) == 0);
#endif

   const auto first = ct::resolve_player_identity(path, 17, "Jane Doe", 42);
   check(first == 1);
   check(ct::resolve_player_identity(path, 17, "Jane Doe", 42) == first);
   check(throws([&] {
      (void)ct::resolve_player_identity(path, 17, "Jane Smith", 42);
   }));
   check(throws([&] {
      (void)ct::resolve_player_identity(path, 17, "Jane Doe", 43);
   }));

   ct::rename_player_identity(path, 17, first, "Jane Smith");
   ct::reindex_player_identity(path, 17, first, 43);
   check(ct::resolve_player_identity(path, 17, "Jane Smith", 43) == first);
   ct::retire_player_identity(path, 17, first);
   const auto replacement =
      ct::resolve_player_identity(path, 17, "Jane Smith", 43);
   check(replacement == 2);

   const auto entries = ct::list_player_identities(path, 17);
   check(entries.size() == 2);
   check(entries[0].retired);
   check(!entries[1].retired);
   check(throws([&] {
      (void)ct::list_player_identities(path, 18);
   }));
}
