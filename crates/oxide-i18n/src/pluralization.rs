//! Pluralization rules for different languages
//!
//! Implements CLDR plural rules for determining which plural form to use
//! based on a numeric value.

use serde::{Deserialize, Serialize};

/// Plural categories as defined by CLDR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluralCategory {
    /// Used for 0 in some languages
    Zero,
    /// Used for 1 in most languages
    One,
    /// Used for 2 in some languages (e.g., Arabic)
    Two,
    /// Used for small numbers in some languages (e.g., Polish)
    Few,
    /// Used for larger numbers in some languages (e.g., Arabic)
    Many,
    /// Default/other form
    Other,
}

impl PluralCategory {
    /// Get the category name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::One => "one",
            Self::Two => "two",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }
}

impl std::fmt::Display for PluralCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Plural rules for a language
pub trait PluralRules {
    /// Get the plural category for an integer value
    fn select_int(&self, n: i64) -> PluralCategory;

    /// Get the plural category for a floating-point value
    fn select_float(&self, n: f64) -> PluralCategory;

    /// Get all plural categories used by this language
    fn categories(&self) -> &[PluralCategory];
}

/// English plural rules (and other Germanic languages)
///
/// - one: n = 1
/// - other: everything else
#[derive(Debug, Clone, Default)]
pub struct EnglishRules;

impl PluralRules for EnglishRules {
    fn select_int(&self, n: i64) -> PluralCategory {
        if n == 1 {
            PluralCategory::One
        } else {
            PluralCategory::Other
        }
    }

    fn select_float(&self, n: f64) -> PluralCategory {
        if (n - 1.0).abs() < f64::EPSILON {
            PluralCategory::One
        } else {
            PluralCategory::Other
        }
    }

    fn categories(&self) -> &[PluralCategory] {
        &[PluralCategory::One, PluralCategory::Other]
    }
}

/// French plural rules
///
/// - one: n = 0 or n = 1
/// - other: everything else
#[derive(Debug, Clone, Default)]
pub struct FrenchRules;

impl PluralRules for FrenchRules {
    fn select_int(&self, n: i64) -> PluralCategory {
        if n == 0 || n == 1 {
            PluralCategory::One
        } else {
            PluralCategory::Other
        }
    }

    fn select_float(&self, n: f64) -> PluralCategory {
        let abs_n = n.abs();
        if abs_n < 2.0 {
            PluralCategory::One
        } else {
            PluralCategory::Other
        }
    }

    fn categories(&self) -> &[PluralCategory] {
        &[PluralCategory::One, PluralCategory::Other]
    }
}

/// Arabic plural rules
///
/// - zero: n = 0
/// - one: n = 1
/// - two: n = 2
/// - few: n % 100 in 3..10
/// - many: n % 100 in 11..99
/// - other: everything else
#[derive(Debug, Clone, Default)]
pub struct ArabicRules;

impl PluralRules for ArabicRules {
    fn select_int(&self, n: i64) -> PluralCategory {
        let n = n.abs();
        let n100 = n % 100;

        if n == 0 {
            PluralCategory::Zero
        } else if n == 1 {
            PluralCategory::One
        } else if n == 2 {
            PluralCategory::Two
        } else if (3..=10).contains(&n100) {
            PluralCategory::Few
        } else if (11..=99).contains(&n100) {
            PluralCategory::Many
        } else {
            PluralCategory::Other
        }
    }

    fn select_float(&self, n: f64) -> PluralCategory {
        self.select_int(n.floor() as i64)
    }

    fn categories(&self) -> &[PluralCategory] {
        &[
            PluralCategory::Zero,
            PluralCategory::One,
            PluralCategory::Two,
            PluralCategory::Few,
            PluralCategory::Many,
            PluralCategory::Other,
        ]
    }
}

/// Polish plural rules
///
/// - one: n = 1
/// - few: n % 10 in 2..4 and n % 100 not in 12..14
/// - many: n != 1 and n % 10 in 0..1, or n % 10 in 5..9, or n % 100 in 12..14
/// - other: everything else (decimals)
#[derive(Debug, Clone, Default)]
pub struct PolishRules;

impl PluralRules for PolishRules {
    fn select_int(&self, n: i64) -> PluralCategory {
        let n = n.abs();
        let n10 = n % 10;
        let n100 = n % 100;

        if n == 1 {
            PluralCategory::One
        } else if (2..=4).contains(&n10) && !(12..=14).contains(&n100) {
            PluralCategory::Few
        } else if n != 1
            && ((0..=1).contains(&n10)
                || (5..=9).contains(&n10)
                || (12..=14).contains(&n100))
        {
            PluralCategory::Many
        } else {
            PluralCategory::Other
        }
    }

    fn select_float(&self, n: f64) -> PluralCategory {
        // For non-integer values, use "other"
        if n.fract() != 0.0 {
            PluralCategory::Other
        } else {
            self.select_int(n as i64)
        }
    }

    fn categories(&self) -> &[PluralCategory] {
        &[
            PluralCategory::One,
            PluralCategory::Few,
            PluralCategory::Many,
            PluralCategory::Other,
        ]
    }
}

/// Russian plural rules (also applies to Ukrainian, Belarusian, etc.)
///
/// - one: n % 10 = 1 and n % 100 != 11
/// - few: n % 10 in 2..4 and n % 100 not in 12..14
/// - many: n % 10 = 0 or n % 10 in 5..9 or n % 100 in 11..14
/// - other: everything else
#[derive(Debug, Clone, Default)]
pub struct RussianRules;

