#include "ct/bbs_config.hpp"

#include <charconv>
#include <chrono>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iterator>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

#ifdef _WIN32
#include <windows.h>
#else
#include <cerrno>
#include <cstring>
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>
#endif

namespace ct {
namespace {

constexpr std::string_view DEFAULT_SERVER = "localhost";
constexpr std::string_view DEFAULT_GAME_PORT = "7323";
constexpr std::string_view DEFAULT_SYSOP_PORT = "7325";
constexpr std::string_view DEFAULT_CREDENTIAL_FILE =
   "cepheus-trader.credential";
constexpr std::string_view DEFAULT_IDENTITY_FILE =
   "cepheus-trader.identities";

std::string_view trim(const std::string_view value) {
   const auto first = value.find_first_not_of(" \t\r");
   if(first == std::string_view::npos) {
      return {};
   }
   const auto last = value.find_last_not_of(" \t\r");
   return value.substr(first, last - first + 1);
}

void assign_once(std::optional<std::string>& destination,
                 const std::string_view key,
                 const std::string_view value,
                 const size_t line_number) {
   if(destination.has_value()) {
      throw std::runtime_error(
         "duplicate configuration key '" + std::string(key) +
         "' on line " + std::to_string(line_number));
   }
   if(value.empty()) {
      throw std::runtime_error(
         "empty configuration value for '" + std::string(key) +
         "' on line " + std::to_string(line_number));
   }
   destination = value;
}

std::string require_value(const std::optional<std::string>& value,
                          const std::string_view key) {
   if(!value.has_value()) {
      throw std::runtime_error(
         "BBS configuration is missing '" + std::string(key) + "'");
   }
   return *value;
}

std::string validate_port(const std::string& text, const std::string_view key) {
   uint32_t port = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), port);
   if(error != std::errc() || end != text.data() + text.size() ||
      port == 0 || port > 65535) {
      throw std::runtime_error(
         "BBS configuration '" + std::string(key) +
         "' must be an integer in 1..65535");
   }
   return text;
}

unsigned int validate_dimension(const std::string& text,
                                const std::string_view key,
                                const unsigned int minimum) {
   unsigned int value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size() ||
      (value != 0 && (value < minimum || value > 255))) {
      throw std::runtime_error(
         "BBS configuration '" + std::string(key) + "' must be 0 or in " +
         std::to_string(minimum) + "..255");
   }
   return value;
}

unsigned int validate_inactivity_timeout(const std::string& text) {
   unsigned int value = 0;
   const auto [end, error] =
      std::from_chars(text.data(), text.data() + text.size(), value);
   if(error != std::errc() || end != text.data() + text.size() ||
      value > MAX_INACTIVITY_TIMEOUT_SECONDS) {
      throw std::runtime_error(
         "BBS configuration 'inactivity-timeout-seconds' must be in 0.." +
         std::to_string(MAX_INACTIVITY_TIMEOUT_SECONDS));
   }
   return value;
}

void create_private_directories(const std::filesystem::path& path) {
   if(path.empty()) {
      return;
   }
   std::vector<std::filesystem::path> missing;
   auto cursor = std::filesystem::absolute(path).lexically_normal();
   std::error_code error;
   while(!std::filesystem::exists(cursor, error)) {
      if(error) {
         throw std::runtime_error(
            "cannot inspect installation directory '" + cursor.string() +
            "': " + error.message());
      }
      missing.push_back(cursor);
      const auto parent = cursor.parent_path();
      if(parent == cursor || parent.empty()) {
         throw std::runtime_error(
            "cannot find an existing parent for installation directory '" +
            path.string() + "'");
      }
      cursor = parent;
   }
   if(error) {
      throw std::runtime_error(
         "cannot inspect installation directory '" + cursor.string() +
         "': " + error.message());
   }
   if(!std::filesystem::is_directory(cursor, error) || error) {
      throw std::runtime_error(
         "installation path parent is not a directory: " + cursor.string());
   }
   if(missing.empty()) {
      return;
   }
   if(!std::filesystem::create_directories(path, error) || error) {
      throw std::runtime_error(
         "cannot create installation directory '" + path.string() +
         "': " + error.message());
   }
#ifndef _WIN32
   for(const auto& directory : missing) {
      std::filesystem::permissions(
         directory,
         std::filesystem::perms::owner_all,
         std::filesystem::perm_options::replace,
         error);
      if(error) {
         throw std::runtime_error(
            "cannot protect installation directory '" + directory.string() +
            "': " + error.message());
      }
   }
#endif
}

