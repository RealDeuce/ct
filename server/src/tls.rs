//! GnuTLS 3.x server transport.

use std::ffi::{CStr, c_char, c_int, c_void};
use std::io;
use std::os::fd::AsRawFd;
use std::ptr::NonNull;

use thiserror::Error;

#[repr(C)]
struct NativeServer(c_void);

#[repr(C)]
struct NativeCredential {
    identity: *const u8,
    identity_len: usize,
    key: *const u8,
    key_len: usize,
}

unsafe extern "C" {
    fn ct_gnutls_server_handshake(
        fd: c_int,
        credentials: *const NativeCredential,
        credential_count: usize,
        error: *mut c_int,
    ) -> *mut NativeServer;
    fn ct_gnutls_server_recv(server: *mut NativeServer, data: *mut u8, size: usize) -> isize;
    fn ct_gnutls_server_send(server: *mut NativeServer, data: *const u8, size: usize) -> isize;
    fn ct_gnutls_server_protocol(server: *mut NativeServer) -> *const c_char;
    fn ct_gnutls_server_identity(
        server: *mut NativeServer,
        identity: *mut *const u8,
        identity_len: *mut usize,
    ) -> c_int;
    fn ct_gnutls_server_destroy(server: *mut NativeServer);
    fn ct_gnutls_error_string(error: c_int) -> *const c_char;
}

#[derive(Debug, Error)]
pub enum TlsError {
    #[error("at least one PSK credential is required")]
    EmptyCredentials,
    #[error("PSK identity must not be empty")]
    EmptyIdentity,
    #[error("PSK must contain at least 32 bytes")]
    ShortKey,
    #[error("GnuTLS error {code}: {message}")]
    GnuTls { code: i32, message: String },
    #[error("TLS connection closed")]
    Closed,
    #[error("socket error: {0}")]
    Io(#[from] io::Error),
}

pub struct TlsServer {
    native: NonNull<NativeServer>,
}

#[derive(Clone)]
pub struct PskCredential {
    pub identity: Vec<u8>,
    pub key: Vec<u8>,
}

// GnuTLS documents concurrent use by exactly one sender and one receiver.
// Construction/handshake happens before sharing; automatic rekey is disabled.
unsafe impl Send for TlsServer {}
unsafe impl Sync for TlsServer {}

impl TlsServer {
    pub fn handshake(socket: &impl AsRawFd, identity: &[u8], key: &[u8]) -> Result<Self, TlsError> {
        Self::handshake_many(
            socket,
            &[PskCredential {
                identity: identity.to_vec(),
                key: key.to_vec(),
            }],
        )
    }

    pub fn handshake_many(
        socket: &impl AsRawFd,
        credentials: &[PskCredential],
    ) -> Result<Self, TlsError> {
        if credentials.is_empty() {
            return Err(TlsError::EmptyCredentials);
        }
        for credential in credentials {
            if credential.identity.is_empty() {
                return Err(TlsError::EmptyIdentity);
            }
            if credential.key.len() < 32 {
                return Err(TlsError::ShortKey);
            }
        }
        let native_credentials: Vec<_> = credentials
            .iter()
            .map(|credential| NativeCredential {
                identity: credential.identity.as_ptr(),
                identity_len: credential.identity.len(),
                key: credential.key.as_ptr(),
                key_len: credential.key.len(),
            })
            .collect();
        let mut error = 0;
        // SAFETY: all credential buffers remain valid during the synchronous
        // constructor; the native object copies them before returning.
        let native = unsafe {
            ct_gnutls_server_handshake(
                socket.as_raw_fd(),
                native_credentials.as_ptr(),
                native_credentials.len(),
                &mut error,
            )
        };
        NonNull::new(native)
            .map(|native| Self { native })
            .ok_or_else(|| native_error(error))
    }

    pub fn protocol(&self) -> String {
        // SAFETY: native remains alive for the lifetime of self.
        unsafe {
            CStr::from_ptr(ct_gnutls_server_protocol(self.native.as_ptr()))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn identity(&self) -> Result<Vec<u8>, TlsError> {
        let mut identity = std::ptr::null();
        let mut identity_len = 0;
        // SAFETY: GnuTLS returns session-owned bytes that remain valid while
        // self is alive; they are copied before returning.
        let result = unsafe {
            ct_gnutls_server_identity(self.native.as_ptr(), &mut identity, &mut identity_len)
        };
        if result < 0 {
            return Err(native_error(result));
        }
        // SAFETY: a successful call returned identity_len readable bytes.
        Ok(unsafe { std::slice::from_raw_parts(identity, identity_len) }.to_vec())
    }

    pub fn receive(&self, data: &mut [u8]) -> Result<usize, TlsError> {
        // SAFETY: GnuTLS has exclusive receive-side access and data is writable.
        let result =
            unsafe { ct_gnutls_server_recv(self.native.as_ptr(), data.as_mut_ptr(), data.len()) };
        convert_io_result(result)
    }

    pub fn send(&self, data: &[u8]) -> Result<usize, TlsError> {
        // SAFETY: GnuTLS has exclusive send-side access and data is readable.
        let result =
            unsafe { ct_gnutls_server_send(self.native.as_ptr(), data.as_ptr(), data.len()) };
        convert_io_result(result)
    }
}

impl Drop for TlsServer {
    fn drop(&mut self) {
        // SAFETY: this is the unique destructor after all Arc users are gone.
        unsafe { ct_gnutls_server_destroy(self.native.as_ptr()) };
    }
}

fn convert_io_result(result: isize) -> Result<usize, TlsError> {
    if result > 0 {
        Ok(result as usize)
    } else if result == 0 {
        Err(TlsError::Closed)
    } else {
        Err(native_error(result as i32))
    }
}

fn native_error(code: i32) -> TlsError {
    // SAFETY: GnuTLS returns a process-lifetime static string.
    let message = unsafe {
        CStr::from_ptr(ct_gnutls_error_string(code))
            .to_string_lossy()
            .into_owned()
    };
    TlsError::GnuTls { code, message }
}
