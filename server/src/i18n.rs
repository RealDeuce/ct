//! Connection language negotiation and server-side message formatting.

use fluent_bundle::types::FluentValue;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use language_tags::LanguageTag;
use thiserror::Error;
use unic_langid::langid;

pub const SUPPORTED_LANGUAGE_TAGS: &[&str] = &["en"];
const ENGLISH_MESSAGES: &str = include_str!("../i18n/en.ftl");

pub const DEFAULT_LANGUAGE_TAG: &str = "en-US";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisplayFormatting {
    pub decimal_separator: &'static str,
    pub grouping_separator: &'static str,
    pub primary_grouping_digits: u8,
    pub secondary_grouping_digits: u8,
    pub game_timestamp_pattern: &'static str,
    pub game_duration_pattern: &'static str,
    pub real_duration_pattern: &'static str,
}

const ENGLISH_DISPLAY_FORMATTING: DisplayFormatting = DisplayFormatting {
    decimal_separator: ".",
    grouping_separator: ",",
    primary_grouping_digits: 3,
    secondary_grouping_digits: 3,
    game_timestamp_pattern: "Day {day}, {hour}:{minute}:{second}",
    game_duration_pattern: "{day} d {hour}:{minute}:{second}",
    real_duration_pattern: "{hour}:{minute}:{second}",
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NegotiatedLanguage(String);

impl NegotiatedLanguage {
    pub fn tag(&self) -> &str {
        &self.0
    }

    pub fn display_formatting(&self) -> DisplayFormatting {
        ENGLISH_DISPLAY_FORMATTING
    }

    pub fn text(&self, key: &str) -> Result<String, LocalizationError> {
        self.format(key, None)
    }

    pub fn format(
        &self,
        key: &str,
        arguments: Option<&FluentArgs<'_>>,
    ) -> Result<String, LocalizationError> {
        let resource = FluentResource::try_new(ENGLISH_MESSAGES.to_owned())
            .map_err(|(_, errors)| LocalizationError::Resource(format!("{errors:?}")))?;
        let mut bundle = FluentBundle::new(vec![langid!("en-US")]);
        // Door terminals cannot reliably render Fluent's Unicode bidi
        // isolation marks. The installed language is left-to-right, so keep
        // interpolated arguments plain on the wire.
        bundle.set_use_isolating(false);
        bundle.set_formatter(Some(|value, _| match value {
            FluentValue::Number(number) if number.options.use_grouping => {
                Some(group_english_number(&number.as_string()))
            }
            _ => None,
        }));
        bundle
            .add_resource(resource)
            .map_err(|errors| LocalizationError::Resource(format!("{errors:?}")))?;
        let message = bundle
            .get_message(key)
            .ok_or_else(|| LocalizationError::MissingMessage(key.to_owned()))?;
        let pattern = message
            .value()
            .ok_or_else(|| LocalizationError::MissingValue(key.to_owned()))?;
        let mut errors = Vec::new();
        let rendered = bundle
            .format_pattern(pattern, arguments, &mut errors)
            .into_owned();
        if errors.is_empty() {
            Ok(rendered)
        } else {
            Err(LocalizationError::Format(format!("{errors:?}")))
        }
    }
}

pub fn default_language() -> NegotiatedLanguage {
    NegotiatedLanguage(DEFAULT_LANGUAGE_TAG.to_owned())
}

fn group_english_number(value: &str) -> String {
    let (sign, unsigned) = value
        .strip_prefix('-')
        .map_or(("", value), |unsigned| ("-", unsigned));
    let integer_end = unsigned.find(['.', 'e', 'E']).unwrap_or(unsigned.len());
    let integer = &unsigned[..integer_end];
    if integer.len() <= 3 || !integer.bytes().all(|byte| byte.is_ascii_digit()) {
        return value.to_owned();
    }
    let mut result = String::with_capacity(value.len() + integer.len() / 3);
    result.push_str(sign);
    let first_group = integer.len() % 3;
    let mut offset = 0;
    if first_group != 0 {
        result.push_str(&integer[..first_group]);
        offset = first_group;
    }
    while offset < integer.len() {
        if !result.is_empty() && result != sign {
            result.push(',');
        }
        result.push_str(&integer[offset..offset + 3]);
        offset += 3;
    }
    result.push_str(&unsigned[integer_end..]);
    result
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum LanguageNegotiationError {
    #[error("malformed BCP 47 language tag")]
    Malformed,
    #[error("unsupported language tag")]
    Unsupported,
}

#[derive(Debug, Error)]
pub enum LocalizationError {
    #[error("invalid localization resource: {0}")]
    Resource(String),
    #[error("missing localization message {0}")]
    MissingMessage(String),
    #[error("localization message {0} has no value")]
    MissingValue(String),
    #[error("localization formatting failed: {0}")]
    Format(String),
}

pub fn negotiate_language(requested: &str) -> Result<NegotiatedLanguage, LanguageNegotiationError> {
    if requested.is_empty() || requested.len() > 128 {
        return Err(LanguageNegotiationError::Malformed);
    }

    let parsed = LanguageTag::parse(requested).map_err(|_| LanguageNegotiationError::Malformed)?;
    if parsed.primary_language() == "en" {
        return Ok(if parsed.as_str() == "en" {
            default_language()
        } else {
            NegotiatedLanguage(parsed.into_string())
        });
    }
    Err(LanguageNegotiationError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_exact_and_regional_english() {
        assert_eq!(negotiate_language("en").unwrap().tag(), "en-US");
        assert_eq!(negotiate_language("en-US").unwrap().tag(), "en-US");
        assert_eq!(negotiate_language("en-GB").unwrap().tag(), "en-GB");
        assert_eq!(
            negotiate_language("EN-latn-us").unwrap().tag(),
            "en-Latn-US"
        );
    }

    #[test]
    fn distinguishes_malformed_and_unsupported_tags() {
        assert_eq!(
            negotiate_language("not_a_tag"),
            Err(LanguageNegotiationError::Malformed)
        );
        assert_eq!(
            negotiate_language("fr-CA"),
            Err(LanguageNegotiationError::Unsupported)
        );
    }

    #[test]
    fn formats_fluent_messages() {
        let language = negotiate_language("en").unwrap();
        let mut arguments = FluentArgs::new();
        arguments.set("languageTag", "fr");
        assert!(
            language
                .format("unsupported-language", Some(&arguments))
                .unwrap()
                .contains("fr")
        );
        arguments.set("protocol", "CT-RPC");
        arguments.set("clientVersion", 1234_u64);
        arguments.set("serverVersion", 4_u64);
        assert!(
            language
                .format("unsupported-version", Some(&arguments))
                .unwrap()
                .contains("1,234")
        );
        assert!(matches!(
            language.text("missing"),
            Err(LocalizationError::MissingMessage(_))
        ));
        assert_eq!(group_english_number("1234567"), "1,234,567");
        assert_eq!(group_english_number("-1234567.25"), "-1,234,567.25");
        assert_eq!(language.display_formatting(), ENGLISH_DISPLAY_FORMATTING);
    }
}
