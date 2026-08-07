#ifndef CT_GNUTLS_H
#define CT_GNUTLS_H

#include <stddef.h>
#include <stdint.h>
#include <sys/types.h>

typedef struct ct_gnutls_server ct_gnutls_server;

typedef struct {
    const uint8_t* identity;
    size_t identity_len;
    const uint8_t* key;
    size_t key_len;
} ct_psk_credential;

ct_gnutls_server* ct_gnutls_server_handshake(int fd,
                                              const ct_psk_credential* credentials,
                                              size_t credential_count,
                                              int* error);
ssize_t ct_gnutls_server_recv(ct_gnutls_server* server, uint8_t* data, size_t size);
ssize_t ct_gnutls_server_send(ct_gnutls_server* server, const uint8_t* data, size_t size);
const char* ct_gnutls_server_protocol(ct_gnutls_server* server);
int ct_gnutls_server_identity(ct_gnutls_server* server,
                              const uint8_t** identity,
                              size_t* identity_len);
void ct_gnutls_server_destroy(ct_gnutls_server* server);
const char* ct_gnutls_error_string(int error);
int ct_gnutls_hmac_sha256(const uint8_t* key,
                          size_t key_len,
                          const uint8_t* data,
                          size_t data_len,
                          uint8_t output[32]);

#endif
