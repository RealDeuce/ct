#include "ct/tls_connection.hpp"

#include <botan/auto_rng.h>
#include <botan/credentials_manager.h>
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
#include <deque>
#include <exception>
#include <mutex>
#include <stdexcept>
#include <string_view>
#include <thread>
#include <utility>

namespace ct {
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
            throw std::runtime_error(
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
         throw std::runtime_error(
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
      throw std::runtime_error("getaddrinfo failed: " + std::string(gai_strerror(result)));
   }
   const auto guard =
      std::unique_ptr<addrinfo, decltype(&freeaddrinfo)>(addresses, freeaddrinfo);
   for(auto* address = addresses; address != nullptr; address = address->ai_next) {
      const SocketHandle fd =
         socket(address->ai_family, address->ai_socktype, address->ai_protocol);
      if(fd == INVALID_SOCKET_HANDLE) {
         continue;
      }
      if(connect(fd, address->ai_addr, address->ai_addrlen) == 0) {
         return fd;
      }
      close_socket(fd);
   }
   throw std::runtime_error("TCP connection failed");
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

class TlsConnection::Impl final : public Botan::TLS::Callbacks {
   public:
      Impl(std::string host,
           std::string service,
           std::string psk_identity,
           std::vector<uint8_t> psk) :
            m_socket(connect_tcp(host, service)),
            m_psk_identity(std::move(psk_identity)),
            m_credentials(make_credentials(m_psk_identity, std::move(psk))),
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

      ~Impl() override {
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
            throw std::runtime_error("fatal TLS alert: " + alert.type_string());
         }
      }

      void tls_session_established(const Botan::TLS::Session_Summary& session) override {
         if(session.version() !=
            Botan::TLS::Protocol_Version(Botan::TLS::Version_Code::TLS_V13)) {
            throw std::runtime_error("server did not negotiate TLS 1.3");
         }
         if(session.external_psk_identity() !=
            std::optional<std::string>(m_psk_identity)) {
            throw std::runtime_error("server did not authenticate the requested external PSK");
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
            throw std::runtime_error("TLS peer closed the connection");
         }
         const std::scoped_lock lock(m_botan_mutex);
         m_client->received_data(
            std::span(encrypted.data(), static_cast<size_t>(count)));
      }

      Socket m_socket;
      std::string m_psk_identity;
      std::shared_ptr<Credentials> m_credentials;
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

TlsConnection::TlsConnection(std::string host,
                             std::string service,
                             std::string psk_identity,
                             std::vector<uint8_t> psk) :
      m_impl(std::make_unique<Impl>(
         std::move(host), std::move(service), std::move(psk_identity), std::move(psk))) {}

TlsConnection::~TlsConnection() = default;
TlsConnection::TlsConnection(TlsConnection&&) noexcept = default;
TlsConnection& TlsConnection::operator=(TlsConnection&&) noexcept = default;

void TlsConnection::send(const std::span<const uint8_t> plaintext) {
   m_impl->send(plaintext);
}

std::vector<uint8_t> TlsConnection::receive_exact(const size_t count) {
   return m_impl->receive_exact(count);
}

void TlsConnection::send_frame(const std::span<const uint8_t> payload) {
   m_impl->send_frame(payload);
}

std::vector<uint8_t> TlsConnection::receive_frame() {
   return m_impl->receive_frame();
}

std::optional<std::vector<uint8_t>> TlsConnection::try_receive_frame() {
   return m_impl->try_receive_frame();
}

void TlsConnection::defer_event_frame(std::vector<uint8_t> frame) {
   m_impl->defer_event_frame(std::move(frame));
}

std::optional<std::vector<uint8_t>> TlsConnection::try_deferred_event_frame() {
   return m_impl->try_deferred_event_frame();
}

void TlsConnection::start_dispatch() {
   m_impl->start_dispatch();
}

std::string TlsConnection::protocol_version() const {
   return m_impl->protocol_version();
}

}  // namespace ct
