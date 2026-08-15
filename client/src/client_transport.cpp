#include "ct/client_transport.h"

#include <botan/auto_rng.h>
#include <botan/credentials_manager.h>
#include <botan/hash.h>
#include <botan/mem_ops.h>
#include <botan/tls_alert.h>
#include <botan/tls_callbacks.h>
#include <botan/tls_client.h>
#include <botan/tls_external_psk.h>
#include <botan/tls_policy.h>
#include <botan/tls_session.h>
#include <botan/tls_session_manager_noop.h>

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#else
#include <netdb.h>
#include <sys/socket.h>
#include <unistd.h>
#endif

#include <algorithm>
#include <array>
#include <cerrno>
#include <climits>
#include <cstring>
#include <optional>
#include <condition_variable>
#include <cstdio>
#include <deque>
#include <exception>
#include <mutex>
#include <memory>
#include <span>
#include <stdexcept>
#include <string>
#include <string_view>
#include <thread>
#include <utility>
#include <vector>

namespace ct {

class TransportFailure final : public std::runtime_error {
   public:
      TransportFailure(const ct_client_error_code code,
                       const int64_t native_code,
                       std::string message) :
            std::runtime_error(std::move(message)),
            m_code(code),
            m_native_code(native_code) {}

      ct_client_error_code code() const noexcept { return m_code; }
      int64_t native_code() const noexcept { return m_native_code; }

   private:
      ct_client_error_code m_code;
      int64_t m_native_code;
};

namespace {

#ifdef _WIN32
using SocketHandle = SOCKET;
constexpr SocketHandle INVALID_SOCKET_HANDLE = INVALID_SOCKET;

class WinsockRuntime final {
   public:
      WinsockRuntime() {
         WSADATA data{};
         const auto result = WSAStartup(MAKEWORD(2, 2), &data);
         if(result != 0) {
            throw TransportFailure(
               CT_CLIENT_ERROR_NETWORK,
               result,
               "Winsock initialization failed with error " + std::to_string(result));
         }
      }

      ~WinsockRuntime() { WSACleanup(); }

      WinsockRuntime(const WinsockRuntime&) = delete;
      WinsockRuntime& operator=(const WinsockRuntime&) = delete;
};

void ensure_socket_runtime() {
   static const WinsockRuntime runtime;
   (void)runtime;
}

int socket_error() { return WSAGetLastError(); }
bool socket_interrupted(const int error) { return error == WSAEINTR; }
void close_socket(const SocketHandle socket) { closesocket(socket); }
constexpr int SHUTDOWN_BOTH = SD_BOTH;
#else
using SocketHandle = int;
constexpr SocketHandle INVALID_SOCKET_HANDLE = -1;

void ensure_socket_runtime() {}
int socket_error() { return errno; }
bool socket_interrupted(const int error) { return error == EINTR; }
void close_socket(const SocketHandle socket) { close(socket); }
constexpr int SHUTDOWN_BOTH = SHUT_RDWR;
#endif

void send_all(const SocketHandle fd, std::span<const uint8_t> data) {
   while(!data.empty()) {
#ifdef _WIN32
      const auto chunk_size = static_cast<int>(std::min<size_t>(data.size(), INT_MAX));
      const auto sent =
         ::send(fd, reinterpret_cast<const char*>(data.data()), chunk_size, 0);
#else
      const auto sent = ::send(fd, data.data(), data.size(), 0);
#endif
      const auto error = sent < 0 ? socket_error() : 0;
      if(sent < 0 && socket_interrupted(error)) {
         continue;
      }
      if(sent <= 0) {
         throw TransportFailure(
            CT_CLIENT_ERROR_NETWORK,
            error,
            "socket send failed with error " + std::to_string(error));
      }
      data = data.subspan(static_cast<size_t>(sent));
   }
}

SocketHandle connect_tcp(const std::string& host, const std::string& service) {
   ensure_socket_runtime();
   addrinfo hints{};
   hints.ai_family = AF_UNSPEC;
   hints.ai_socktype = SOCK_STREAM;
   addrinfo* addresses = nullptr;
   const auto result = getaddrinfo(host.c_str(), service.c_str(), &hints, &addresses);
   if(result != 0) {
      throw TransportFailure(
         CT_CLIENT_ERROR_NETWORK,
         result,
         "getaddrinfo failed: " + std::string(gai_strerror(result)));
   }
   const auto guard =
      std::unique_ptr<addrinfo, decltype(&freeaddrinfo)>(addresses, freeaddrinfo);
   int last_connect_error = 0;
   for(auto* address = addresses; address != nullptr; address = address->ai_next) {
      const SocketHandle fd =
         socket(address->ai_family, address->ai_socktype, address->ai_protocol);
      if(fd == INVALID_SOCKET_HANDLE) {
         continue;
      }
      if(connect(fd, address->ai_addr, address->ai_addrlen) == 0) {
         return fd;
      }
      last_connect_error = socket_error();
      close_socket(fd);
   }
   throw TransportFailure(
      CT_CLIENT_ERROR_NETWORK, last_connect_error, "TCP connection failed");
}

class Socket final {
   public:
      explicit Socket(const SocketHandle fd) : m_fd(fd) {}
      ~Socket() {
         if(m_fd != INVALID_SOCKET_HANDLE) {
            close_socket(m_fd);
         }
      }

