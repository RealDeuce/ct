//! Versioned, catalog-backed place naming.

use std::collections::HashSet;

use thiserror::Error;

use crate::crypto::{CryptoError, SeedStream, derive_seed};

pub const PLACE_NAMING_VERSION: u16 = 1;
pub const FEDERATION_NAMING_PROFILE_ID: u16 = 1;
const REGION_EDGE_PARSECS: f64 = 25.0;
const MAX_UNIQUE_NAME_DRAWS: usize = 4_096;

pub struct PlaceNameProfile {
    pub id: u16,
    pub tag: &'static str,
    pub system_pattern: &'static str,
    pub world_pattern: &'static str,
    pub system_parts: [&'static [&'static str]; 4],
    pub world_parts: [&'static [&'static str]; 4],
}

include!(concat!(env!("OUT_DIR"), "/place_name_catalog.rs"));

#[derive(Debug, Error)]
pub enum PlaceNameError {
    #[error("cryptographic name stream failed: {0}")]
    Crypto(#[from] CryptoError),
    #[error("unknown place naming profile {0}")]
    UnknownProfile(u16),
    #[error("place naming profile {0} exhausted its collision retry budget")]
    CollisionBudgetExhausted(u16),
}

pub fn profile(profile_id: u16) -> Result<&'static PlaceNameProfile, PlaceNameError> {
    PLACE_NAME_PROFILES
        .iter()
        .find(|profile| profile.id == profile_id)
        .ok_or(PlaceNameError::UnknownProfile(profile_id))
}

pub fn polity_profile(placement_seed: [u8; 32]) -> Result<u16, PlaceNameError> {
    let seed = derive_seed(placement_seed, b"place-naming/polity-profile/v1")?;
    let mut stream = SeedStream::new(seed);
    let non_federation = PLACE_NAME_PROFILES
        .len()
        .checked_sub(1)
        .expect("build script requires multiple naming profiles");
    Ok(2 + (stream.next_u64()? as usize % non_federation) as u16)
}

pub fn regional_profile(position_parsecs: [f64; 3]) -> u16 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for coordinate in position_parsecs {
        let cell = (coordinate / REGION_EDGE_PARSECS).floor() as i64;
        for byte in cell.to_be_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    let non_federation = PLACE_NAME_PROFILES.len() - 1;
    2 + (hash as usize % non_federation) as u16
}

pub fn naming_stream(seed: [u8; 32], domain: &[u8]) -> Result<SeedStream, PlaceNameError> {
    Ok(SeedStream::new(derive_seed(seed, domain)?))
}

pub fn system_name(stream: &mut SeedStream, profile_id: u16) -> Result<String, PlaceNameError> {
    let profile = profile(profile_id)?;
    render(stream, profile.system_pattern, &profile.system_parts)
}

pub fn world_name(stream: &mut SeedStream, profile_id: u16) -> Result<String, PlaceNameError> {
    let profile = profile(profile_id)?;
    render(stream, profile.world_pattern, &profile.world_parts)
}

pub fn unique_system_name(
    stream: &mut SeedStream,
    profile_id: u16,
    used_casefolded: &mut HashSet<String>,
) -> Result<String, PlaceNameError> {
    for _ in 0..MAX_UNIQUE_NAME_DRAWS {
        let candidate = system_name(stream, profile_id)?;
        if used_casefolded.insert(candidate.to_ascii_lowercase()) {
            return Ok(candidate);
        }
    }
    Err(PlaceNameError::CollisionBudgetExhausted(profile_id))
}

fn render(
    stream: &mut SeedStream,
    pattern: &str,
    parts: &[&[&str]; 4],
) -> Result<String, PlaceNameError> {
    let mut result = pattern.to_owned();
    for (index, choices) in parts.iter().enumerate() {
        let selected = choices[stream.next_u64()? as usize % choices.len()];
        result = result.replace(&format!("{{{index}}}"), selected);
    }
    Ok(result.split_whitespace().collect::<Vec<_>>().join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_profiles_are_contiguous_and_have_large_system_spaces() {
        assert_eq!(PLACE_NAME_PROFILES[0].id, FEDERATION_NAMING_PROFILE_ID);
        for (expected, profile) in (1_u16..).zip(PLACE_NAME_PROFILES) {
            assert_eq!(profile.id, expected);
            assert!(!profile.tag.is_empty());
            let candidates = profile
                .system_parts
                .iter()
                .map(|part| part.len())
                .product::<usize>();
            assert!(candidates >= 100_000);
        }
    }

    #[test]
    fn generation_is_repeatable_and_profiles_are_distinct() {
        let mut first = naming_stream([0x42; 32], b"test/names").unwrap();
        let mut second = naming_stream([0x42; 32], b"test/names").unwrap();
        assert_eq!(
            system_name(&mut first, 2).unwrap(),
            system_name(&mut second, 2).unwrap()
        );
        let mut lyric = naming_stream([0x42; 32], b"test/names").unwrap();
        let mut marcher = naming_stream([0x42; 32], b"test/names").unwrap();
        assert_ne!(
            system_name(&mut lyric, 2).unwrap(),
            system_name(&mut marcher, 3).unwrap()
        );
    }

    #[test]
    fn collision_retry_never_returns_an_existing_system_name() {
        let mut probe = naming_stream([0x55; 32], b"test/collision").unwrap();
        let collision = system_name(&mut probe, 4).unwrap();
        let mut used = HashSet::from([collision.to_ascii_lowercase()]);
        let mut retry = naming_stream([0x55; 32], b"test/collision").unwrap();
        let selected = unique_system_name(&mut retry, 4, &mut used).unwrap();
        assert_ne!(
            selected.to_ascii_lowercase(),
            collision.to_ascii_lowercase()
        );
        assert!(used.contains(&selected.to_ascii_lowercase()));
    }

    #[test]
    fn regional_profiles_are_stable_within_a_region() {
        assert_eq!(
            regional_profile([1.0, 2.0, 3.0]),
            regional_profile([20.0, 24.0, 12.0])
        );
        assert!(regional_profile([1.0, 2.0, 3.0]) >= 2);
    }
}
