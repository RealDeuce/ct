//! Small deterministic cryptographic streams for persisted generation.
//!
//! Entropy enters through a 256-bit operating-system CSPRNG seed. Generation
//! then uses HMAC-SHA-256 in counter mode so retries and rejection sampling
//! consume a stable, auditable stream without adding another runtime library.

use std::ffi::{CStr, c_char, c_int};

use thiserror::Error;

const SEED_STREAM_DOMAIN: &[u8] = b"cepheus-trader/seed-stream/v1";
const DERIVED_SEED_DOMAIN: &[u8] = b"cepheus-trader/derived-seed/v1";

unsafe extern "C" {
    fn ct_gnutls_hmac_sha256(
        key: *const u8,
        key_len: usize,
        data: *const u8,
        data_len: usize,
        output: *mut u8,
    ) -> c_int;
    fn ct_gnutls_error_string(error: c_int) -> *const c_char;
}

#[derive(Debug, Error)]
#[error("GnuTLS HMAC error {code}: {message}")]
pub struct CryptoError {
    code: i32,
    message: String,
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<[u8; 32], CryptoError> {
    let mut output = [0; 32];
    // SAFETY: all slices remain alive for this synchronous call and output is
    // exactly the SHA-256 digest size required by the native adapter.
    let result = unsafe {
        ct_gnutls_hmac_sha256(
            key.as_ptr(),
            key.len(),
            data.as_ptr(),
            data.len(),
            output.as_mut_ptr(),
        )
    };
    if result < 0 {
        // SAFETY: GnuTLS returns a process-lifetime static error string.
        let message = unsafe {
            CStr::from_ptr(ct_gnutls_error_string(result))
                .to_string_lossy()
                .into_owned()
        };
        return Err(CryptoError {
            code: result,
            message,
        });
    }
    Ok(output)
}

/// Derive a stable, domain-separated child seed.
///
/// Labels are part of the generation contract and must include their own
/// version. Adding a sibling label never consumes or shifts another stream.
pub fn derive_seed(key: [u8; 32], label: &[u8]) -> Result<[u8; 32], CryptoError> {
    let mut input = Vec::with_capacity(DERIVED_SEED_DOMAIN.len() + 8 + label.len());
    input.extend_from_slice(DERIVED_SEED_DOMAIN);
    input.extend_from_slice(&(label.len() as u64).to_be_bytes());
    input.extend_from_slice(label);
    hmac_sha256(&key, &input)
}

#[derive(Clone, Debug)]
pub struct SeedStream {
    key: [u8; 32],
    counter: u64,
}

impl SeedStream {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key, counter: 0 }
    }

    pub fn next_seed(&mut self) -> Result<[u8; 32], CryptoError> {
        let counter = self.counter;
        self.counter = self
            .counter
            .checked_add(1)
            .expect("seed-stream counter exhausted");
        let mut input = Vec::with_capacity(SEED_STREAM_DOMAIN.len() + 8);
        input.extend_from_slice(SEED_STREAM_DOMAIN);
        input.extend_from_slice(&counter.to_be_bytes());
        hmac_sha256(&self.key, &input)
    }

    pub fn next_u64(&mut self) -> Result<u64, CryptoError> {
        Ok(u64::from_be_bytes(
            self.next_seed()?[..8]
                .try_into()
                .expect("eight-byte seed prefix"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_stream_is_repeatable_and_counter_separated() {
        let mut first = SeedStream::new([0x42; 32]);
        let mut second = SeedStream::new([0x42; 32]);
        let first_zero = first.next_seed().unwrap();
        let first_one = first.next_seed().unwrap();
        assert_eq!(first_zero, second.next_seed().unwrap());
        assert_eq!(first_one, second.next_seed().unwrap());
        assert_ne!(first_zero, first_one);
    }

    #[test]
    fn derived_seed_labels_are_repeatable_and_independent() {
        let key = [0x5a; 32];
        assert_eq!(
            derive_seed(key, b"celestial/stellar/v1").unwrap(),
            derive_seed(key, b"celestial/stellar/v1").unwrap()
        );
        assert_ne!(
            derive_seed(key, b"celestial/stellar/v1").unwrap(),
            derive_seed(key, b"celestial/orbits/v1").unwrap()
        );
    }
}
