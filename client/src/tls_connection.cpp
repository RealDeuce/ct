#include "ct/tls_connection.hpp"

#include "ct/client_transport.h"

#include <array>
#include <stdexcept>
#include <string_view>
#include <utility>

namespace ct {
namespace {

std::string_view error_name(const ct_client_error_code code) {
   switch(code) {
      case CT_CLIENT_ERROR_INVALID_ARGUMENT: return "invalid-argument";
      case CT_CLIENT_ERROR_NETWORK: return "network";
      case CT_CLIENT_ERROR_TLS: return "TLS";
      case CT_CLIENT_ERROR_CRYPTOGRAPHY: return "cryptography";
      case CT_CLIENT_ERROR_INTERNAL: return "internal";
      case CT_CLIENT_ERROR_NONE: return "unspecified";
   }
   return "unknown";
}

[[noreturn]] void throw_transport_error() {
   ct_client_error_info info{};
   std::string message = "client transport failure";
   if(ct_client_last_error_info(&info) == CT_CLIENT_OK && info.message_bytes != 0) {
      std::vector<char> copied(info.message_bytes);
      if(ct_client_last_error_copy(copied.data(), copied.size()) == CT_CLIENT_OK) {
         message = copied.data();
      }
   }
   std::string detail = "client transport error [";
   detail += error_name(info.code);
   if(info.native_code != 0) {
      detail += ", native=" + std::to_string(info.native_code);
   }
   detail += "]: " + message;
   throw std::runtime_error(detail);
}

void require_success(const int status) {
   if(status != CT_CLIENT_OK) {
      throw_transport_error();
   }
}

std::vector<uint8_t> take_frame(
   ct_client_connection* connection,
   const bool deferred,
   const bool nonblocking,
   bool& available) {
   uint8_t* data = nullptr;
   size_t size = 0;
   int status = CT_CLIENT_ERROR;
   if(deferred) {
      status = ct_client_connection_try_deferred_event_frame(connection, &data, &size);
   } else if(nonblocking) {
      status = ct_client_connection_try_receive_frame(connection, &data, &size);
   } else {
      status = ct_client_connection_receive_frame(connection, &data, &size);
   }
   if(status == CT_CLIENT_UNAVAILABLE) {
      available = false;
      return {};
   }
   require_success(status);
   const auto cleanup = std::unique_ptr<uint8_t, decltype(&ct_client_buffer_destroy)>(
      data, ct_client_buffer_destroy);
   available = true;
   if(size == 0) {
      return {};
   }
   return {data, data + size};
}

}  // namespace

class TlsConnection::Impl final {
   public:
      Impl(const std::string& host,
           const std::string& service,
           const std::string& psk_identity,
           std::vector<uint8_t>& psk) {
         const auto status = ct_client_connection_create(
            host.c_str(),
            service.c_str(),
            psk_identity.c_str(),
            psk.data(),
            psk.size(),
            &m_connection);
         ct_client_scrub(psk.data(), psk.size());
         require_success(status);
      }

      ~Impl() { ct_client_connection_destroy(m_connection); }

      Impl(const Impl&) = delete;
      Impl& operator=(const Impl&) = delete;

      ct_client_connection* get() const { return m_connection; }

   private:
      ct_client_connection* m_connection = nullptr;
};

TlsConnection::TlsConnection(std::string host,
                             std::string service,
                             std::string psk_identity,
                             std::vector<uint8_t> psk) :
      m_impl(std::make_unique<Impl>(host, service, psk_identity, psk)) {}

TlsConnection::~TlsConnection() = default;
TlsConnection::TlsConnection(TlsConnection&&) noexcept = default;
TlsConnection& TlsConnection::operator=(TlsConnection&&) noexcept = default;

void TlsConnection::send(const std::span<const uint8_t> plaintext) {
   require_success(ct_client_connection_send(
      m_impl->get(), plaintext.data(), plaintext.size()));
}

std::vector<uint8_t> TlsConnection::receive_exact(const size_t count) {
   std::vector<uint8_t> result(count);
   require_success(ct_client_connection_receive_exact(
      m_impl->get(), result.data(), result.size()));
   return result;
}

void TlsConnection::send_frame(const std::span<const uint8_t> payload) {
   require_success(ct_client_connection_send_frame(
      m_impl->get(), payload.data(), payload.size()));
}

std::vector<uint8_t> TlsConnection::receive_frame() {
   bool available = false;
   return take_frame(m_impl->get(), false, false, available);
}

std::optional<std::vector<uint8_t>> TlsConnection::try_receive_frame() {
   bool available = false;
   auto frame = take_frame(m_impl->get(), false, true, available);
   if(!available) {
      return std::nullopt;
   }
   return frame;
}

void TlsConnection::defer_event_frame(std::vector<uint8_t> frame) {
   require_success(ct_client_connection_defer_event_frame(
      m_impl->get(), frame.data(), frame.size()));
}

std::optional<std::vector<uint8_t>> TlsConnection::try_deferred_event_frame() {
   bool available = false;
   auto frame = take_frame(m_impl->get(), true, true, available);
   if(!available) {
      return std::nullopt;
   }
   return frame;
}

void TlsConnection::start_dispatch() {
   require_success(ct_client_connection_start_dispatch(m_impl->get()));
}

std::string TlsConnection::protocol_version() const {
   std::array<char, 32> version{};
   require_success(ct_client_connection_protocol_version(
      m_impl->get(), version.data(), version.size()));
   return version.data();
}

}  // namespace ct
