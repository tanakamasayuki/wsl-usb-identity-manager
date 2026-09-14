//! Identity key formatting.
//!
//! Keys follow the `<variant>-<unique-id>` shape that board-identify settled on
//! (requirements §4.5). They are internal to this application; no compatibility
//! with other tools is claimed.
//!
//! **These keys are persistence keys.** Requirement R4.10 makes the formatting
//! deterministic and forbids changing it after a release — a changed rule would
//! orphan every stored device. Change it only alongside a schema version bump
//! and a migration (§7.4). The tests below pin the exact output for that reason.

use std::fmt;

/// The shortest unique id worth storing. Anything shorter is more likely to
/// collide than to identify (requirements §4.5).
const MIN_UNIQUE_ID_LEN: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityKeyError {
    /// The unique id had fewer than [`MIN_UNIQUE_ID_LEN`] usable characters.
    UniqueIdTooShort { got: String },
    /// The variant name was empty once normalised.
    EmptyVariant,
}

impl fmt::Display for IdentityKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UniqueIdTooShort { got } => write!(
                f,
                "unique id {got:?} has fewer than {MIN_UNIQUE_ID_LEN} usable characters"
            ),
            Self::EmptyVariant => f.write_str("variant name is empty"),
        }
    }
}

impl std::error::Error for IdentityKeyError {}

/// Builds an identity key from a chip variant and a unique id.
///
/// ```
/// # use wuim_probe::identity::identity_key;
/// assert_eq!(identity_key("esp32-s3", "7C:DF:A1:12:34:56").unwrap(), "esp32-s3-7cdfa1123456");
/// ```
pub fn identity_key(variant: &str, unique_id: &str) -> Result<String, IdentityKeyError> {
    let variant = normalize_variant(variant);
    if variant.is_empty() {
        return Err(IdentityKeyError::EmptyVariant);
    }
    let unique = normalize_unique_id(unique_id);
    if unique.len() < MIN_UNIQUE_ID_LEN {
        return Err(IdentityKeyError::UniqueIdTooShort {
            got: unique_id.to_owned(),
        });
    }
    Ok(format!("{variant}-{unique}"))
}

/// Lowercase ASCII, `-` as the only separator, no leading or trailing separator.
fn normalize_variant(variant: &str) -> String {
    let mut out = String::with_capacity(variant.len());
    for ch in variant.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

/// Lowercase ASCII alphanumerics only; separators in the source value
/// (the colons of a MAC address, for instance) are dropped.
fn normalize_unique_id(unique_id: &str) -> String {
    unique_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Turns espflash's chip name into the variant form board-identify uses.
///
/// espflash renders `Chip::Esp32s3` as `esp32s3`; the convention here puts a
/// separator before the series, giving `esp32-s3`. Deriving it from the name
/// rather than from a lookup table means a chip added to espflash later gets
/// the same treatment without a table update — and, crucially, without the key
/// for an existing chip ever changing (R4.10).
pub fn esp_variant(chip_name: &str) -> String {
    let lower = chip_name.to_ascii_lowercase();
    match lower.strip_prefix("esp32") {
        Some("") | None => lower,
        Some(series) => format!("esp32-{}", series.trim_start_matches('-')),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact strings from requirements §4.5. These are persistence keys:
    /// if this test needs changing, the schema version has to move with it.
    #[test]
    fn matches_the_documented_examples() {
        assert_eq!(
            identity_key("esp32-s3", "7cdfa1123456").unwrap(),
            "esp32-s3-7cdfa1123456"
        );
        assert_eq!(
            identity_key("ch32x035c8t6", "1ff9abcd880ebc48").unwrap(),
            "ch32x035c8t6-1ff9abcd880ebc48"
        );
        assert_eq!(
            identity_key("wch-link", "fc928f068181").unwrap(),
            "wch-link-fc928f068181"
        );
    }

    #[test]
    fn strips_separators_from_the_unique_id() {
        assert_eq!(
            identity_key("esp32-s3", "7C:DF:A1:12:34:56").unwrap(),
            "esp32-s3-7cdfa1123456"
        );
    }

    #[test]
    fn is_deterministic_across_input_spellings() {
        let a = identity_key("ESP32-S3", "7C:DF:A1:12:34:56").unwrap();
        let b = identity_key("esp32 s3", "7cdfa1123456").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn rejects_a_unique_id_that_is_too_short() {
        assert_eq!(
            identity_key("esp32", "12:34"),
            Err(IdentityKeyError::UniqueIdTooShort {
                got: "12:34".into()
            })
        );
    }

    #[test]
    fn rejects_an_empty_variant() {
        assert_eq!(
            identity_key("///", "7cdfa1123456"),
            Err(IdentityKeyError::EmptyVariant)
        );
    }

    #[test]
    fn never_emits_a_slash_or_nul() {
        let key = identity_key("esp32/s3\u{0}", "7cdf/a112\u{0}3456").unwrap();
        assert!(!key.contains('/'));
        assert!(!key.contains('\0'));
    }

    #[test]
    fn esp_variant_inserts_the_series_separator() {
        assert_eq!(esp_variant("esp32"), "esp32");
        assert_eq!(esp_variant("esp32s3"), "esp32-s3");
        assert_eq!(esp_variant("esp32c61"), "esp32-c61");
        assert_eq!(esp_variant("ESP32-S2"), "esp32-s2");
        // An unrelated name is left alone rather than forced into the pattern.
        assert_eq!(esp_variant("ch32x035"), "ch32x035");
    }
}
