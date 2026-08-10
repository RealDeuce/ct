#include "ct/player_identity_registry.hpp"

#include <botan/hash.h>

#include <algorithm>
#include <array>
#include <cerrno>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <limits>
#include <span>
#include <stdexcept>
#include <string>
#include <vector>

#ifdef _WIN32
#include <windows.h>
#include <sddl.h>
#else
#include <fcntl.h>
#include <sys/file.h>
#include <sys/stat.h>
#include <unistd.h>
#endif

namespace ct {
namespace {

constexpr std::array<uint8_t, 8> MAGIC = {'C', 'T', 'I', 'D', 'M', 'A', 'P', 0};
constexpr uint16_t FORMAT_VERSION = 2;
constexpr size_t DIGEST_BYTES = 32;
constexpr size_t MAX_NAME_BYTES = 255;
constexpr size_t MAX_RECORDS = 1'000'000;

struct Registry {
   uint32_t bbs_id;
   uint32_t next_id = 1;
   std::vector<LocalPlayerIdentity> entries;
};

void put_u16(std::vector<uint8_t>& out, const uint16_t value) {
   out.push_back(static_cast<uint8_t>(value));
   out.push_back(static_cast<uint8_t>(value >> 8));
}

void put_u32(std::vector<uint8_t>& out, const uint32_t value) {
   for(unsigned shift = 0; shift != 32; shift += 8) {
      out.push_back(static_cast<uint8_t>(value >> shift));
   }
}

uint16_t get_u16(std::span<const uint8_t> bytes, size_t& offset) {
   if(offset + 2 > bytes.size()) {
      throw std::runtime_error("truncated player identity registry");
   }
   const auto value = static_cast<uint16_t>(bytes[offset]) |
                      (static_cast<uint16_t>(bytes[offset + 1]) << 8);
   offset += 2;
   return value;
}

uint32_t get_u32(std::span<const uint8_t> bytes, size_t& offset) {
   if(offset + 4 > bytes.size()) {
      throw std::runtime_error("truncated player identity registry");
   }
   uint32_t value = 0;
   for(unsigned shift = 0; shift != 32; shift += 8) {
      value |= static_cast<uint32_t>(bytes[offset++]) << shift;
   }
   return value;
}

std::vector<uint8_t> encode(const Registry& registry) {
   std::vector<uint8_t> bytes(MAGIC.begin(), MAGIC.end());
   put_u16(bytes, FORMAT_VERSION);
   put_u16(bytes, 0);
   put_u32(bytes, registry.bbs_id);
   put_u32(bytes, registry.next_id);
   put_u32(bytes, static_cast<uint32_t>(registry.entries.size()));
   for(const auto& entry : registry.entries) {
      bytes.push_back(entry.retired ? 1 : 0);
      bytes.push_back(entry.record_index.has_value() ? 1 : 0);
      bytes.push_back(entry.help_level == HelpLevel::Expert ? 1 : 0);
      bytes.push_back(entry.orientation_shown ? 1 : 0);
      put_u16(bytes, static_cast<uint16_t>(entry.name.size()));
      put_u32(bytes, entry.player_id);
      put_u32(bytes, entry.record_index.value_or(0));
      bytes.insert(bytes.end(), entry.name.begin(), entry.name.end());
   }
   auto hash = Botan::HashFunction::create_or_throw("SHA-256");
   hash->update(bytes);
   const auto digest = hash->final();
   bytes.insert(bytes.end(), digest.begin(), digest.end());
   return bytes;
}

Registry decode(const std::vector<uint8_t>& bytes, const uint32_t bbs_id) {
   if(bytes.size() < 24 + DIGEST_BYTES) {
      throw std::runtime_error("player identity registry is truncated");
   }
   auto hash = Botan::HashFunction::create_or_throw("SHA-256");
   hash->update(bytes.data(), bytes.size() - DIGEST_BYTES);
   const auto digest = hash->final();
   if(!std::equal(digest.begin(), digest.end(), bytes.end() - DIGEST_BYTES)) {
      throw std::runtime_error("player identity registry checksum is invalid");
   }
   const std::span<const uint8_t> payload(bytes.data(),
                                          bytes.size() - DIGEST_BYTES);
   if(!std::equal(MAGIC.begin(), MAGIC.end(), payload.begin())) {
      throw std::runtime_error("player identity registry has invalid magic");
   }
   size_t offset = MAGIC.size();
   const auto version = get_u16(payload, offset);
   if((version != 1 && version != FORMAT_VERSION) || get_u16(payload, offset) != 0) {
      throw std::runtime_error("unsupported player identity registry format");
   }
   Registry result{
      .bbs_id = get_u32(payload, offset),
      .next_id = 1,
      .entries = {},
   };
   result.next_id = get_u32(payload, offset);
   const auto count = get_u32(payload, offset);
   if(result.bbs_id != bbs_id) {
      throw std::runtime_error("player identity registry belongs to another BBS");
   }
   if(result.next_id == 0 || count > MAX_RECORDS) {
      throw std::runtime_error("player identity registry header is invalid");
   }
   result.entries.reserve(count);
   for(uint32_t index = 0; index < count; ++index) {
      if(offset + (version == 1 ? 12 : 14) > payload.size()) {
         throw std::runtime_error("truncated player identity registry entry");
      }
      const bool retired = payload[offset++] != 0;
      const bool has_index = payload[offset++] != 0;
      HelpLevel help_level = HelpLevel::Beginner;
      bool orientation_shown = false;
      if(version >= 2) {
         const auto encoded_help_level = payload[offset++];
         const auto encoded_orientation = payload[offset++];
         if(encoded_help_level > 1 || encoded_orientation > 1) {
            throw std::runtime_error("player identity registry entry is invalid");
         }
         help_level = encoded_help_level == 0
            ? HelpLevel::Beginner
            : HelpLevel::Expert;
         orientation_shown = encoded_orientation != 0;
      }
      const auto name_size = get_u16(payload, offset);
      const auto player_id = get_u32(payload, offset);
      const auto record_index = get_u32(payload, offset);
      if(player_id == 0 || name_size == 0 || name_size > MAX_NAME_BYTES ||
         offset + name_size > payload.size()) {
         throw std::runtime_error("player identity registry entry is invalid");
      }
      result.entries.push_back(LocalPlayerIdentity{
         .player_id = player_id,
         .name = std::string(reinterpret_cast<const char*>(payload.data() + offset),
                             name_size),
         .record_index = has_index ? std::optional(record_index) : std::nullopt,
         .retired = retired,
         .help_level = help_level,
         .orientation_shown = orientation_shown,
      });
      offset += name_size;
   }
   if(offset != payload.size()) {
      throw std::runtime_error("player identity registry has trailing data");
   }
   return result;
}

void validate_name(const std::string& name) {
   if(name.empty() || name.size() > MAX_NAME_BYTES ||
      name.find('\0') != std::string::npos) {
      throw std::invalid_argument("BBS user name must contain 1..255 bytes");
   }
}

#ifndef _WIN32

class LockedFile final {
public:
   LockedFile(const std::string& path, const bool create)
      : path_(path), descriptor_(open((path + ".lock").c_str(),
            O_RDWR | O_CLOEXEC | O_NOFOLLOW | O_CREAT,
            S_IRUSR | S_IWUSR)) {
      if(descriptor_ < 0) {
         throw std::runtime_error("cannot open player identity registry lock '" + path +
                                  "': " + std::strerror(errno));
      }
      struct stat status {};
      if(fstat(descriptor_, &status) != 0 || !S_ISREG(status.st_mode) ||
         status.st_nlink != 1 || status.st_uid != geteuid() ||
         (status.st_mode & (S_IRWXG | S_IRWXO)) != 0) {
         close(descriptor_);
         descriptor_ = -1;
         throw std::runtime_error(
            "player identity registry must be an owner-only regular file");
      }
      if(flock(descriptor_, LOCK_EX) != 0) {
         close(descriptor_);
         descriptor_ = -1;
         throw std::runtime_error("cannot lock player identity registry");
      }
      struct stat target {};
      const bool exists = lstat(path_.c_str(), &target) == 0;
      if((create && exists) || (!create && !exists)) {
         throw std::runtime_error(
            create ? "player identity registry already exists"
                   : "player identity registry does not exist");
      }
      if(!create && (!S_ISREG(target.st_mode) || target.st_nlink != 1 ||
         target.st_uid != geteuid() ||
         (target.st_mode & (S_IRWXG | S_IRWXO)) != 0)) {
         throw std::runtime_error(
            "player identity registry must be an owner-only regular file");
      }
   }