      Socket(const Socket&) = delete;
      Socket& operator=(const Socket&) = delete;

      SocketHandle get() const { return m_fd; }
      void shutdown_both() const { ::shutdown(m_fd, SHUTDOWN_BOTH); }

   private:
      SocketHandle m_fd;
};

class Credentials;

std::shared_ptr<Credentials> make_credentials(
   std::string identity,
   std::vector<uint8_t> psk);

class Credentials final : public Botan::Credentials_Manager {
   public:
      Credentials(std::string identity, Botan::secure_vector<uint8_t> key) :
            m_identity(std::move(identity)), m_key(std::move(key)) {}

      std::vector<Botan::TLS::ExternalPSK> find_preshared_keys(
         std::string_view,
         Botan::TLS::Connection_Side,
         const std::vector<std::string>& identities,
         const std::optional<std::string>& prf) override {
         if(!identities.empty() &&
            std::find(identities.begin(), identities.end(), m_identity) == identities.end()) {
            return {};
         }
         if(prf.has_value() && *prf != "SHA-256") {
            return {};
         }
         std::vector<Botan::TLS::ExternalPSK> result;
         result.emplace_back(m_identity, "SHA-256", m_key);
         return result;
      }

   private:
      std::string m_identity;
      Botan::secure_vector<uint8_t> m_key;
};

std::shared_ptr<Credentials> make_credentials(
   std::string identity,
   std::vector<uint8_t> psk) {
   if(identity.empty()) {
      throw std::invalid_argument("BBS/PSK identity must not be empty");
   }
   if(psk.size() < 32) {
      throw std::invalid_argument("PSK must contain at least 32 bytes");
   }
   Botan::secure_vector<uint8_t> secure_psk(psk.begin(), psk.end());
   Botan::secure_scrub_memory(psk.data(), psk.size());
   return std::make_shared<Credentials>(std::move(identity), std::move(secure_psk));
}

class Policy final : public Botan::TLS::Strict_Policy {
   public:
      std::vector<Botan::TLS::Group_Params> key_exchange_groups() const override {
         return {
            Botan::TLS::Group_Params::X25519,
         };
      }

      std::vector<std::string> allowed_key_exchange_methods() const override {
         return {
            "ECDHE_PSK",
         };
      }

      std::vector<std::string> allowed_signature_methods() const override {
         return {
            "IMPLICIT",
         };
      }
};

}  // namespace

class TransportImpl final : public Botan::TLS::Callbacks {
   public:
      TransportImpl(std::string host,
                    std::string service,
                    std::string psk_identity,
                    std::vector<uint8_t> psk) :
            m_psk_identity(std::move(psk_identity)),
            m_credentials(make_credentials(m_psk_identity, std::move(psk))),
            m_socket(connect_tcp(host, service)),
            m_sessions(std::make_shared<Botan::TLS::Session_Manager_Noop>()),
            m_rng(std::make_shared<Botan::AutoSeeded_RNG>()),
            m_policy(std::make_shared<Policy>()),
            m_client(std::make_unique<Botan::TLS::Client>(
               std::shared_ptr<Botan::TLS::Callbacks>(this, [](Botan::TLS::Callbacks*) {}),
               m_sessions,
               m_credentials,
               m_policy,
               m_rng,
               Botan::TLS::Server_Information(host, static_cast<uint16_t>(std::stoul(service))),
               Botan::TLS::Protocol_Version(Botan::TLS::Version_Code::TLS_V13))) {
         while(!m_client->is_active()) {
            pump();
         }
      }

