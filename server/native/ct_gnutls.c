#include "ct_gnutls.h"

#include <gnutls/crypto.h>
#include <gnutls/gnutls.h>
#include <stdlib.h>
#include <string.h>

struct ct_gnutls_server {
    gnutls_session_t session;
    gnutls_psk_server_credentials_t credentials;
    ct_psk_credential* entries;
    size_t entry_count;
};

static int ct_psk_callback(gnutls_session_t session,
                           const gnutls_datum_t* username,
                           gnutls_datum_t* key) {
    struct ct_gnutls_server* server = gnutls_session_get_ptr(session);
    if(server == NULL || username == NULL) {
        return -1;
    }
    for(size_t index = 0; index < server->entry_count; ++index) {
        const ct_psk_credential* entry = &server->entries[index];
        if(username->size != entry->identity_len ||
           memcmp(username->data, entry->identity, entry->identity_len) != 0) {
            continue;
        }
        key->data = gnutls_malloc(entry->key_len);
        if(key->data == NULL) {
            return GNUTLS_E_MEMORY_ERROR;
        }
        memcpy(key->data, entry->key, entry->key_len);
        key->size = (unsigned int)entry->key_len;
        return 0;
    }
    return -1;
}

static void ct_clear_free(uint8_t* data, size_t size) {
    if(data != NULL) {
        gnutls_memset(data, 0, size);
        free(data);
    }
}

static void ct_clear_entries(struct ct_gnutls_server* server) {
    if(server->entries == NULL) {
        return;
    }
    for(size_t index = 0; index < server->entry_count; ++index) {
        ct_psk_credential* entry = &server->entries[index];
        ct_clear_free((uint8_t*)entry->key, entry->key_len);
        free((uint8_t*)entry->identity);
    }
    free(server->entries);
}

ct_gnutls_server* ct_gnutls_server_handshake(int fd,
                                              const ct_psk_credential* credentials,
                                              size_t credential_count,
                                              int* error) {
    struct ct_gnutls_server* server = calloc(1, sizeof(*server));
    int rc = GNUTLS_E_MEMORY_ERROR;
    if(server == NULL || credentials == NULL || credential_count == 0) {
        goto failed;
    }
    server->entries = calloc(credential_count, sizeof(*server->entries));
    if(server->entries == NULL) {
        goto failed;
    }
    server->entry_count = credential_count;
    for(size_t index = 0; index < credential_count; ++index) {
        const ct_psk_credential* source = &credentials[index];
        ct_psk_credential* target = &server->entries[index];
        if(source->identity == NULL || source->identity_len == 0 ||
           source->key == NULL || source->key_len < 32) {
            rc = GNUTLS_E_INVALID_REQUEST;
            goto failed;
        }
        target->identity = malloc(source->identity_len);
        target->key = malloc(source->key_len);
        if(target->identity == NULL || target->key == NULL) {
            goto failed;
        }
        memcpy((uint8_t*)target->identity, source->identity, source->identity_len);
        memcpy((uint8_t*)target->key, source->key, source->key_len);
        target->identity_len = source->identity_len;
        target->key_len = source->key_len;
    }

    rc = gnutls_psk_allocate_server_credentials2(
        &server->credentials, GNUTLS_MAC_SHA256);
    if(rc < 0) {
        goto failed;
    }
    gnutls_psk_set_server_credentials_function2(server->credentials, ct_psk_callback);

    rc = gnutls_init(&server->session,
                     GNUTLS_SERVER | GNUTLS_NO_AUTO_REKEY |
                     GNUTLS_NO_TICKETS | GNUTLS_NO_AUTO_SEND_TICKET);
    if(rc < 0) {
        goto failed;
    }
    gnutls_session_set_ptr(server->session, server);
    rc = gnutls_priority_set_direct(
        server->session,
        "NORMAL:-VERS-ALL:+VERS-TLS1.3:-CIPHER-ALL:+AES-128-GCM"
        ":+DHE-PSK:+ECDHE-PSK:+PSK",
        NULL);
    if(rc < 0) {
        goto failed;
    }
    rc = gnutls_credentials_set(server->session, GNUTLS_CRD_PSK, server->credentials);
    if(rc < 0) {
        goto failed;
    }
    gnutls_transport_set_int(server->session, fd);
    do {
        rc = gnutls_handshake(server->session);
    } while(rc == GNUTLS_E_INTERRUPTED || rc == GNUTLS_E_AGAIN);
    if(rc < 0) {
        goto failed;
    }
    if(gnutls_protocol_get_version(server->session) != GNUTLS_TLS1_3) {
        rc = GNUTLS_E_UNSUPPORTED_VERSION_PACKET;
        goto failed;
    }
    if(error != NULL) {
        *error = 0;
    }
    return server;

failed:
    if(error != NULL) {
        *error = rc;
    }
    if(server != NULL) {
        if(server->session != NULL) {
            gnutls_deinit(server->session);
        }
        if(server->credentials != NULL) {
            gnutls_psk_free_server_credentials(server->credentials);
        }
        ct_clear_entries(server);
        free(server);
    }
    return NULL;
}

ssize_t ct_gnutls_server_recv(ct_gnutls_server* server, uint8_t* data, size_t size) {
    return gnutls_record_recv(server->session, data, size);
}

ssize_t ct_gnutls_server_send(ct_gnutls_server* server, const uint8_t* data, size_t size) {
    return gnutls_record_send(server->session, data, size);
}

const char* ct_gnutls_server_protocol(ct_gnutls_server* server) {
    return gnutls_protocol_get_name(gnutls_protocol_get_version(server->session));
}

int ct_gnutls_server_identity(ct_gnutls_server* server,
                              const uint8_t** identity,
                              size_t* identity_len) {
    gnutls_datum_t username = {0};
    int rc = gnutls_psk_server_get_username2(server->session, &username);
    if(rc < 0) {
        return rc;
    }
    *identity = username.data;
    *identity_len = username.size;
    return 0;
}

void ct_gnutls_server_destroy(ct_gnutls_server* server) {
    if(server == NULL) {
        return;
    }
    gnutls_bye(server->session, GNUTLS_SHUT_WR);
    gnutls_deinit(server->session);
    gnutls_psk_free_server_credentials(server->credentials);
    ct_clear_entries(server);
    free(server);
}

const char* ct_gnutls_error_string(int error) {
    return gnutls_strerror(error);
}

int ct_gnutls_hmac_sha256(const uint8_t* key,
                          size_t key_len,
                          const uint8_t* data,
                          size_t data_len,
                          uint8_t output[32]) {
    if(key == NULL || key_len == 0 || data == NULL || output == NULL) {
        return GNUTLS_E_INVALID_REQUEST;
    }
    return gnutls_hmac_fast(
        GNUTLS_MAC_SHA256, key, key_len, data, data_len, output);
}
