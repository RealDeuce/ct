#pragma once

#include <cstdint>
#include <span>
#include <string>
#include <vector>

namespace ct {

inline constexpr size_t BBS_PSK_BYTES = 32;

struct BbsCredentialFile {
   uint32_t bbs_id;
   std::vector<uint8_t> psk;
};

void create_bbs_credential_file(const std::string& path,
                                uint32_t bbs_id,
                                std::span<const uint8_t> psk);

BbsCredentialFile read_bbs_credential_file(const std::string& path);

}  // namespace ct
