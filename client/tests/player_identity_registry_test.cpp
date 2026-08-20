#include "ct/player_identity_registry.hpp"
#include "ct/crypto.hpp"

#include <chrono>
#include <filesystem>
#include <fstream>
#include <iterator>
#include <stdexcept>
#include <vector>

#ifdef _WIN32
#include <windows.h>
#include <aclapi.h>
#else
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
   const auto digest = ct::sha256(bytes);
   bytes.insert(bytes.end(), digest.begin(), digest.end());
   std::ofstream output(path, std::ios::binary);
   output.write(reinterpret_cast<const char*>(bytes.data()),
                static_cast<std::streamsize>(bytes.size()));
   output.close();
#ifndef _WIN32
   chmod(path.c_str(), S_IRUSR | S_IWUSR);
#endif
}

void write_version_two_registry(const std::filesystem::path& path) {
   std::vector<uint8_t> bytes{'C', 'T', 'I', 'D', 'M', 'A', 'P', 0};
   append_u16(bytes, 2);
   append_u16(bytes, 0);
   append_u32(bytes, 17);
   append_u32(bytes, 2);
   append_u32(bytes, 1);
   bytes.push_back(0);
   bytes.push_back(1);
   bytes.push_back(1);
   bytes.push_back(1);
   append_u16(bytes, 8);
   append_u32(bytes, 1);
   append_u32(bytes, 43);
   bytes.insert(bytes.end(), {'V', '2', ' ', 'U', 's', 'e', 'r', '!'});
   const auto digest = ct::sha256(bytes);
   bytes.insert(bytes.end(), digest.begin(), digest.end());
   std::ofstream output(path, std::ios::binary);
   output.write(reinterpret_cast<const char*>(bytes.data()),
                static_cast<std::streamsize>(bytes.size()));
   output.close();
#ifndef _WIN32
   chmod(path.c_str(), S_IRUSR | S_IWUSR);
#endif
}

void write_version_three_registry(const std::filesystem::path& path) {
   std::vector<uint8_t> bytes{'C', 'T', 'I', 'D', 'M', 'A', 'P', 0};
   append_u16(bytes, 3);
   append_u16(bytes, 0);
   append_u32(bytes, 17);
   append_u32(bytes, 2);
   append_u32(bytes, 1);
   bytes.push_back(0);
   bytes.push_back(1);
   bytes.push_back(0);
   bytes.push_back(1);
   bytes.push_back(0);
   append_u16(bytes, 7);
   append_u32(bytes, 1);
   append_u32(bytes, 44);
   bytes.insert(bytes.end(), {'V', '3', ' ', 'U', 's', 'e', 'r'});
   const auto digest = ct::sha256(bytes);
   bytes.insert(bytes.end(), digest.begin(), digest.end());
   std::ofstream output(path, std::ios::binary);
   output.write(reinterpret_cast<const char*>(bytes.data()),
                static_cast<std::streamsize>(bytes.size()));
   output.close();
#ifndef _WIN32
   chmod(path.c_str(), S_IRUSR | S_IWUSR);
#endif
}

void corrupt_first_watch_disposition(const std::filesystem::path& path) {
   std::ifstream input(path, std::ios::binary);
   std::vector<uint8_t> bytes{
      std::istreambuf_iterator<char>(input), std::istreambuf_iterator<char>()};
   input.close();
   check(bytes.size() > 29 + 32);
   bytes[29] = 255;
   bytes.resize(bytes.size() - 32);
   const auto digest = ct::sha256(bytes);
   bytes.insert(bytes.end(), digest.begin(), digest.end());
   std::ofstream output(path, std::ios::binary | std::ios::trunc);
   output.write(reinterpret_cast<const char*>(bytes.data()),
                static_cast<std::streamsize>(bytes.size()));
}

#ifdef _WIN32

