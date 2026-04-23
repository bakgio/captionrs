use regex::Regex;
use std::sync::OnceLock;

/// Regex patterns for subtitle processing
pub struct RegexPatterns;

impl RegexPatterns {
    pub const TAGS: &'static str = r"[<{][/\\]?[a-z0-9.]+[}>]";
    pub const POSITION_TAGS: &'static str = r"^\{\\an[0-9]\}";
    // Speaker timestamp guarding and bleep preservation are handled in processor logic because
    // the regex engine cannot express those checks directly.

    pub const FRONT_NOTES: &'static str = r"(?:♪+\s+)";
    pub const BACK_NOTES: &'static str = r"(?:\s+♪+)";

    pub const DESCRIPTION_BRACKET: &'static str = r"\[(?:[^\]]|\s)*\]";
    pub const DESCRIPTION_PARENTHESES: &'static str = r"\((?:[^\)]|\s)*\)";

    pub const INLINE_DESCRIPTION: &'static str =
        r"(?:<[a-z]+>)?[\[\(][A-Za-z ]+[)\]](?:</[a-z]+>)?";
}

/// Constructs complex regex patterns
pub struct ComputedPatterns;

impl ComputedPatterns {
    pub fn front_optional_tags_with_hyphen() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| {
            format!(
                r"^\s*({})??\s*(-)??\s*({})??\s*",
                RegexPatterns::TAGS,
                RegexPatterns::TAGS
            )
        })
    }

    pub fn speaker() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| {
            format!(
                r"({})\s*(Mc[A-Z][a-zA-Z]+|[A-Z0-9\&\[\]\.#\' ]+\s*|[A-Z][a-z]+):\s*",
                Self::front_optional_tags_with_hyphen()
            )
        })
    }

    pub fn speaker_parentheses() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| {
            format!(
                r"({})\s*(?:[A-Z0-9\&\[\]\.#\' ]+\s*|[A-Z][a-z]+)(?: \([a-zA-Z ]+\)):\s*",
                Self::front_optional_tags_with_hyphen()
            )
        })
    }

    pub fn full_line_description_bracket() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| {
            format!(
                r"^-?\s*{}?\[[^\]]+\]{}?$",
                RegexPatterns::FRONT_NOTES,
                RegexPatterns::BACK_NOTES
            )
        })
    }

    pub fn new_line_description_bracket() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| {
            format!(
                r"^(?:{})?-?\s*{}?{}(?:{})?{}?$",
                RegexPatterns::TAGS,
                RegexPatterns::FRONT_NOTES,
                RegexPatterns::DESCRIPTION_BRACKET,
                RegexPatterns::TAGS,
                RegexPatterns::BACK_NOTES
            )
        })
    }

    pub fn front_description_bracket() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| {
            format!(
                r"^(?:{}|{})?({}){}:?",
                Self::speaker(),
                Self::speaker_parentheses(),
                Self::front_optional_tags_with_hyphen(),
                RegexPatterns::DESCRIPTION_BRACKET
            )
        })
    }

    pub fn end_description_bracket() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| format!(r"\s*{}\s*$", RegexPatterns::DESCRIPTION_BRACKET))
    }

    pub fn full_line_description_parentheses() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| {
            format!(
                r"^-?\s*{}?\([^\)]+\){}?$",
                RegexPatterns::FRONT_NOTES,
                RegexPatterns::BACK_NOTES
            )
        })
    }

    pub fn new_line_description_parentheses() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| {
            format!(
                r"^(?:{})?-?\s*{}?{}{}?(?:{})?$",
                RegexPatterns::TAGS,
                RegexPatterns::FRONT_NOTES,
                RegexPatterns::DESCRIPTION_PARENTHESES,
                RegexPatterns::BACK_NOTES,
                RegexPatterns::TAGS
            )
        })
    }

    pub fn front_description_parentheses() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| {
            format!(
                r"^({})(?:{}|{})?{}:?",
                Self::front_optional_tags_with_hyphen(),
                Self::speaker(),
                Self::speaker_parentheses(),
                RegexPatterns::DESCRIPTION_PARENTHESES
            )
        })
    }

    pub fn end_description_parentheses() -> &'static str {
        static PATTERN: OnceLock<String> = OnceLock::new();
        PATTERN.get_or_init(|| format!(r"\s*{}:?\s*$", RegexPatterns::DESCRIPTION_PARENTHESES))
    }
}

/// Compiled regex cache for efficient access to all SDH processing patterns
#[derive(Clone)]
pub struct CompiledRegexes {
    pub tags: Regex,
    pub position_tags: Regex,
    pub speaker: Regex,
    pub speaker_parentheses: Regex,
    pub full_line_description_bracket: Regex,
    pub full_line_description_parentheses: Regex,
    pub new_line_description_bracket: Regex,
    pub new_line_description_parentheses: Regex,
    pub front_description_bracket: Regex,
    pub front_description_parentheses: Regex,
    pub end_description_bracket: Regex,
    pub end_description_parentheses: Regex,
    pub inline_description: Regex,
}

impl CompiledRegexes {
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            tags: Regex::new(RegexPatterns::TAGS)?,
            position_tags: Regex::new(RegexPatterns::POSITION_TAGS)?,
            speaker: Regex::new(&format!(r"(?m){}", ComputedPatterns::speaker()))?,
            speaker_parentheses: Regex::new(&format!(
                r"(?m){}",
                ComputedPatterns::speaker_parentheses()
            ))?,
            full_line_description_bracket: Regex::new(
                ComputedPatterns::full_line_description_bracket(),
            )?,
            full_line_description_parentheses: Regex::new(
                ComputedPatterns::full_line_description_parentheses(),
            )?,
            new_line_description_bracket: Regex::new(&format!(
                r"(?m){}",
                ComputedPatterns::new_line_description_bracket()
            ))?,
            new_line_description_parentheses: Regex::new(&format!(
                r"(?m){}",
                ComputedPatterns::new_line_description_parentheses()
            ))?,
            front_description_bracket: Regex::new(&format!(
                r"(?m){}",
                ComputedPatterns::front_description_bracket()
            ))?,
            front_description_parentheses: Regex::new(&format!(
                r"(?m){}",
                ComputedPatterns::front_description_parentheses()
            ))?,
            end_description_bracket: Regex::new(&format!(
                r"(?m){}",
                ComputedPatterns::end_description_bracket()
            ))?,
            end_description_parentheses: Regex::new(&format!(
                r"(?m){}",
                ComputedPatterns::end_description_parentheses()
            ))?,
            inline_description: Regex::new(&format!(r"(?m){}", RegexPatterns::INLINE_DESCRIPTION))?,
        })
    }

    /// Strip HTML and SRT tags from text
    pub fn strip_tags(&self, text: &str) -> String {
        self.tags.replace_all(text, "").to_string()
    }
}

impl Default for CompiledRegexes {
    fn default() -> Self {
        Self::new().expect("Failed to compile regex patterns")
    }
}