#ifdef _WIN32

std::runtime_error system_error(const std::string& operation) {
   return std::runtime_error(
      operation + " failed with Windows error " + std::to_string(GetLastError()));
}

std::wstring wide_path(const std::string& path) {
   if(path.empty()) {
      throw std::invalid_argument("BBS configuration path must not be empty");
   }
   const int count =
      MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path.data(),
                          static_cast<int>(path.size()), nullptr, 0);
   if(count <= 0) {
      throw system_error("UTF-8 path conversion");
   }
   std::wstring result(static_cast<size_t>(count), L'\0');
   if(MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path.data(),
                          static_cast<int>(path.size()), result.data(), count) != count) {
      throw system_error("UTF-8 path conversion");
   }
   return result;
}

void create_exclusive_file(const std::string& path,
                           const std::string_view content) {
   const auto path_wide = wide_path(path);
   const auto file =
      CreateFileW(path_wide.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_NEW,
                  FILE_ATTRIBUTE_NORMAL, nullptr);
   if(file == INVALID_HANDLE_VALUE) {
      throw system_error("exclusive BBS configuration file creation");
   }
   DWORD written = 0;
   const bool success =
      WriteFile(file, content.data(), static_cast<DWORD>(content.size()),
                &written, nullptr) != 0 &&
      written == content.size() &&
      FlushFileBuffers(file) != 0;
   const auto failure = success ? ERROR_SUCCESS : GetLastError();
   CloseHandle(file);
   if(!success) {
      DeleteFileW(path_wide.c_str());
      SetLastError(failure);
      throw system_error("BBS configuration file write");
   }
}

void replace_file(const std::string& path, const std::string_view content) {
   const auto path_wide = wide_path(path);
   const auto suffix =
      L".tmp-" + std::to_wstring(GetCurrentProcessId()) + L"-" +
      std::to_wstring(
         std::chrono::steady_clock::now().time_since_epoch().count());
   const auto temporary = path_wide + suffix;
   const auto file =
      CreateFileW(temporary.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_NEW,
                  FILE_ATTRIBUTE_NORMAL, nullptr);
   if(file == INVALID_HANDLE_VALUE) {
      throw system_error("temporary BBS configuration file creation");
   }
   DWORD written = 0;
   const bool success =
      WriteFile(file, content.data(), static_cast<DWORD>(content.size()),
                &written, nullptr) != 0 &&
      written == content.size() && FlushFileBuffers(file) != 0;
   const auto failure = success ? ERROR_SUCCESS : GetLastError();
   CloseHandle(file);
   if(!success) {
      DeleteFileW(temporary.c_str());
      SetLastError(failure);
      throw system_error("BBS configuration file write");
   }
   if(ReplaceFileW(path_wide.c_str(), temporary.c_str(), nullptr,
                   REPLACEFILE_WRITE_THROUGH, nullptr, nullptr) == 0) {
      const auto move_failure = GetLastError();
      DeleteFileW(temporary.c_str());
      SetLastError(move_failure);
      throw system_error("BBS configuration file replacement");
   }
}

#else

std::runtime_error system_error(const std::string& operation) {
   return std::runtime_error(operation + " failed: " + std::strerror(errno));
}

class FileDescriptor final {
public:
   explicit FileDescriptor(const int descriptor) : descriptor_(descriptor) {}
   ~FileDescriptor() {
      if(descriptor_ >= 0) {
         close(descriptor_);
      }
   }
   FileDescriptor(const FileDescriptor&) = delete;
   FileDescriptor& operator=(const FileDescriptor&) = delete;
   int get() const {
      return descriptor_;
   }

private:
   int descriptor_;
};

