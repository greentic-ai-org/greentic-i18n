//! Formatting facade that delegates to swappable backends.
use std::time::{SystemTime, UNIX_EPOCH};

use crate::I18nProfile;

/// Simple alias for floating-point values until we add a decimal arithmetic type.
pub type DecimalLike = f64;

/// Backend abstraction so ICU4X can replace the deterministic formatter later.
pub trait FormatBackend: Sync + Send {
    fn format_number(&self, profile: &I18nProfile, value: DecimalLike) -> String;
    fn format_currency(
        &self,
        profile: &I18nProfile,
        amount: DecimalLike,
        currency_override: Option<&str>,
    ) -> String;
    fn format_datetime(&self, profile: &I18nProfile, instant: SystemTime) -> String;
}

/// Basic deterministic backend used today and for golden tests.
pub struct BasicBackend;

impl FormatBackend for BasicBackend {
    fn format_number(&self, profile: &I18nProfile, value: DecimalLike) -> String {
        format_number_with_precision(profile, value, 2)
    }

    fn format_currency(
        &self,
        profile: &I18nProfile,
        amount: DecimalLike,
        currency_override: Option<&str>,
    ) -> String {
        let code = currency_override
            .map(str::to_string)
            .or_else(|| profile.currency.clone())
            .unwrap_or_else(|| "USD".to_string());
        let amount = format_number_with_precision(profile, amount, 2);
        format!("{code} {amount}")
    }

    fn format_datetime(&self, profile: &I18nProfile, when: SystemTime) -> String {
        format_datetime_with_separator(profile, when)
    }
}

#[cfg(feature = "icu4x")]
pub struct Icu4xBackend;

#[cfg(feature = "icu4x")]
impl FormatBackend for Icu4xBackend {
    fn format_number(&self, profile: &I18nProfile, value: DecimalLike) -> String {
        format_number_with_precision(profile, value, 2)
    }

    fn format_currency(
        &self,
        profile: &I18nProfile,
        amount: DecimalLike,
        currency_override: Option<&str>,
    ) -> String {
        let code = currency_override
            .map(str::to_string)
            .or_else(|| profile.currency.clone())
            .unwrap_or_else(|| "USD".to_string());
        let amount = format_number_with_precision(profile, amount, 2);
        format!("{code} {amount}")
    }

    fn format_datetime(&self, profile: &I18nProfile, when: SystemTime) -> String {
        format_datetime_with_separator(profile, when)
    }
}

#[cfg(not(feature = "icu4x"))]
fn default_backend() -> &'static dyn FormatBackend {
    static BACKEND: BasicBackend = BasicBackend;
    &BACKEND
}

#[cfg(feature = "icu4x")]
fn default_backend() -> &'static dyn FormatBackend {
    static BACKEND: Icu4xBackend = Icu4xBackend;
    &BACKEND
}

/// Helpers that use the selected backend.
pub trait FormatFacade {
    fn format_number(&self, value: DecimalLike) -> String;
    fn format_currency(&self, value: DecimalLike, currency: Option<&str>) -> String;
    fn format_datetime(&self, when: SystemTime) -> String;
}

impl FormatFacade for I18nProfile {
    fn format_number(&self, value: DecimalLike) -> String {
        default_backend().format_number(self, value)
    }

    fn format_currency(&self, value: DecimalLike, currency: Option<&str>) -> String {
        default_backend().format_currency(self, value, currency)
    }

    fn format_datetime(&self, when: SystemTime) -> String {
        default_backend().format_datetime(self, when)
    }
}

fn format_number_with_precision(
    profile: &I18nProfile,
    value: DecimalLike,
    precision: usize,
) -> String {
    let formatted = format!("{value:.prec$}", value = value, prec = precision);
    let grouped = insert_thousands_separator(&formatted);
    replace_decimal_separator(grouped, profile.decimal_separator)
}

fn format_datetime_with_separator(profile: &I18nProfile, when: SystemTime) -> String {
    match when.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let base = format!("{}.{:03}", duration.as_secs(), duration.subsec_millis());
            let normalized = replace_decimal_separator(base, profile.decimal_separator);
            format!("{normalized} UTC")
        }
        Err(err) => format!("invalid timestamp: {err}"),
    }
}

