//! Deterministic, setting-neutral names for materialized people.

pub const PERSON_NAME_GENERATION_VERSION: u16 = 1;

include!(concat!(env!("OUT_DIR"), "/person_name_catalog.rs"));

pub fn generated_person_name(entropy: u64, discriminator: u64) -> String {
    let domain = entropy
        ^ discriminator.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ PERSON_NAME_CATALOG_REVISION.rotate_left(29)
        ^ u64::from(PERSON_NAME_GENERATION_VERSION).rotate_left(47);
    let given_draw = mix64(domain ^ 0x4749_5645_4E4E_414D);
    let family_draw = mix64(domain ^ 0x4641_4D49_4C59_4E4D);
    format!(
        "{} {}",
        PERSON_GIVEN_NAMES[given_draw as usize % PERSON_GIVEN_NAMES.len()],
        PERSON_FAMILY_NAMES[family_draw as usize % PERSON_FAMILY_NAMES.len()]
    )
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalogue_provides_thousands_of_combinations() {
        assert!(PERSON_GIVEN_NAMES.len() >= 64);
        assert!(PERSON_FAMILY_NAMES.len() >= 64);
        assert!(PERSON_GIVEN_NAMES.len() * PERSON_FAMILY_NAMES.len() >= 4_096);
        let generated = (0_u64..512)
            .map(|value| generated_person_name(value, value.rotate_left(17)))
            .collect::<HashSet<_>>();
        assert!(generated.len() >= 480);
    }

    #[test]
    fn generation_is_stable_and_iso_646_safe() {
        let first = generated_person_name(0x1234_5678, 9);
        assert_eq!(first, generated_person_name(0x1234_5678, 9));
        assert!(first.is_ascii());
        assert!(first.contains(' '));
    }
}