void create_exclusive_file(const std::string& path,
                           const std::string_view content) {
   FileDescriptor file(open(path.c_str(),
                            O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW,
                            S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH));
   if(file.get() < 0) {
      throw system_error("exclusive BBS configuration file creation");
   }
   size_t offset = 0;
   while(offset < content.size()) {
      const auto count =
         write(file.get(), content.data() + offset, content.size() - offset);
      if(count < 0 && errno == EINTR) {
         continue;
      }
      if(count <= 0) {
         const auto failure = errno;
         unlink(path.c_str());
         errno = failure;
         throw system_error("BBS configuration file write");
      }
      offset += static_cast<size_t>(count);
   }
   if(fsync(file.get()) != 0) {
      const auto failure = errno;
      unlink(path.c_str());
      errno = failure;
      throw system_error("BBS configuration file flush");
   }
}

void replace_file(const std::string& path, const std::string_view content) {
   struct stat existing {};
   if(lstat(path.c_str(), &existing) != 0) {
      throw system_error("BBS configuration file inspection");
   }
   if(!S_ISREG(existing.st_mode)) {
      throw std::runtime_error("BBS configuration path is not a regular file");
   }
   const auto temporary =
      path + ".tmp-" + std::to_string(getpid()) + "-" +
      std::to_string(
         std::chrono::steady_clock::now().time_since_epoch().count());
   FileDescriptor file(open(temporary.c_str(),
                            O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW,
                            existing.st_mode & 0777));
   if(file.get() < 0) {
      throw system_error("temporary BBS configuration file creation");
   }
   if(fchmod(file.get(), existing.st_mode & 07777) != 0) {
      const auto failure = errno;
      unlink(temporary.c_str());
      errno = failure;
      throw system_error("temporary BBS configuration file permissions");
   }
   size_t offset = 0;
   while(offset < content.size()) {
      const auto count =
         write(file.get(), content.data() + offset, content.size() - offset);
      if(count < 0 && errno == EINTR) {
         continue;
      }
      if(count <= 0) {
         const auto failure = errno;
         unlink(temporary.c_str());
         errno = failure;
         throw system_error("BBS configuration file write");
      }
      offset += static_cast<size_t>(count);
   }
   if(fsync(file.get()) != 0) {
      const auto failure = errno;
      unlink(temporary.c_str());
      errno = failure;
      throw system_error("BBS configuration file flush");
   }
   if(rename(temporary.c_str(), path.c_str()) != 0) {
      const auto failure = errno;
      unlink(temporary.c_str());
      errno = failure;
      throw system_error("BBS configuration file replacement");
   }
}

#endif

}  // namespace