   ~LockedFile() {
      if(descriptor_ >= 0) {
         close(descriptor_);
      }
   }

   std::vector<uint8_t> read_all() const {
      const int input = open(path_.c_str(), O_RDONLY | O_CLOEXEC | O_NOFOLLOW);
      if(input < 0) {
         throw std::runtime_error("cannot open player identity registry");
      }
      struct stat status {};
      if(fstat(input, &status) != 0 || status.st_size < 0 ||
         status.st_size > 128 * 1024 * 1024) {
         close(input);
         throw std::runtime_error("invalid player identity registry size");
      }
      std::vector<uint8_t> bytes(static_cast<size_t>(status.st_size));
      size_t offset = 0;
      while(offset < bytes.size()) {
         const auto count = read(input, bytes.data() + offset,
                                 bytes.size() - offset);
         if(count < 0 && errno == EINTR) {
            continue;
         }
         if(count <= 0) {
            close(input);
            throw std::runtime_error("cannot read player identity registry");
         }
         offset += static_cast<size_t>(count);
      }
      close(input);
      return bytes;
   }

   void replace(const std::vector<uint8_t>& bytes) const {
      const auto temporary = path_ + ".new";
      const int output = open(temporary.c_str(),
                              O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW,
                              S_IRUSR | S_IWUSR);
      if(output < 0) {
         throw std::runtime_error("cannot create temporary identity registry");
      }
      bool success = false;
      size_t offset = 0;
      while(offset < bytes.size()) {
         const auto count = write(output, bytes.data() + offset, bytes.size() - offset);
         if(count < 0 && errno == EINTR) {
            continue;
         }
         if(count <= 0) {
            break;
         }
         offset += static_cast<size_t>(count);
      }
      success = offset == bytes.size() && fsync(output) == 0;
      const auto failure = errno;
      close(output);
      if(!success || rename(temporary.c_str(), path_.c_str()) != 0) {
         unlink(temporary.c_str());
         errno = failure;
         throw std::runtime_error("cannot atomically replace identity registry");
      }
      const auto directory = std::filesystem::path(path_).parent_path();
      const int directory_fd = open(directory.empty() ? "." : directory.c_str(),
                                    O_RDONLY | O_CLOEXEC);
      if(directory_fd >= 0) {
         (void)fsync(directory_fd);
         close(directory_fd);
      }
   }

private:
   std::string path_;
   int descriptor_;
};

#else

std::runtime_error windows_error(const std::string& operation) {
   return std::runtime_error(
      operation + " failed with Windows error " + std::to_string(GetLastError()));
}

std::wstring wide_path(const std::string& path) {
   const int count = MultiByteToWideChar(
      CP_UTF8, MB_ERR_INVALID_CHARS, path.data(),
      static_cast<int>(path.size()), nullptr, 0);
   if(count <= 0) {
      throw windows_error("identity registry path conversion");
   }
   std::wstring result(static_cast<size_t>(count), L'\0');
   if(MultiByteToWideChar(
         CP_UTF8, MB_ERR_INVALID_CHARS, path.data(),
         static_cast<int>(path.size()), result.data(), count) != count) {
      throw windows_error("identity registry path conversion");
   }
   return result;
}

class LocalMemory final {
public:
   explicit LocalMemory(PSECURITY_DESCRIPTOR descriptor)
      : descriptor_(descriptor) {}
   ~LocalMemory() {
      if(descriptor_) {
         LocalFree(descriptor_);
      }
   }
   LocalMemory(const LocalMemory&) = delete;
   LocalMemory& operator=(const LocalMemory&) = delete;
   PSECURITY_DESCRIPTOR get() const {
      return descriptor_;
   }

private:
   PSECURITY_DESCRIPTOR descriptor_;
};

LocalMemory secure_descriptor() {
   PSECURITY_DESCRIPTOR descriptor = nullptr;
   if(ConvertStringSecurityDescriptorToSecurityDescriptorW(
         L"D:P(A;;FA;;;OW)(A;;FA;;;SY)(A;;FA;;;BA)",
         SDDL_REVISION_1, &descriptor, nullptr) == 0) {
      throw windows_error("identity registry security descriptor creation");
   }
   return LocalMemory(descriptor);
}

class Handle final {
public:
   explicit Handle(HANDLE handle) : handle_(handle) {}
   ~Handle() {
      if(handle_ != INVALID_HANDLE_VALUE) {
         CloseHandle(handle_);
      }
   }
   Handle(const Handle&) = delete;
   Handle& operator=(const Handle&) = delete;
   HANDLE get() const {
      return handle_;
   }

private:
   HANDLE handle_;
};

void require_regular_handle(const HANDLE handle, const char* description) {
   BY_HANDLE_FILE_INFORMATION information{};
   if(GetFileInformationByHandle(handle, &information) == 0) {
      throw windows_error(std::string(description) + " information");
   }
   if((information.dwFileAttributes &
       (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)) != 0) {
      throw std::runtime_error(
         std::string(description) + " must be a regular non-reparse file");
   }
}

void protect_path(const std::wstring& path) {
   auto descriptor = secure_descriptor();
   if(SetFileSecurityW(
         path.c_str(),
         DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
         descriptor.get()) == 0) {
      throw windows_error("identity registry DACL protection");
   }
}

class LockedFile final {
public:
   LockedFile(const std::string& path, const bool create)
      : path_(wide_path(path)), lock_path_(wide_path(path + ".lock")) {
      auto descriptor = secure_descriptor();
      SECURITY_ATTRIBUTES attributes{
         .nLength = sizeof(SECURITY_ATTRIBUTES),
         .lpSecurityDescriptor = descriptor.get(),
         .bInheritHandle = FALSE,
      };
      handle_ = CreateFileW(
         lock_path_.c_str(), GENERIC_READ | GENERIC_WRITE, 0, &attributes,
         OPEN_ALWAYS,
         FILE_ATTRIBUTE_HIDDEN | FILE_FLAG_OPEN_REPARSE_POINT, nullptr);
      if(handle_ == INVALID_HANDLE_VALUE) {
         throw windows_error("identity registry lock open");
      }
      require_regular_handle(handle_, "identity registry lock");
      protect_path(lock_path_);

      const auto attributes_value = GetFileAttributesW(path_.c_str());
      const bool exists = attributes_value != INVALID_FILE_ATTRIBUTES;
      if(!exists && GetLastError() != ERROR_FILE_NOT_FOUND &&
         GetLastError() != ERROR_PATH_NOT_FOUND) {
         throw windows_error("identity registry inspection");
      }
      if((create && exists) || (!create && !exists)) {
         throw std::runtime_error(
            create ? "player identity registry already exists"
                   : "player identity registry does not exist");
      }
   }
   ~LockedFile() {
      if(handle_ != INVALID_HANDLE_VALUE) {
         CloseHandle(handle_);
      }
   }
   std::vector<uint8_t> read_all() const {
      Handle input(CreateFileW(
         path_.c_str(), GENERIC_READ, 0, nullptr, OPEN_EXISTING,
         FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT, nullptr));
      if(input.get() == INVALID_HANDLE_VALUE) {
         throw windows_error("identity registry open");
      }
      require_regular_handle(input.get(), "identity registry");
      protect_path(path_);
      LARGE_INTEGER size {};
      if(!GetFileSizeEx(input.get(), &size) || size.QuadPart < 0 ||
         size.QuadPart > 128 * 1024 * 1024) {
         throw std::runtime_error("invalid player identity registry size");
      }
      std::vector<uint8_t> bytes(static_cast<size_t>(size.QuadPart));
      DWORD read = 0;
      if(!bytes.empty() && (!ReadFile(input.get(), bytes.data(),
             static_cast<DWORD>(bytes.size()), &read, nullptr) || read != bytes.size())) {
         throw std::runtime_error("cannot read player identity registry");
      }
      return bytes;
   }
   void replace(const std::vector<uint8_t>& bytes) const {
      const auto temporary = path_ + L".new";
      auto descriptor = secure_descriptor();
      SECURITY_ATTRIBUTES attributes{
         .nLength = sizeof(SECURITY_ATTRIBUTES),
         .lpSecurityDescriptor = descriptor.get(),
         .bInheritHandle = FALSE,
      };
      bool okay = false;
      {
         Handle output(CreateFileW(
            temporary.c_str(), GENERIC_WRITE, 0, &attributes, CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT, nullptr));
         if(output.get() == INVALID_HANDLE_VALUE) {
            throw windows_error("temporary identity registry creation");
         }
         DWORD written = 0;
         okay = WriteFile(output.get(), bytes.data(),
                          static_cast<DWORD>(bytes.size()), &written, nullptr) &&
                written == bytes.size() && FlushFileBuffers(output.get());
      }
      if(!okay || !MoveFileExW(temporary.c_str(), path_.c_str(),
                               MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)) {
         DeleteFileW(temporary.c_str());
         throw windows_error("atomic identity registry replacement");
      }
   }
private:
   std::wstring path_;
   std::wstring lock_path_;
   HANDLE handle_ = INVALID_HANDLE_VALUE;
};

#endif

Registry load(LockedFile& file, const uint32_t bbs_id) {
   return decode(file.read_all(), bbs_id);
}

LocalPlayerIdentity& active_by_id(Registry& registry, const uint32_t player_id) {
   const auto found = std::find_if(registry.entries.begin(), registry.entries.end(),
      [player_id](const auto& entry) {
         return entry.player_id == player_id && !entry.retired;
      });
   if(found == registry.entries.end()) {
      throw std::invalid_argument("active local player ID was not found");
   }
   return *found;
}

void ensure_no_partial_collision(const Registry& registry,
                                 const LocalPlayerIdentity& changed) {
   for(const auto& entry : registry.entries) {
      if(entry.retired || entry.player_id == changed.player_id) {
         continue;
      }
      if(entry.name == changed.name ||
         (entry.record_index && changed.record_index &&
          entry.record_index == changed.record_index)) {
         throw std::invalid_argument(
            "identity conflicts with another active BBS account");
      }
   }
}

}  // namespace

void create_player_identity_registry(const std::string& path,
                                     const uint32_t bbs_id) {
   if(path.empty() || bbs_id == 0) {
      throw std::invalid_argument("identity registry path and BBS ID are required");
   }
   LockedFile file(path, true);
   file.replace(encode(Registry{
      .bbs_id = bbs_id,
      .next_id = 1,
      .entries = {},
   }));
}

LocalPlayerIdentity resolve_player_identity(
   const std::string& path,
   const uint32_t bbs_id,
   const std::string& name,
   const std::optional<uint32_t> record_index) {
   validate_name(name);
   LockedFile file(path, false);
   auto registry = load(file, bbs_id);
   for(const auto& entry : registry.entries) {
      if(!entry.retired && entry.name == name &&
         entry.record_index == record_index) {
         return entry;
      }
   }
   for(const auto& entry : registry.entries) {
      if(entry.retired) {
         continue;
      }
      const bool same_name = entry.name == name;
      const bool same_index = entry.record_index && record_index &&
                              entry.record_index == record_index;
      if(same_name || same_index) {
         throw std::runtime_error(
            "BBS account identity changed; the sysop must rename, reindex, or "
            "retire its local mapping before this player can enter");
      }
   }
   if(registry.next_id == std::numeric_limits<uint32_t>::max()) {
      throw std::runtime_error("local player ID space is exhausted");
   }
   const auto player_id = registry.next_id++;
   registry.entries.push_back(LocalPlayerIdentity{
      .player_id = player_id,
      .name = name,
      .record_index = record_index,
      .retired = false,
      .help_level = HelpLevel::Beginner,
      .orientation_shown = false,
   });
   const auto created = registry.entries.back();
   file.replace(encode(registry));
   return created;
}

std::vector<LocalPlayerIdentity> list_player_identities(
   const std::string& path,
   const uint32_t bbs_id) {
   LockedFile file(path, false);
   return load(file, bbs_id).entries;
}

void rename_player_identity(const std::string& path,
                            const uint32_t bbs_id,
                            const uint32_t player_id,
                            const std::string& new_name) {
   validate_name(new_name);
   LockedFile file(path, false);
   auto registry = load(file, bbs_id);
   auto& entry = active_by_id(registry, player_id);
   entry.name = new_name;
   ensure_no_partial_collision(registry, entry);
   file.replace(encode(registry));
}

void reindex_player_identity(const std::string& path,
                             const uint32_t bbs_id,
                             const uint32_t player_id,
                             const std::optional<uint32_t> new_record_index) {
   LockedFile file(path, false);
   auto registry = load(file, bbs_id);
   auto& entry = active_by_id(registry, player_id);
   entry.record_index = new_record_index;
   ensure_no_partial_collision(registry, entry);
   file.replace(encode(registry));
}

void retire_player_identity(const std::string& path,
                            const uint32_t bbs_id,
                            const uint32_t player_id) {
   LockedFile file(path, false);
   auto registry = load(file, bbs_id);
   active_by_id(registry, player_id).retired = true;
   file.replace(encode(registry));
}

void set_player_help_level(const std::string& path,
                           const uint32_t bbs_id,
                           const uint32_t player_id,
                           const HelpLevel help_level) {
   LockedFile file(path, false);
   auto registry = load(file, bbs_id);
   active_by_id(registry, player_id).help_level = help_level;
   file.replace(encode(registry));
}

void mark_player_orientation_shown(const std::string& path,
                                   const uint32_t bbs_id,
                                   const uint32_t player_id) {
   LockedFile file(path, false);
   auto registry = load(file, bbs_id);
   active_by_id(registry, player_id).orientation_shown = true;
   file.replace(encode(registry));
}

}  // namespace ct