      ~TransportImpl() override {
         if(m_client != nullptr && m_client->is_active()) {
            try {
               const std::scoped_lock lock(m_botan_mutex);
               m_client->close();
            } catch(...) {
            }
         }
         if(m_dispatch.joinable()) {
            m_socket.shutdown_both();
            m_dispatch.join();
         }
      }

      void tls_emit_data(std::span<const uint8_t> data) override {
         send_all(m_socket.get(), data);
      }

      void tls_record_received(uint64_t, std::span<const uint8_t> data) override {
         m_plaintext.insert(m_plaintext.end(), data.begin(), data.end());
      }

      void tls_alert(Botan::TLS::Alert alert) override {
         if(alert.is_fatal()) {
            throw TransportFailure(
               CT_CLIENT_ERROR_TLS,
               static_cast<int64_t>(alert.type()),
               "fatal TLS alert: " + alert.type_string());
         }
      }

      void tls_session_established(const Botan::TLS::Session_Summary& session) override {
         if(session.version() !=
            Botan::TLS::Protocol_Version(Botan::TLS::Version_Code::TLS_V13)) {
            throw TransportFailure(
               CT_CLIENT_ERROR_TLS, 0, "server did not negotiate TLS 1.3");
         }
         if(session.external_psk_identity() !=
            std::optional<std::string>(m_psk_identity)) {
            throw TransportFailure(
               CT_CLIENT_ERROR_TLS,
               0,
               "server did not authenticate the requested external PSK");
         }
         m_protocol_version = "TLS1.3";
      }

      void send(std::span<const uint8_t> plaintext) {
         const std::scoped_lock lock(m_botan_mutex);
         m_client->send(plaintext);
      }

      void send_frame(std::span<const uint8_t> payload) {
         if(payload.empty() || payload.size() > 1024 * 1024) {
            throw std::runtime_error("invalid outgoing CT-RPC frame size");
         }
         const auto size = static_cast<uint32_t>(payload.size());
         const std::array<uint8_t, 4> header = {
            static_cast<uint8_t>(size >> 24),
            static_cast<uint8_t>(size >> 16),
            static_cast<uint8_t>(size >> 8),
            static_cast<uint8_t>(size),
         };
         const std::scoped_lock lock(m_botan_mutex);
         m_client->send(header);
         m_client->send(payload);
      }

      std::vector<uint8_t> receive_exact(const size_t count) {
         if(m_dispatch.joinable()) {
            throw std::runtime_error("byte receive is unavailable after CT-RPC dispatch starts");
         }
         while(m_plaintext.size() - m_plaintext_offset < count) {
            pump();
         }
         const auto begin = m_plaintext.begin() + static_cast<std::ptrdiff_t>(m_plaintext_offset);
         std::vector<uint8_t> result(begin, begin + static_cast<std::ptrdiff_t>(count));
         m_plaintext_offset += count;
         if(m_plaintext_offset == m_plaintext.size()) {
            m_plaintext.clear();
            m_plaintext_offset = 0;
         }
         return result;
      }

      void start_dispatch() {
         if(m_dispatch.joinable()) {
            return;
         }
         m_dispatch = std::thread([this] {
            try {
               for(;;) {
                  while(m_plaintext.size() - m_plaintext_offset < 4) {
                     pump();
                  }
                  const auto* header = m_plaintext.data() + m_plaintext_offset;
                  const auto size = (static_cast<uint32_t>(header[0]) << 24) |
                                    (static_cast<uint32_t>(header[1]) << 16) |
                                    (static_cast<uint32_t>(header[2]) << 8) |
                                    static_cast<uint32_t>(header[3]);
                  if(size == 0 || size > 1024 * 1024) {
                     throw std::runtime_error("invalid incoming CT-RPC frame size");
                  }
                  while(m_plaintext.size() - m_plaintext_offset < 4 + size) {
                     pump();
                  }
                  const auto begin = m_plaintext.begin() +
                                     static_cast<std::ptrdiff_t>(m_plaintext_offset + 4);
                  std::vector<uint8_t> frame(
                     begin, begin + static_cast<std::ptrdiff_t>(size));
                  m_plaintext_offset += 4 + size;
                  if(m_plaintext_offset == m_plaintext.size()) {
                     m_plaintext.clear();
                     m_plaintext_offset = 0;
                  }
                  {
                     const std::scoped_lock lock(m_queue_mutex);
                     m_frames.push_back(std::move(frame));
                  }
                  m_queue_changed.notify_all();
               }
            } catch(...) {
               {
                  const std::scoped_lock lock(m_queue_mutex);
                  m_dispatch_error = std::current_exception();
                  m_dispatch_closed = true;
               }
               m_queue_changed.notify_all();
            }
         });
      }