std::vector<uint8_t> current_user_sid() {
   HANDLE token = nullptr;
   check(OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &token) != 0);
   DWORD size = 0;
   (void)GetTokenInformation(token, TokenUser, nullptr, 0, &size);
   check(size != 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER);
   std::vector<uint8_t> token_buffer(size);
   check(GetTokenInformation(
      token, TokenUser, token_buffer.data(), size, &size) != 0);
   CloseHandle(token);
   const auto user = reinterpret_cast<TOKEN_USER*>(token_buffer.data());
   std::vector<uint8_t> result(GetLengthSid(user->User.Sid));
   check(CopySid(
      static_cast<DWORD>(result.size()), result.data(), user->User.Sid) != 0);
   return result;
}

std::vector<uint8_t> file_owner_sid(const std::filesystem::path& path) {
   PSECURITY_DESCRIPTOR descriptor = nullptr;
   PSID owner = nullptr;
   const auto result = GetNamedSecurityInfoW(
      path.c_str(), SE_FILE_OBJECT, OWNER_SECURITY_INFORMATION,
      &owner, nullptr, nullptr, nullptr, &descriptor);
   check(result == ERROR_SUCCESS && descriptor != nullptr && owner != nullptr);
   std::vector<uint8_t> bytes(GetLengthSid(owner));
   check(CopySid(static_cast<DWORD>(bytes.size()), bytes.data(), owner) != 0);
   LocalFree(descriptor);
   return bytes;
}

bool same_sid(const std::vector<uint8_t>& left,
              const std::vector<uint8_t>& right) {
   return EqualSid(
      const_cast<uint8_t*>(left.data()),
      const_cast<uint8_t*>(right.data())) != 0;
}

bool set_administrators_owner(const std::filesystem::path& path,
                              std::vector<uint8_t>& administrator_sid) {
   DWORD size = SECURITY_MAX_SID_SIZE;
   administrator_sid.resize(size);
   if(CreateWellKnownSid(
         WinBuiltinAdministratorsSid, nullptr, administrator_sid.data(),
         &size) == 0) {
      return false;
   }
   administrator_sid.resize(size);

   HANDLE token = nullptr;
   if(OpenProcessToken(
         GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
         &token) != 0) {
      LUID identifier{};
      if(LookupPrivilegeValueW(
            nullptr, L"SeRestorePrivilege", &identifier) != 0) {
         TOKEN_PRIVILEGES requested{};
         requested.PrivilegeCount = 1;
         requested.Privileges[0].Luid = identifier;
         requested.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
         (void)AdjustTokenPrivileges(
            token, FALSE, &requested, 0, nullptr, nullptr);
      }
      CloseHandle(token);
   }
   return SetNamedSecurityInfoW(
      const_cast<wchar_t*>(path.c_str()), SE_FILE_OBJECT,
      OWNER_SECURITY_INFORMATION, administrator_sid.data(), nullptr,
      nullptr, nullptr) == ERROR_SUCCESS;
}

#endif

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
   check(legacy.page_pauses);
   check(legacy.first_watch.disposition ==
         ct::FirstWatchDisposition::NotOffered);
   ct::mark_player_orientation_shown(legacy_path.string(), 17, 1);
   check(ct::resolve_player_identity(legacy_path.string(), 17, "Legacy", 42)
            .orientation_shown);

   const auto version_two_path = scratch.path / "version-two.bin";
   write_version_two_registry(version_two_path);
   const auto version_two =
      ct::resolve_player_identity(version_two_path.string(), 17, "V2 User!", 43);
   check(version_two.help_level == ct::HelpLevel::Expert);
   check(version_two.orientation_shown);
   check(version_two.page_pauses);
   check(!version_two.first_watch_preferences_recovered);

   const auto version_three_path = scratch.path / "version-three.bin";
   write_version_three_registry(version_three_path);
   const auto version_three =
      ct::resolve_player_identity(version_three_path.string(), 17, "V3 User", 44);
   check(!version_three.page_pauses);
   check(version_three.first_watch.disposition ==
         ct::FirstWatchDisposition::NotOffered);

   const auto path = (scratch.path / "players.bin").string();
   ct::create_player_identity_registry(path, 17);
#ifdef _WIN32
   const auto expected_owner = current_user_sid();
   check(same_sid(
      file_owner_sid(std::filesystem::path(path)), expected_owner));
#else
   struct stat status {};
   check(stat(path.c_str(), &status) == 0);
   check((status.st_mode & (S_IRWXG | S_IRWXO)) == 0);