BbsConfig read_bbs_config(const std::string& path) {
   std::ifstream input(path);
   if(!input) {
      throw std::runtime_error("cannot open BBS configuration file: " + path);
   }

   std::optional<std::string> server;
   std::optional<std::string> game_port;
   std::optional<std::string> sysop_port;
   std::optional<std::string> credential_path;
   std::optional<std::string> identity_registry_path;
   std::optional<std::string> identity_name_field;
   std::optional<std::string> terminal_profile;
   std::optional<std::string> terminal_columns;
   std::optional<std::string> terminal_rows;
   std::optional<std::string> inactivity_timeout_seconds;
   size_t line_number = 0;
   size_t total_bytes = 0;
   std::string line;
   while(std::getline(input, line)) {
      ++line_number;
      total_bytes += line.size() + 1;
      if(total_bytes > 64 * 1024) {
         throw std::runtime_error("BBS configuration file is too large");
      }
      const auto content = trim(line);
      if(content.empty() || content.front() == '#') {
         continue;
      }
      const auto separator = content.find('=');
      if(separator == std::string_view::npos) {
         throw std::runtime_error(
            "expected key=value on configuration line " +
            std::to_string(line_number));
      }
      const auto key = trim(content.substr(0, separator));
      const auto value = trim(content.substr(separator + 1));
      if(key == "server") {
         assign_once(server, key, value, line_number);
      } else if(key == "game-port") {
         assign_once(game_port, key, value, line_number);
      } else if(key == "sysop-port") {
         assign_once(sysop_port, key, value, line_number);
      } else if(key == "credential-file") {
         assign_once(credential_path, key, value, line_number);
      } else if(key == "identity-file") {
         assign_once(identity_registry_path, key, value, line_number);
      } else if(key == "identity-name") {
         assign_once(identity_name_field, key, value, line_number);
      } else if(key == "terminal-profile") {
         assign_once(terminal_profile, key, value, line_number);
      } else if(key == "terminal-columns") {
         assign_once(terminal_columns, key, value, line_number);
      } else if(key == "terminal-rows") {
         assign_once(terminal_rows, key, value, line_number);
      } else if(key == "inactivity-timeout-seconds") {
         assign_once(inactivity_timeout_seconds, key, value, line_number);
      } else {
         throw std::runtime_error(
            "unknown configuration key '" + std::string(key) +
            "' on line " + std::to_string(line_number));
      }
   }
   if(input.bad()) {
      throw std::runtime_error("error reading BBS configuration file: " + path);
   }

   auto credential = std::filesystem::path(
      require_value(credential_path, "credential-file"));
   if(credential.is_relative()) {
      credential = std::filesystem::path(path).parent_path() / credential;
   }
   auto identity = std::filesystem::path(
      require_value(identity_registry_path, "identity-file"));
   if(identity.is_relative()) {
      identity = std::filesystem::path(path).parent_path() / identity;
   }
   const auto name_field_text =
      require_value(identity_name_field, "identity-name");
   IdentityNameField name_field;
   if(name_field_text == "real-name") {
      name_field = IdentityNameField::RealName;
   } else if(name_field_text == "handle") {
      name_field = IdentityNameField::Handle;
   } else {
      throw std::runtime_error(
         "BBS configuration 'identity-name' must be real-name or handle");
   }
   const auto profile = require_value(terminal_profile, "terminal-profile");
   if(profile != "auto" && profile != "iso646" &&
      profile != "iso646-color" && profile != "cp437-plain" &&
      profile != "cp437-color") {
      throw std::runtime_error(
         "BBS configuration 'terminal-profile' has an invalid value");
   }
   return BbsConfig{
      .server = require_value(server, "server"),
      .game_port =
         validate_port(require_value(game_port, "game-port"), "game-port"),
      .sysop_port =
         validate_port(require_value(sysop_port, "sysop-port"), "sysop-port"),
      .credential_path = credential.lexically_normal().string(),
      .identity_registry_path = identity.lexically_normal().string(),
      .identity_name_field = name_field,
      .terminal_profile = profile,
      .terminal_columns = validate_dimension(
         require_value(terminal_columns, "terminal-columns"),
         "terminal-columns", 40),
      .terminal_rows = validate_dimension(
         require_value(terminal_rows, "terminal-rows"), "terminal-rows", 24),
      .inactivity_timeout_seconds =
         inactivity_timeout_seconds.has_value()
            ? validate_inactivity_timeout(*inactivity_timeout_seconds)
            : DEFAULT_INACTIVITY_TIMEOUT_SECONDS,
   };
}

BbsConfig default_bbs_config(const std::string& path) {
   const auto credential =
      std::filesystem::path(path).parent_path() / DEFAULT_CREDENTIAL_FILE;
   const auto identity =
      std::filesystem::path(path).parent_path() / DEFAULT_IDENTITY_FILE;
   return BbsConfig{
      .server = std::string(DEFAULT_SERVER),
      .game_port = std::string(DEFAULT_GAME_PORT),
      .sysop_port = std::string(DEFAULT_SYSOP_PORT),
      .credential_path = credential.lexically_normal().string(),
      .identity_registry_path = identity.lexically_normal().string(),
      .identity_name_field = IdentityNameField::RealName,
      .terminal_profile = "auto",
      .terminal_columns = 0,
      .terminal_rows = 0,
      .inactivity_timeout_seconds = DEFAULT_INACTIVITY_TIMEOUT_SECONDS,
   };
}

void create_bbs_installation_directories(
   const std::string& config_path,
   const std::string& credential_path) {
   if(config_path.empty()) {
      throw std::invalid_argument("BBS configuration path must not be empty");
   }
   if(credential_path.empty()) {
      throw std::invalid_argument("credential path must not be empty");
   }
   create_private_directories(std::filesystem::path(config_path).parent_path());
   create_private_directories(
      std::filesystem::path(credential_path).parent_path());
}

void create_default_bbs_config_file(const std::string& path) {
   create_bbs_config_file(path, default_bbs_config(path).credential_path);
}