impl PluralRules for RussianRules {
    fn select_int(&self, n: i64) -> PluralCategory {
        let n = n.abs();
        let n10 = n % 10;
        let n100 = n % 100;

        if n10 == 1 && n100 != 11 {
            PluralCategory::One
        } else if (2..=4).contains(&n10) && !(12..=14).contains(&n100) {
            PluralCategory::Few
        } else if n10 == 0 || (5..=9).contains(&n10) || (11..=14).contains(&n100) {
            PluralCategory::Many
        } else {
            PluralCategory::Other
        }
    }

    fn select_float(&self, n: f64) -> PluralCategory {
        if n.fract() != 0.0 {
            PluralCategory::Other
        } else {
            self.select_int(n as i64)
        }
    }

    fn categories(&self) -> &[PluralCategory] {
        &[
            PluralCategory::One,
            PluralCategory::Few,
            PluralCategory::Many,
            PluralCategory::Other,
        ]
    }
}

/// Chinese/Japanese/Korean plural rules (no pluralization)
///
/// - other: always
#[derive(Debug, Clone, Default)]
pub struct CjkRules;

impl PluralRules for CjkRules {
    fn select_int(&self, _n: i64) -> PluralCategory {
        PluralCategory::Other
    }

    fn select_float(&self, _n: f64) -> PluralCategory {
        PluralCategory::Other
    }

    fn categories(&self) -> &[PluralCategory] {
        &[PluralCategory::Other]
    }
}

/// Get plural rules for a language code
pub fn get_plural_rules(language: &str) -> Box<dyn PluralRules + Send + Sync> {
    match language.to_lowercase().as_str() {
        // English-like (1 vs other)
        "en" | "de" | "nl" | "it" | "es" | "pt" | "sv" | "da" | "no" | "fi" | "el" | "hu"
        | "tr" | "bg" | "et" | "lv" | "lt" | "ca" | "eu" | "gl" => Box::new(EnglishRules),

        // French-like (0-1 vs other)
        "fr" | "pt-br" => Box::new(FrenchRules),

        // Arabic
        "ar" => Box::new(ArabicRules),

        // Polish
        "pl" => Box::new(PolishRules),

        // Russian/Slavic
        "ru" | "uk" | "be" | "sr" | "hr" | "bs" | "me" => Box::new(RussianRules),

        // CJK (no pluralization)
        "zh" | "ja" | "ko" | "vi" | "th" | "id" | "ms" => Box::new(CjkRules),

        // Default to English-like rules
        _ => Box::new(EnglishRules),
    }
}

/// Helper to select plural category with any numeric type
pub fn select_plural<T: Into<f64>>(rules: &dyn PluralRules, n: T) -> PluralCategory {
    let n = n.into();
    if n.fract() == 0.0 {
        rules.select_int(n as i64)
    } else {
        rules.select_float(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_rules() {
        let rules = EnglishRules;
        assert_eq!(rules.select_int(0), PluralCategory::Other);
        assert_eq!(rules.select_int(1), PluralCategory::One);
        assert_eq!(rules.select_int(2), PluralCategory::Other);
        assert_eq!(rules.select_int(5), PluralCategory::Other);
        assert_eq!(rules.select_int(21), PluralCategory::Other);
    }

    #[test]
    fn test_french_rules() {
        let rules = FrenchRules;
        assert_eq!(rules.select_int(0), PluralCategory::One);
        assert_eq!(rules.select_int(1), PluralCategory::One);
        assert_eq!(rules.select_int(2), PluralCategory::Other);
        assert_eq!(rules.select_int(5), PluralCategory::Other);
    }

    #[test]
    fn test_arabic_rules() {
        let rules = ArabicRules;
        assert_eq!(rules.select_int(0), PluralCategory::Zero);
        assert_eq!(rules.select_int(1), PluralCategory::One);
        assert_eq!(rules.select_int(2), PluralCategory::Two);
        assert_eq!(rules.select_int(3), PluralCategory::Few);
        assert_eq!(rules.select_int(10), PluralCategory::Few);
        assert_eq!(rules.select_int(11), PluralCategory::Many);
        assert_eq!(rules.select_int(99), PluralCategory::Many);
        assert_eq!(rules.select_int(100), PluralCategory::Other);
    }

    #[test]
    fn test_polish_rules() {
        let rules = PolishRules;
        assert_eq!(rules.select_int(1), PluralCategory::One);
        assert_eq!(rules.select_int(2), PluralCategory::Few);
        assert_eq!(rules.select_int(4), PluralCategory::Few);
        assert_eq!(rules.select_int(5), PluralCategory::Many);
        assert_eq!(rules.select_int(12), PluralCategory::Many);
        assert_eq!(rules.select_int(22), PluralCategory::Few);
    }

    #[test]
    fn test_russian_rules() {
        let rules = RussianRules;
        assert_eq!(rules.select_int(1), PluralCategory::One);
        assert_eq!(rules.select_int(2), PluralCategory::Few);
        assert_eq!(rules.select_int(5), PluralCategory::Many);
        assert_eq!(rules.select_int(11), PluralCategory::Many);
        assert_eq!(rules.select_int(21), PluralCategory::One);
        assert_eq!(rules.select_int(22), PluralCategory::Few);
    }

    #[test]
    fn test_cjk_rules() {
        let rules = CjkRules;
        assert_eq!(rules.select_int(0), PluralCategory::Other);
        assert_eq!(rules.select_int(1), PluralCategory::Other);
        assert_eq!(rules.select_int(100), PluralCategory::Other);
    }
}
