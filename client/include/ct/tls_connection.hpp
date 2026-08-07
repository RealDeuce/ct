#pragma once

#include <cstdint>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <vector>

namespace ct {

class TlsConnection {
   public:
      TlsConnection(std::string host,
                    std::string service,
                    std::string psk_identity,
                    std::vector<uint8_t> psk);
      ~TlsConnection();

      TlsConnection(const TlsConnection&) = delete;
      TlsConnection& operator=(const TlsConnection&) = delete;
      TlsConnection(TlsConnection&&) noexcept;
      TlsConnection& operator=(TlsConnection&&) noexcept;

      void send(std::span<const uint8_t> plaintext);
      std::vector<uint8_t> receive_exact(size_t count);
      void send_frame(std::span<const uint8_t> payload);
      std::vector<uint8_t> receive_frame();
      std::optional<std::vector<uint8_t>> try_receive_frame();
      void defer_event_frame(std::vector<uint8_t> frame);
      std::optional<std::vector<uint8_t>> try_deferred_event_frame();
      void start_dispatch();
      std::string protocol_version() const;

   private:
      class Impl;
      std::unique_ptr<Impl> m_impl;
};

}  // namespace ct