void create_bbs_config_file(const std::string& path,
                            const std::string& credential_path) {
   create_bbs_config_file(
      path,
      credential_path,
      std::string(DEFAULT_SERVER),
      std::string(DEFAULT_GAME_PORT),
      std::string(DEFAULT_SYSOP_PORT));
}

void create_bbs_config_file(const std::string& path,
                            const std::string& credential_path,
                            const std::string& server,
                            const std::string& game_port,
                            const std::string& sysop_port) {
   if(path.empty()) {
      throw std::invalid_argument("BBS configuration path must not be empty");
   }
   if(credential_path.empty()) {
      throw std::invalid_argument("credential path must not be empty");
   }
   if(server.empty() || server.find_first_of("\r\n") != std::string::npos) {
      throw std::invalid_argument("server must be a nonempty hostname or address");
   }
   const auto checked_game_port = validate_port(game_port, "game-port");
   const auto checked_sysop_port = validate_port(sysop_port, "sysop-port");
   create_bbs_installation_directories(path, credential_path);

   const auto config_directory =
      std::filesystem::absolute(std::filesystem::path(path))
         .lexically_normal()
         .parent_path();
   const auto credential =
      std::filesystem::absolute(std::filesystem::path(credential_path))
         .lexically_normal();
   const auto relative = credential.lexically_relative(config_directory);
   const auto reference = relative.empty() ? credential : relative;
   const auto reference_text = reference.generic_string();
   if(reference_text.find_first_of("\r\n") != std::string::npos) {
      throw std::invalid_argument(
         "credential path cannot be represented in the configuration file");
   }

   const std::string content =
      "# Shared by cepheus-trader-sysop and cepheus-trader-door.\n"
      "# The PSK is stored only in the separate protected credential file.\n"
      "server=" + server + "\n"
      "game-port=" + checked_game_port + "\n"
      "sysop-port=" + checked_sysop_port + "\n"
      "credential-file=" + reference_text + "\n"
      "identity-file=cepheus-trader.identities\n"
      "identity-name=real-name\n"
      "terminal-profile=auto\n"
      "terminal-columns=0\n"
      "terminal-rows=0\n"
      "inactivity-timeout-seconds=" +
      std::to_string(DEFAULT_INACTIVITY_TIMEOUT_SECONDS) + "\n";
   create_exclusive_file(path, content);
}

void set_bbs_inactivity_timeout(const std::string& path,
                                const unsigned int timeout_seconds) {
   if(timeout_seconds > MAX_INACTIVITY_TIMEOUT_SECONDS) {
      throw std::invalid_argument(
         "inactivity timeout must be in 0.." +
         std::to_string(MAX_INACTIVITY_TIMEOUT_SECONDS));
   }
   (void)read_bbs_config(path);

   std::ifstream input(path, std::ios::binary);
   if(!input) {
      throw std::runtime_error("cannot open BBS configuration file: " + path);
   }
   const std::string original{
      std::istreambuf_iterator<char>(input),
      std::istreambuf_iterator<char>()};
   if(input.bad()) {
      throw std::runtime_error("error reading BBS configuration file: " + path);
   }

   const std::string replacement =
      "inactivity-timeout-seconds=" + std::to_string(timeout_seconds);
   std::string updated;
   updated.reserve(original.size() + replacement.size() + 1);
   bool replaced = false;
   size_t offset = 0;
   while(offset < original.size()) {
      const auto newline = original.find('\n', offset);
      const auto end = newline == std::string::npos ? original.size() : newline;
      const auto length = end - offset;
      std::string_view line(original.data() + offset, length);
      const auto content = trim(line);
      const auto separator = content.find('=');
      if(separator != std::string_view::npos &&
         trim(content.substr(0, separator)) == "inactivity-timeout-seconds") {
         updated += replacement;
         if(!line.empty() && line.back() == '\r') {
            updated += '\r';
         }
         replaced = true;
      } else {
         updated.append(line);
      }
      if(newline != std::string::npos) {
         updated += '\n';
         offset = newline + 1;
      } else {
         offset = original.size();
      }
   }
   if(!replaced) {
      if(!updated.empty() && updated.back() != '\n') {
         updated += '\n';
      }
      updated += replacement;
      updated += '\n';
   }
   replace_file(path, updated);
}

}  // namespace ct