      std::vector<uint8_t> receive_frame() {
         std::unique_lock lock(m_queue_mutex);
         m_queue_changed.wait(lock, [this] {
            return !m_frames.empty() || m_dispatch_closed;
         });
         if(m_frames.empty()) {
            if(m_dispatch_error) {
               std::rethrow_exception(m_dispatch_error);
            }
            throw std::runtime_error("TLS peer closed the connection");
         }
         auto frame = std::move(m_frames.front());
         m_frames.pop_front();
         return frame;
      }

      std::optional<std::vector<uint8_t>> try_receive_frame() {
         const std::scoped_lock lock(m_queue_mutex);
         if(m_frames.empty()) {
            if(m_dispatch_closed && m_dispatch_error) {
               std::rethrow_exception(m_dispatch_error);
            }
            return std::nullopt;
         }
         auto frame = std::move(m_frames.front());
         m_frames.pop_front();
         return frame;
      }

      void defer_event_frame(std::vector<uint8_t> frame) {
         const std::scoped_lock lock(m_queue_mutex);
         m_deferred_events.push_back(std::move(frame));
      }

      std::optional<std::vector<uint8_t>> try_deferred_event_frame() {
         const std::scoped_lock lock(m_queue_mutex);
         if(m_deferred_events.empty()) {
            return std::nullopt;
         }
         auto frame = std::move(m_deferred_events.front());
         m_deferred_events.pop_front();
         return frame;
      }

      std::string protocol_version() const {
         return m_protocol_version;
      }

   private:
      void pump() {
         std::vector<uint8_t> encrypted(64 * 1024);
         std::ptrdiff_t count = 0;
         int error = 0;
         do {
#ifdef _WIN32
            count = recv(
               m_socket.get(),
               reinterpret_cast<char*>(encrypted.data()),
               static_cast<int>(encrypted.size()),
               0);
#else
            count = recv(m_socket.get(), encrypted.data(), encrypted.size(), 0);
#endif
            error = count < 0 ? socket_error() : 0;
         } while(count < 0 && socket_interrupted(error));
         if(count <= 0) {
            throw TransportFailure(
               CT_CLIENT_ERROR_NETWORK,
               error,
               "TLS peer closed the connection");
         }
         const std::scoped_lock lock(m_botan_mutex);
         m_client->received_data(
            std::span(encrypted.data(), static_cast<size_t>(count)));
      }

      std::string m_psk_identity;
      std::shared_ptr<Credentials> m_credentials;
      Socket m_socket;
      std::shared_ptr<Botan::TLS::Session_Manager> m_sessions;
      std::shared_ptr<Botan::RandomNumberGenerator> m_rng;
      std::shared_ptr<Policy> m_policy;
      std::unique_ptr<Botan::TLS::Client> m_client;
      mutable std::mutex m_botan_mutex;
      std::vector<uint8_t> m_plaintext;
      size_t m_plaintext_offset = 0;
      std::string m_protocol_version;
      std::thread m_dispatch;
      std::mutex m_queue_mutex;
      std::condition_variable m_queue_changed;
      std::deque<std::vector<uint8_t>> m_frames;
      std::deque<std::vector<uint8_t>> m_deferred_events;
      std::exception_ptr m_dispatch_error;
      bool m_dispatch_closed = false;
};

}  // namespace ct

struct ct_client_connection {
   std::unique_ptr<ct::TransportImpl> implementation;
};

