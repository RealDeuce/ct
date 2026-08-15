#pragma once

#include <stddef.h>
#include <stdint.h>

#if defined(_WIN32)
#if defined(CT_CLIENT_TRANSPORT_BUILD)
#define CT_CLIENT_TRANSPORT_API __declspec(dllexport)
#else
#define CT_CLIENT_TRANSPORT_API __declspec(dllimport)
#endif
#else
#define CT_CLIENT_TRANSPORT_API __attribute__((visibility("default")))
#endif

#ifdef __cplusplus
#define CT_CLIENT_NOEXCEPT noexcept
extern "C" {
#else
#define CT_CLIENT_NOEXCEPT
#endif

typedef struct ct_client_connection ct_client_connection;

#define CT_CLIENT_TRANSPORT_ABI_VERSION 1

enum {
   CT_CLIENT_OK = 0,
   CT_CLIENT_UNAVAILABLE = 1,
   CT_CLIENT_ERROR = -1,
};

typedef enum ct_client_error_code {
   CT_CLIENT_ERROR_NONE = 0,
   CT_CLIENT_ERROR_INVALID_ARGUMENT = 1,
   CT_CLIENT_ERROR_NETWORK = 2,
   CT_CLIENT_ERROR_TLS = 3,
   CT_CLIENT_ERROR_CRYPTOGRAPHY = 4,
   CT_CLIENT_ERROR_INTERNAL = 5,
} ct_client_error_code;

typedef struct ct_client_error_info {
   ct_client_error_code code;
   int64_t native_code;
   size_t message_bytes;
} ct_client_error_info;

CT_CLIENT_TRANSPORT_API int ct_client_last_error_info(
   ct_client_error_info* info) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API int ct_client_last_error_copy(
   char* message,
   size_t message_size) CT_CLIENT_NOEXCEPT;

CT_CLIENT_TRANSPORT_API int ct_client_connection_create(
   const char* host,
   const char* service,
   const char* psk_identity,
   const uint8_t* psk,
   size_t psk_size,
   ct_client_connection** result) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API void ct_client_connection_destroy(
   ct_client_connection* connection) CT_CLIENT_NOEXCEPT;

CT_CLIENT_TRANSPORT_API int ct_client_connection_send(
   ct_client_connection* connection,
   const uint8_t* plaintext,
   size_t plaintext_size) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API int ct_client_connection_receive_exact(
   ct_client_connection* connection,
   uint8_t* plaintext,
   size_t plaintext_size) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API int ct_client_connection_send_frame(
   ct_client_connection* connection,
   const uint8_t* payload,
   size_t payload_size) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API int ct_client_connection_receive_frame(
   ct_client_connection* connection,
   uint8_t** payload,
   size_t* payload_size) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API int ct_client_connection_try_receive_frame(
   ct_client_connection* connection,
   uint8_t** payload,
   size_t* payload_size) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API void ct_client_buffer_destroy(
   uint8_t* buffer) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API int ct_client_connection_defer_event_frame(
   ct_client_connection* connection,
   const uint8_t* payload,
   size_t payload_size) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API int ct_client_connection_try_deferred_event_frame(
   ct_client_connection* connection,
   uint8_t** payload,
   size_t* payload_size) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API int ct_client_connection_start_dispatch(
   ct_client_connection* connection) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API int ct_client_connection_protocol_version(
   const ct_client_connection* connection,
   char* version,
   size_t version_size) CT_CLIENT_NOEXCEPT;

CT_CLIENT_TRANSPORT_API int ct_client_randomize(
   uint8_t* output,
   size_t output_size) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API int ct_client_sha256(
   const uint8_t* input,
   size_t input_size,
   uint8_t output[32]) CT_CLIENT_NOEXCEPT;
CT_CLIENT_TRANSPORT_API void ct_client_scrub(
   void* memory,
   size_t memory_size) CT_CLIENT_NOEXCEPT;

#ifdef __cplusplus
}
#endif

#undef CT_CLIENT_NOEXCEPT