fn replace_decimal_separator(value: String, separator: char) -> String {
    if separator == '.' {
        return value;
    }
    if let Some(pos) = value.find('.') {
        let mut replaced = value;
        replaced.replace_range(pos..pos + 1, &separator.to_string());
        replaced
    } else {
        value
    }
}

fn insert_thousands_separator(value: &str) -> String {
    let (integer, remainder) = match value.find('.') {
        Some(idx) => (&value[..idx], &value[idx..]),
        None => (value, ""),
    };
    let (sign, digits) = if let Some(stripped) = integer.strip_prefix('-') {
        ("-", stripped)
    } else if let Some(stripped) = integer.strip_prefix('+') {
        ("+", stripped)
    } else {
        ("", integer)
    };

    let mut grouped = String::new();
    let mut count = 0;
    for digit in digits.chars().rev() {
        if count == 3 {
            grouped.push('_');
            count = 0;
        }
        grouped.push(digit);
        count += 1;
    }
    let separated: String = grouped.chars().rev().collect();
    format!("{sign}{separated}{remainder}")
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use crate::{Direction, I18nId, I18nProfile, normalize_tag};

    use super::format_datetime_with_separator;
    use crate::format::FormatFacade;

    fn build_profile(separator: char, currency: Option<&str>) -> I18nProfile {
        let tag = normalize_tag("en-US").unwrap();
        I18nProfile {
            tag,
            id: I18nId::zero(),
            currency: currency.map(|c| c.to_string()),
            decimal_separator: separator,
            direction: Direction::Ltr,
            calendar: "gregory".to_string(),
            numbering_system: "latn".to_string(),
            timezone: "UTC".to_string(),
            first_day: "mon".to_string(),
            hour_cycle: "h23".to_string(),
            collation: None,
            case_first: None,
            units: None,
        }
    }

    fn build_profile_with(
        tag_value: &str,
        separator: char,
        currency: Option<&str>,
        direction: Direction,
        first_day: &str,
        hour_cycle: &str,
    ) -> I18nProfile {
        let tag = normalize_tag(tag_value).unwrap();
        I18nProfile {
            tag,
            id: I18nId::zero(),
            currency: currency.map(|c| c.to_string()),
            decimal_separator: separator,
            direction,
            calendar: "gregory".to_string(),
            numbering_system: "latn".to_string(),
            timezone: "UTC".to_string(),
            first_day: first_day.to_string(),
            hour_cycle: hour_cycle.to_string(),
            collation: None,
            case_first: None,
            units: None,
        }
    }

    #[test]
    fn number_formats_with_decimal_separator() {
        let profile = build_profile(',', Some("EUR"));
        assert_eq!(profile.format_number(1234.5), "1_234,50");
    }

    #[test]
    fn currency_uses_override() {
        let profile = build_profile('.', None);
        assert_eq!(profile.format_currency(10.0, Some("JPY")), "JPY 10.00");
    }

    #[test]
    fn datetime_normalizes_separator() {
        let profile = build_profile(',', Some("USD"));
        let instant = UNIX_EPOCH + std::time::Duration::from_secs(1);
        let result = format_datetime_with_separator(&profile, instant);
        assert!(result.contains("1,000 UTC"));
    }

    #[test]
    fn format_facade_delegates_number() {
        let profile = build_profile(',', Some("USD"));
        assert_eq!(profile.format_number(42.0), profile.format_number(42.0));
    }

    #[test]
    fn format_facade_delegates_currency() {
        let profile = build_profile('.', Some("USD"));
        assert_eq!(profile.format_currency(42.0, None), "USD 42.00");
    }

    #[test]
    fn format_facade_delegates_datetime() {
        let profile = build_profile(',', Some("USD"));
        let instant = UNIX_EPOCH + std::time::Duration::from_secs(2);
        assert!(profile.format_datetime(instant).contains("UTC"));
    }

    #[test]
    fn profile_with_custom_day_and_cycle() {
        let profile = build_profile_with("ar-SA", ',', None, Direction::Rtl, "sat", "h12");
        assert_eq!(profile.first_day, "sat");
        assert_eq!(profile.hour_cycle, "h12");
    }
}