namespace {

struct ErrorState {
   ct_client_error_code code = CT_CLIENT_ERROR_NONE;
   int64_t native_code = 0;
   std::string message;
};

thread_local ErrorState last_error;

void clear_error() noexcept
{
   last_error.code = CT_CLIENT_ERROR_NONE;
   last_error.native_code = 0;
   last_error.message.clear();
}

void record_error(const ct_client_error_code code,
                  const int64_t native_code,
                  const char* message) noexcept
{
   last_error.code = code;
   last_error.native_code = native_code;
   try {
      last_error.message =
         message != nullptr ? message : "unknown client transport failure";
   } catch(...) {
      last_error.message.clear();
   }
}

template<typename Operation>
int guarded(
   Operation&& operation,
   const ct_client_error_code fallback = CT_CLIENT_ERROR_INTERNAL) noexcept
{
   clear_error();
   try {
      std::forward<Operation>(operation)();
      return CT_CLIENT_OK;
   } catch(const ct::TransportFailure& error) {
      record_error(error.code(), error.native_code(), error.what());
   } catch(const std::invalid_argument& error) {
      record_error(CT_CLIENT_ERROR_INVALID_ARGUMENT, 0, error.what());
   } catch(const std::exception& error) {
      record_error(fallback, 0, error.what());
   } catch(...) {
      record_error(fallback, 0, "unknown client transport failure");
   }
   return CT_CLIENT_ERROR;
}

ct::TransportImpl& implementation(ct_client_connection* connection)
{
   if(connection == nullptr || connection->implementation == nullptr) {
      throw std::invalid_argument("client transport connection is null");
   }
   return *connection->implementation;
}

const ct::TransportImpl& implementation(const ct_client_connection* connection)
{
   if(connection == nullptr || connection->implementation == nullptr) {
      throw std::invalid_argument("client transport connection is null");
   }
   return *connection->implementation;
}

std::span<const uint8_t> input_bytes(const uint8_t* data, const size_t size)
{
   if(size != 0 && data == nullptr) {
      throw std::invalid_argument("client transport input buffer is null");
   }
   return {data, size};
}

void return_frame(std::vector<uint8_t> frame, uint8_t** payload, size_t* payload_size)
{
   if(payload == nullptr || payload_size == nullptr) {
      throw std::invalid_argument("client transport output buffer is null");
   }
   *payload = nullptr;
   *payload_size = 0;
   auto result = std::make_unique<uint8_t[]>(frame.size());
   std::copy(frame.begin(), frame.end(), result.get());
   *payload_size = frame.size();
   *payload = result.release();
}

}  // namespace