#endif

   const auto first = ct::resolve_player_identity(path, 17, "Jane Doe", 42);
   check(first.player_id == 1);
   check(first.help_level == ct::HelpLevel::Beginner);
   check(!first.orientation_shown);
   check(first.page_pauses);
   check(first.first_watch.presentation_version ==
         ct::FIRST_WATCH_PRESENTATION_VERSION);
   check(first.first_watch.seen == 0);
#ifdef _WIN32
   std::vector<uint8_t> administrator_owner;
   if(set_administrators_owner(
         std::filesystem::path(path), administrator_owner)) {
      check(same_sid(
         file_owner_sid(std::filesystem::path(path)), administrator_owner));
      ct::set_player_help_level(
         path, 17, first.player_id, ct::HelpLevel::Expert);
      check(same_sid(
         file_owner_sid(std::filesystem::path(path)), administrator_owner));
   }
#endif
   check(ct::resolve_player_identity(path, 17, "Jane Doe", 42).player_id ==
         first.player_id);
   check(throws([&] {
      (void)ct::resolve_player_identity(path, 17, "Jane Smith", 42);
   }));
   check(throws([&] {
      (void)ct::resolve_player_identity(path, 17, "Jane Doe", 43);
   }));

   ct::set_player_help_level(path, 17, first.player_id, ct::HelpLevel::Expert);
   ct::set_player_page_pauses(path, 17, first.player_id, false);
   ct::mark_player_orientation_shown(path, 17, first.player_id);
   auto updated = ct::resolve_player_identity(path, 17, "Jane Doe", 42);
   check(updated.help_level == ct::HelpLevel::Expert);
   check(updated.orientation_shown);
   check(!updated.page_pauses);
   ct::set_player_first_watch_state(path, 17, first.player_id,
      ct::FirstWatchPreferenceState{
         .disposition = ct::FirstWatchDisposition::Active,
         .presentation_version = ct::FIRST_WATCH_PRESENTATION_VERSION,
         .seen = 0x41,
      });
   updated = ct::resolve_player_identity(path, 17, "Jane Doe", 42);
   check(updated.first_watch.disposition == ct::FirstWatchDisposition::Active);
   check(updated.first_watch.seen == 0x41);
   ct::set_player_first_watch_state(path, 17, first.player_id,
      ct::FirstWatchPreferenceState{
         .disposition = ct::FirstWatchDisposition::Active,
         .presentation_version = ct::FIRST_WATCH_PRESENTATION_VERSION,
         .seen = 0x80000000u,
      });
   ct::set_player_first_watch_state(path, 17, first.player_id,
      ct::FirstWatchPreferenceState{
         .disposition = ct::FirstWatchDisposition::Hidden,
         .presentation_version = ct::FIRST_WATCH_PRESENTATION_VERSION,
         .seen = 1,
      });
   updated = ct::resolve_player_identity(path, 17, "Jane Doe", 42);
   check(updated.first_watch.disposition == ct::FirstWatchDisposition::Hidden);
   check(updated.first_watch.seen == 0x80000001u);

   corrupt_first_watch_disposition(path);
   updated = ct::resolve_player_identity(path, 17, "Jane Doe", 42);
   check(updated.first_watch.disposition ==
         ct::FirstWatchDisposition::NotOffered);
   check(updated.first_watch.seen == 0);
   check(updated.first_watch_preferences_recovered);
   ct::rename_player_identity(path, 17, first.player_id, "Jane Smith");
   ct::reindex_player_identity(path, 17, first.player_id, 43);
   check(ct::resolve_player_identity(path, 17, "Jane Smith", 43).player_id ==
         first.player_id);
   ct::retire_player_identity(path, 17, first.player_id);
   const auto replacement =
      ct::resolve_player_identity(path, 17, "Jane Smith", 43);
   check(replacement.player_id == 2);
   check(replacement.page_pauses);

   const auto entries = ct::list_player_identities(path, 17);
   check(entries.size() == 2);
   check(entries[0].retired);
   check(!entries[1].retired);
   check(throws([&] {
      (void)ct::list_player_identities(path, 18);
   }));
}
