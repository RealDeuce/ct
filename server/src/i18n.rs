//! Connection language negotiation and server-side message formatting.

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use language_tags::LanguageTag;
use thiserror::Error;
use unic_langid::langid;

pub const SUPPORTED_LANGUAGE_TAGS: &[&str] = &["en"];
const ENGLISH_MESSAGES: &str = include_str!("../i18n/en.ftl");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NegotiatedLanguage(&'static str);

impl NegotiatedLanguage {
    pub fn tag(&self) -> &'static str {
        self.0
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
        let mut bundle = FluentBundle::new(vec![langid!("en")]);
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
    NegotiatedLanguage("en")
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
    if requested.is_empty() || requested.len() > 128 || LanguageTag::parse(requested).is_err() {
        return Err(LanguageNegotiationError::Malformed);
    }

    let mut candidate = requested.to_ascii_lowercase();
    loop {
        if let Some(supported) = SUPPORTED_LANGUAGE_TAGS
            .iter()
            .copied()
            .find(|supported| *supported == candidate)
        {
            return Ok(NegotiatedLanguage(supported));
        }
        let Some(separator) = candidate.rfind('-') else {
            break;
        };
        candidate.truncate(separator);
        if candidate
            .rsplit_once('-')
            .is_some_and(|(_, subtag)| subtag.len() == 1)
        {
            candidate.truncate(candidate.rfind('-').unwrap_or(0));
        }
    }
    Err(LanguageNegotiationError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_exact_and_regional_english() {
        assert_eq!(negotiate_language("en").unwrap().tag(), "en");
        assert_eq!(negotiate_language("en-US").unwrap().tag(), "en");
        assert_eq!(negotiate_language("EN-latn-us").unwrap().tag(), "en");
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
        assert!(matches!(
            language.text("missing"),
            Err(LocalizationError::MissingMessage(_))
        ));
    }
}