extern "C" {

int ct_client_last_error_info(ct_client_error_info* info) noexcept
{
   if(info == nullptr) {
      return CT_CLIENT_ERROR;
   }
   info->code = last_error.code;
   info->native_code = last_error.native_code;
   info->message_bytes = last_error.message.size() + 1;
   return CT_CLIENT_OK;
}

int ct_client_last_error_copy(char* message, const size_t message_size) noexcept
{
   if(message == nullptr || message_size < last_error.message.size() + 1) {
      return CT_CLIENT_ERROR;
   }
   std::copy(last_error.message.begin(), last_error.message.end(), message);
   message[last_error.message.size()] = '\0';
   return CT_CLIENT_OK;
}

int ct_client_connection_create(
   const char* host,
   const char* service,
   const char* psk_identity,
   const uint8_t* psk,
   const size_t psk_size,
   ct_client_connection** result) noexcept
{
   return guarded([&] {
      if(result == nullptr) {
         throw std::invalid_argument("client transport creation argument is null");
      }
      *result = nullptr;
      if(host == nullptr || service == nullptr || psk_identity == nullptr) {
         throw std::invalid_argument("client transport creation argument is null");
      }
      const auto key = input_bytes(psk, psk_size);
      auto connection = std::make_unique<ct_client_connection>();
      connection->implementation = std::make_unique<ct::TransportImpl>(
         host,
         service,
         psk_identity,
         std::vector<uint8_t>(key.begin(), key.end()));
      *result = connection.release();
   }, CT_CLIENT_ERROR_TLS);
}

void ct_client_connection_destroy(ct_client_connection* connection) noexcept
{
   try {
      delete connection;
   } catch(...) {
   }
}

int ct_client_connection_send(
   ct_client_connection* connection,
   const uint8_t* plaintext,
   const size_t plaintext_size) noexcept
{
   return guarded([&] {
      implementation(connection).send(input_bytes(plaintext, plaintext_size));
   }, CT_CLIENT_ERROR_TLS);
}

int ct_client_connection_receive_exact(
   ct_client_connection* connection,
   uint8_t* plaintext,
   const size_t plaintext_size) noexcept
{
   return guarded([&] {
      if(plaintext_size != 0 && plaintext == nullptr) {
         throw std::invalid_argument("client transport output buffer is null");
      }
      const auto received = implementation(connection).receive_exact(plaintext_size);
      std::copy(received.begin(), received.end(), plaintext);
   }, CT_CLIENT_ERROR_TLS);
}

int ct_client_connection_send_frame(
   ct_client_connection* connection,
   const uint8_t* payload,
   const size_t payload_size) noexcept
{
   return guarded([&] {
      implementation(connection).send_frame(input_bytes(payload, payload_size));
   }, CT_CLIENT_ERROR_TLS);
}

int ct_client_connection_receive_frame(
   ct_client_connection* connection,
   uint8_t** payload,
   size_t* payload_size) noexcept
{
   return guarded([&] {
      return_frame(
         implementation(connection).receive_frame(), payload, payload_size);
   }, CT_CLIENT_ERROR_TLS);
}

int ct_client_connection_try_receive_frame(
   ct_client_connection* connection,
   uint8_t** payload,
   size_t* payload_size) noexcept
{
   int status = CT_CLIENT_OK;
   const auto guarded_status = guarded([&] {
      if(payload == nullptr || payload_size == nullptr) {
         throw std::invalid_argument("client transport output buffer is null");
      }
      *payload = nullptr;
      *payload_size = 0;
      auto frame = implementation(connection).try_receive_frame();
      if(!frame.has_value()) {
         status = CT_CLIENT_UNAVAILABLE;
         return;
      }
      return_frame(std::move(*frame), payload, payload_size);
   }, CT_CLIENT_ERROR_TLS);
   return guarded_status == CT_CLIENT_OK ? status : guarded_status;
}

void ct_client_buffer_destroy(uint8_t* buffer) noexcept
{
   delete[] buffer;
}

int ct_client_connection_defer_event_frame(
   ct_client_connection* connection,
   const uint8_t* payload,
   const size_t payload_size) noexcept
{
   return guarded([&] {
      const auto bytes = input_bytes(payload, payload_size);
      implementation(connection).defer_event_frame(
         std::vector<uint8_t>(bytes.begin(), bytes.end()));
   }, CT_CLIENT_ERROR_TLS);
}

int ct_client_connection_try_deferred_event_frame(
   ct_client_connection* connection,
   uint8_t** payload,
   size_t* payload_size) noexcept
{
   int status = CT_CLIENT_OK;
   const auto guarded_status = guarded([&] {
      if(payload == nullptr || payload_size == nullptr) {
         throw std::invalid_argument("client transport output buffer is null");
      }
      *payload = nullptr;
      *payload_size = 0;
      auto frame = implementation(connection).try_deferred_event_frame();
      if(!frame.has_value()) {
         status = CT_CLIENT_UNAVAILABLE;
         return;
      }
      return_frame(std::move(*frame), payload, payload_size);
   }, CT_CLIENT_ERROR_TLS);
   return guarded_status == CT_CLIENT_OK ? status : guarded_status;
}

int ct_client_connection_start_dispatch(ct_client_connection* connection) noexcept
{
   return guarded([&] {
      implementation(connection).start_dispatch();
   }, CT_CLIENT_ERROR_TLS);
}

int ct_client_connection_protocol_version(
   const ct_client_connection* connection,
   char* version,
   const size_t version_size) noexcept
{
   return guarded([&] {
      if(version == nullptr || version_size == 0) {
         throw std::invalid_argument("client transport version buffer is null");
      }
      const auto value = implementation(connection).protocol_version();
      if(value.size() + 1 > version_size) {
         throw std::runtime_error("client transport version buffer is too small");
      }
      std::copy(value.begin(), value.end(), version);
      version[value.size()] = '\0';
   }, CT_CLIENT_ERROR_TLS);
}

int ct_client_randomize(uint8_t* output, const size_t output_size) noexcept
{
   return guarded([&] {
      if(output_size != 0 && output == nullptr) {
         throw std::invalid_argument("random output buffer is null");
      }
      thread_local Botan::AutoSeeded_RNG random;
      random.randomize(std::span<uint8_t>(output, output_size));
   }, CT_CLIENT_ERROR_CRYPTOGRAPHY);
}

int ct_client_sha256(
   const uint8_t* input,
   const size_t input_size,
   uint8_t output[32]) noexcept
{
   return guarded([&] {
      if(output == nullptr) {
         throw std::invalid_argument("SHA-256 output buffer is null");
      }
      const auto bytes = input_bytes(input, input_size);
      auto hash = Botan::HashFunction::create_or_throw("SHA-256");
      hash->update(bytes);
      hash->final(std::span<uint8_t>(output, 32));
   }, CT_CLIENT_ERROR_CRYPTOGRAPHY);
}

void ct_client_scrub(void* memory, const size_t memory_size) noexcept
{
   if(memory_size != 0 && memory != nullptr) {
      Botan::secure_scrub_memory(memory, memory_size);
   }
}

}  // extern "C"
