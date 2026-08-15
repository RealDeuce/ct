#include "ct/bbs_credential.hpp"
#include "ct/crypto.hpp"

#include <array>
#include <algorithm>
#include <cerrno>
#include <cstring>
#include <stdexcept>
#include <string>

#ifdef _WIN32
#include <windows.h>
#include <sddl.h>
#else
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>
#endif

namespace ct {
namespace {

constexpr std::array<uint8_t, 8> MAGIC{
   'C', 'T', 'B', 'B', 'S', 'P', 'S', 'K',
};
constexpr uint16_t FORMAT_VERSION = 1;
constexpr size_t FILE_BYTES = 8 + 2 + 2 + 4 + BBS_PSK_BYTES;

std::runtime_error system_error(const std::string& operation) {
#ifdef _WIN32
   return std::runtime_error(
      operation + " failed with Windows error " + std::to_string(GetLastError()));
#else
   return std::runtime_error(operation + " failed: " + std::strerror(errno));
#endif
}

std::array<uint8_t, FILE_BYTES> encode(const uint32_t bbs_id,
                                       const std::span<const uint8_t> psk) {
   if(bbs_id == 0) {
      throw std::invalid_argument("BBS ID must be nonzero");
   }
   if(psk.size() != BBS_PSK_BYTES) {
      throw std::invalid_argument("BBS PSK must contain exactly 32 bytes");
   }
   std::array<uint8_t, FILE_BYTES> bytes{};
   std::copy(MAGIC.begin(), MAGIC.end(), bytes.begin());
   bytes[8] = static_cast<uint8_t>(FORMAT_VERSION >> 8);
   bytes[9] = static_cast<uint8_t>(FORMAT_VERSION);
   bytes[12] = static_cast<uint8_t>(bbs_id >> 24);
   bytes[13] = static_cast<uint8_t>(bbs_id >> 16);
   bytes[14] = static_cast<uint8_t>(bbs_id >> 8);
   bytes[15] = static_cast<uint8_t>(bbs_id);
   std::copy(psk.begin(), psk.end(), bytes.begin() + 16);
   return bytes;
}

BbsCredentialFile decode(std::array<uint8_t, FILE_BYTES>& bytes) {
   if(!std::equal(MAGIC.begin(), MAGIC.end(), bytes.begin())) {
      throw std::runtime_error("BBS credential file has an invalid magic value");
   }
   const auto version =
      static_cast<uint16_t>((static_cast<uint16_t>(bytes[8]) << 8) | bytes[9]);
   if(version != FORMAT_VERSION || bytes[10] != 0 || bytes[11] != 0) {
      throw std::runtime_error("BBS credential file has an unsupported format");
   }
   const uint32_t bbs_id =
      (static_cast<uint32_t>(bytes[12]) << 24) |
      (static_cast<uint32_t>(bytes[13]) << 16) |
      (static_cast<uint32_t>(bytes[14]) << 8) |
      static_cast<uint32_t>(bytes[15]);
   if(bbs_id == 0) {
      throw std::runtime_error("BBS credential file contains BBS ID zero");
   }
   BbsCredentialFile result{
      .bbs_id = bbs_id,
      .psk = std::vector<uint8_t>(bytes.begin() + 16, bytes.end()),
   };
   scrub_memory(std::span<uint8_t>(bytes));
   return result;
}

#ifdef _WIN32

std::wstring wide_path(const std::string& path) {
   if(path.empty()) {
      throw std::invalid_argument("credential path must not be empty");
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

class LocalMemory final {
   public:
      explicit LocalMemory(PSECURITY_DESCRIPTOR descriptor) : m_descriptor(descriptor) {}
      ~LocalMemory() {
         if(m_descriptor != nullptr) {
            LocalFree(m_descriptor);
         }
      }
      LocalMemory(const LocalMemory&) = delete;
      LocalMemory& operator=(const LocalMemory&) = delete;
      LocalMemory(LocalMemory&& other) noexcept : m_descriptor(other.m_descriptor) {
         other.m_descriptor = nullptr;
      }
      PSECURITY_DESCRIPTOR get() const { return m_descriptor; }

   private:
      PSECURITY_DESCRIPTOR m_descriptor;
};

LocalMemory secure_descriptor() {
   PSECURITY_DESCRIPTOR descriptor = nullptr;
   if(ConvertStringSecurityDescriptorToSecurityDescriptorW(
         L"D:P(A;;FA;;;OW)(A;;FA;;;SY)(A;;FA;;;BA)",
         SDDL_REVISION_1, &descriptor, nullptr) == 0) {
      throw system_error("security descriptor creation");
   }
   return LocalMemory(descriptor);
}

class Handle final {
   public:
      explicit Handle(HANDLE handle) : m_handle(handle) {}
      ~Handle() {
         if(m_handle != INVALID_HANDLE_VALUE) {
            CloseHandle(m_handle);
         }
      }
      HANDLE get() const { return m_handle; }

   private:
      HANDLE m_handle;
};

void write_file(const std::string& path, std::array<uint8_t, FILE_BYTES>& bytes) {
   auto descriptor = secure_descriptor();
   SECURITY_ATTRIBUTES attributes{
      .nLength = sizeof(SECURITY_ATTRIBUTES),
      .lpSecurityDescriptor = descriptor.get(),
      .bInheritHandle = FALSE,
   };
   const auto path_wide = wide_path(path);
   Handle file(CreateFileW(path_wide.c_str(), GENERIC_WRITE, 0, &attributes,
                           CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr));
   if(file.get() == INVALID_HANDLE_VALUE) {
      throw system_error("exclusive credential file creation");
   }
   DWORD written = 0;
   if(WriteFile(file.get(), bytes.data(), static_cast<DWORD>(bytes.size()),
                &written, nullptr) == 0 ||
      written != bytes.size()) {
      throw system_error("credential file write");
   }
   if(FlushFileBuffers(file.get()) == 0) {
      throw system_error("credential file flush");
   }
}

std::array<uint8_t, FILE_BYTES> read_file(const std::string& path) {
   const auto path_wide = wide_path(path);
   Handle file(CreateFileW(path_wide.c_str(), GENERIC_READ, 0, nullptr,
                           OPEN_EXISTING,
                           FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
                           nullptr));
   if(file.get() == INVALID_HANDLE_VALUE) {
      throw system_error("credential file open");
   }
   BY_HANDLE_FILE_INFORMATION information{};
   if(GetFileInformationByHandle(file.get(), &information) == 0) {
      throw system_error("credential file information");
   }
   if((information.dwFileAttributes &
       (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)) != 0) {
      throw std::runtime_error("BBS credential path must be a regular file");
   }
   // Reapply the protected owner/System/Administrators DACL while the
   // non-shared, verified non-reparse handle prevents path replacement.
   auto descriptor = secure_descriptor();
   if(SetFileSecurityW(
         path_wide.c_str(),
         DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
         descriptor.get()) == 0) {
      throw system_error("credential file DACL validation");
   }
   const uint64_t size =
      (static_cast<uint64_t>(information.nFileSizeHigh) << 32) |
      information.nFileSizeLow;
   if(size != FILE_BYTES) {
      throw std::runtime_error("BBS credential file has an invalid size");
   }
   std::array<uint8_t, FILE_BYTES> bytes{};
   DWORD read = 0;
   if(ReadFile(file.get(), bytes.data(), static_cast<DWORD>(bytes.size()),
               &read, nullptr) == 0 ||
      read != bytes.size()) {
      throw system_error("credential file read");
   }
   return bytes;
}

#else

class FileDescriptor final {
   public:
      explicit FileDescriptor(const int descriptor) : m_descriptor(descriptor) {}
      ~FileDescriptor() {
         if(m_descriptor >= 0) {
            close(m_descriptor);
         }
      }
      int get() const { return m_descriptor; }

   private:
      int m_descriptor;
};

void write_all(const int fd, const std::span<const uint8_t> bytes) {
   size_t offset = 0;
   while(offset < bytes.size()) {
      const auto count = write(fd, bytes.data() + offset, bytes.size() - offset);
      if(count < 0 && errno == EINTR) {
         continue;
      }
      if(count <= 0) {
         throw system_error("credential file write");
      }
      offset += static_cast<size_t>(count);
   }
}

void read_all(const int fd, const std::span<uint8_t> bytes) {
   size_t offset = 0;
   while(offset < bytes.size()) {
      const auto count = read(fd, bytes.data() + offset, bytes.size() - offset);
      if(count < 0 && errno == EINTR) {
         continue;
      }
      if(count <= 0) {
         throw std::runtime_error("BBS credential file is truncated");
      }
      offset += static_cast<size_t>(count);
   }
   uint8_t extra = 0;
   ssize_t count = 0;
   do {
      count = read(fd, &extra, 1);
   } while(count < 0 && errno == EINTR);
   if(count != 0) {
      throw std::runtime_error("BBS credential file has an invalid size");
   }
}

void write_file(const std::string& path, std::array<uint8_t, FILE_BYTES>& bytes) {
   FileDescriptor file(open(path.c_str(),
                            O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW,
                            S_IRUSR | S_IWUSR));
   if(file.get() < 0) {
      throw system_error("exclusive credential file creation");
   }
   write_all(file.get(), bytes);
   if(fsync(file.get()) != 0) {
      throw system_error("credential file flush");
   }
}

std::array<uint8_t, FILE_BYTES> read_file(const std::string& path) {
   FileDescriptor file(open(path.c_str(), O_RDONLY | O_CLOEXEC | O_NOFOLLOW));
   if(file.get() < 0) {
      throw system_error("credential file open");
   }
   struct stat status {};
   if(fstat(file.get(), &status) != 0) {
      throw system_error("credential file information");
   }
   if(!S_ISREG(status.st_mode)) {
      throw std::runtime_error("BBS credential path must be a regular file");
   }
   if(status.st_uid != geteuid()) {
      throw std::runtime_error("BBS credential file is not owned by the current user");
   }
   if((status.st_mode & (S_IRWXG | S_IRWXO)) != 0) {
      throw std::runtime_error(
         "BBS credential file must not grant group or other permissions");
   }
   if(status.st_nlink != 1) {
      throw std::runtime_error("BBS credential file must have exactly one link");
   }
   std::array<uint8_t, FILE_BYTES> bytes{};
   read_all(file.get(), bytes);
   return bytes;
}

#endif

}  // namespace

void create_bbs_credential_file(const std::string& path,
                                const uint32_t bbs_id,
                                const std::span<const uint8_t> psk) {
   auto bytes = encode(bbs_id, psk);
   try {
      write_file(path, bytes);
   } catch(...) {
      scrub_memory(std::span<uint8_t>(bytes));
      throw;
   }
   scrub_memory(std::span<uint8_t>(bytes));
}

BbsCredentialFile read_bbs_credential_file(const std::string& path) {
   auto bytes = read_file(path);
   try {
      return decode(bytes);
   } catch(...) {
      scrub_memory(std::span<uint8_t>(bytes));
      throw;
   }
}

}  // namespace ct
