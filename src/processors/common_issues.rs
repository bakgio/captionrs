use std::sync::OnceLock;

use chrono::TimeDelta;
use encoding_rs::WINDOWS_1252;
use html_escape::decode_html_entities;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::processors::base::BaseProcessor;
use crate::processors::rtl::{RTL_LANGUAGES, RTLFixer};
use crate::subripfile::{SubRipFile, SubtitleError};
use crate::utils::time::line_duration;

#[derive(Clone)]
pub struct CommonIssuesFixer {
    pub remove_gaps: bool,
}

impl CommonIssuesFixer {
    pub fn new() -> Self {
        Self { remove_gaps: true }
    }

    fn normalize_unicode(&self, mut srt: SubRipFile) -> SubRipFile {
        for subtitle in srt.iter_mut() {
            subtitle.content = subtitle.content.nfkc().collect::<String>();
        }

        srt
    }

    fn fix_time_codes(&self, mut srt: SubRipFile) -> SubRipFile {
        let mut offset_hours = 0_i64;

        for subtitle in srt.iter_mut() {
            let hours = subtitle.start.num_seconds() / 3600;

            if offset_hours == 0 && hours > 23 {
                offset_hours = hours;
            }

            if offset_hours != 0 {
                let offset = TimeDelta::hours(offset_hours);
                subtitle.start -= offset;
                subtitle.end -= offset;
            }
        }

        srt
    }

    fn correct_subtitles(&self, mut srt: SubRipFile) -> SubRipFile {
        for subtitle in srt.iter_mut() {
            for _ in 0..2 {
                subtitle.content = decode_html_entities(&subtitle.content).to_string();
            }

            for _ in 0..2 {
                subtitle.content = self.fix_line(&subtitle.content);
                subtitle.content = subtitle.content.trim().to_string();
            }

            subtitle.content = subtitle.content.trim_matches('\n').to_string();
        }

        let srt = self.remove_global_tags(srt);
        let combined = self.combine_timecodes(srt);

        if self.remove_gaps {
            self.remove_short_gaps(combined)
        } else {
            combined
        }
    }

    fn remove_global_tags(&self, mut srt: SubRipFile) -> SubRipFile {
        if srt.len() > 10
            && srt
                .iter()
                .all(|line| line.content.starts_with("<i>") && line.content.ends_with("</i>"))
        {
            for subtitle in srt.iter_mut() {
                subtitle.content = subtitle.content[3..subtitle.content.len() - 4].to_string();
            }
        }

        if srt.len() > 100 && srt.iter().all(|line| line.content.starts_with(r"{\an8}")) {
            for subtitle in srt.iter_mut() {
                subtitle.content = subtitle.content[6..].to_string();
            }
        }

        srt
    }

    fn combine_timecodes(&self, srt: SubRipFile) -> SubRipFile {
        let mut subtitles = Vec::new();

        for line in srt {
            if subtitles.is_empty() {
                subtitles.push(line);
                continue;
            }

            let last_index = subtitles.len() - 1;
            let last = &subtitles[last_index];

            if line_duration(last) == line_duration(&line)
                && last.start == line.start
                && last.end == line.end
            {
                if last.content != line.content {
                    subtitles[last_index].content.push('\n');
                    subtitles[last_index]
                        .content
                        .push_str(&line.content.replace(r"{\an8}", ""));
                }
            } else if self.subtract_ts(line.start, last.end) < 10 && line.content == last.content {
                subtitles[last_index].end = line.end;
            } else if 0 < self.subtract_ts(line.start, last.end)
                && self.subtract_ts(line.start, last.end) <= 85
                && line.content.starts_with(&last.content)
                && self.remove_gaps
            {
                subtitles[last_index].end = line.end;
                subtitles[last_index].content = line.content;
            } else if self.subtract_ts(line.start, last.end) == 0 {
                subtitles[last_index].end = last.end - TimeDelta::milliseconds(1);
                subtitles.push(line);
            } else if !line.content.trim().is_empty() {
                subtitles.push(line);
            }
        }

        let mut srt = SubRipFile::new(Some(subtitles));
        srt.clean_indexes();
        srt
    }

    fn remove_short_gaps(&self, srt: SubRipFile) -> SubRipFile {
        let mut subtitles = Vec::new();

        for line in srt {
            if subtitles.is_empty() {
                subtitles.push(line);
                continue;
            }

            let last_index = subtitles.len() - 1;
            let last = &subtitles[last_index];
            let gap = self.subtract_ts(line.start, last.end);

            if 1 < gap && gap <= 85 {
                subtitles[last_index].end = line.start - TimeDelta::milliseconds(1);
                subtitles.push(line);
            } else if !line.content.trim().is_empty() {
                subtitles.push(line);
            }
        }

        let mut srt = SubRipFile::new(Some(subtitles));
        srt.clean_indexes();
        srt
    }

    fn fix_line(&self, line: &str) -> String {
        let mut fixed = line.to_string();

        fixed = multiple_spaces_regex().replace_all(&fixed, " ").to_string();
        fixed = leading_space_regex().replace_all(&fixed, "").to_string();
        fixed = newline_space_regex().replace_all(&fixed, "\n").to_string();

        fixed = fix_mojibake(&fixed);
        fixed = fixed.replace('Â', "");
        fixed = fixed.replace("Â£", "£");
        fixed = fixed.replace("Â¶", "♪");
        fixed = fixed.replace("‐", "-");
        fixed = fixed.replace("♫", "♪");

        fixed = hash_start_regex()
            .replace_all(&fixed, "$1$2♪$3")
            .to_string();
        fixed = hash_end_regex().replace_all(&fixed, " ♪$1").to_string();
        fixed = only_hash_regex().replace_all(&fixed, "♪").to_string();
        fixed = note_italic_start_regex()
            .replace_all(&fixed, "<i>♪ $1")
            .to_string();
        fixed = note_italic_end_regex()
            .replace_all(&fixed, "$1 ♪</i>")
            .to_string();
        fixed = line_start_pound_regex()
            .replace_all(&fixed, "♪ ")
            .to_string();
        fixed = line_end_pound_regex().replace_all(&fixed, " ♪").to_string();
        fixed = duplicate_notes_regex().replace_all(&fixed, "♪").to_string();
        fixed = note_text_start_regex()
            .replace_all(&fixed, "♪ $1")
            .to_string();
        fixed = note_text_end_regex()
            .replace_all(&fixed, "$1 ♪")
            .to_string();
        fixed = ass_nbsp_regex()
            .replace_all(&fixed, " ")
            .to_string()
            .trim()
            .to_string();
        fixed = leftover_amps_regex().replace_all(&fixed, "&").to_string();
        fixed = quote_fixes_regex().replace_all(&fixed, "'").to_string();

        fixed = ass_position_regex()
            .replace_all(&fixed, r"{\an8}")
            .to_string();
        fixed = ass_space_regex()
            .replace_all(&fixed, r"{\an8}$2")
            .to_string();
        fixed = hanging_tag_start_regex()
            .replace_all(&fixed, "$1")
            .to_string();
        fixed = hanging_tag_end_regex()
            .replace_all(&fixed, "\n")
            .to_string();
        fixed = duplicate_open_tags_regex()
            .replace_all(&fixed, "$1")
            .to_string();
        fixed = duplicate_close_tags_regex()
            .replace_all(&fixed, "$1")
            .to_string();
        fixed = tag_space_regex().replace_all(&fixed, "$1").to_string();
        fixed = leading_space_after_tag_regex()
            .replace_all(&fixed, "")
            .to_string();
        fixed = strip_non_italic_tags(&fixed);
        fixed = tag_spacing_regex().replace_all(&fixed, "$1$2").to_string();
        fixed = hanging_open_tag_regex()
            .replace_all(&fixed, "\n$1")
            .to_string();
        fixed = hanging_close_tag_regex()
            .replace_all(&fixed, "$1\n")
            .to_string();
        fixed = space_inside_open_tag_regex()
            .replace_all(&fixed, " $1")
            .to_string();
        fixed = space_inside_close_tag_regex()
            .replace_all(&fixed, "$1 ")
            .to_string();
        fixed = needless_space_in_tag_regex()
            .replace_all(&fixed, "$1")
            .to_string();
        fixed = tag_space_tag_regex().replace_all(&fixed, "$1").to_string();
        fixed = empty_tags_regex().replace_all(&fixed, "").to_string();
        fixed = an8_newline_regex().replace_all(&fixed, "$1").to_string();

        if let Some(captures) = opening_tag_regex().captures(&fixed) {
            let closing_tag = format!("</{}>", &captures[1]);
            if !fixed.contains(&closing_tag) {
                fixed.push_str(&closing_tag);
            }
        }

        fixed = bracket_spaces_regex()
            .replace_all(&fixed, "($1)")
            .to_string();
        fixed = br_tags_regex().replace_all(&fixed, "\n").to_string();
        fixed = empty_line_dot_regex().replace_all(&fixed, "").to_string();
        fixed = empty_line_dash_regex().replace_all(&fixed, "").to_string();
        fixed = empty_line_tag_regex().replace_all(&fixed, "").to_string();
        fixed = single_char_regex().replace_all(&fixed, "").to_string();
        fixed = ellipsis_space_regex()
            .replace_all(&fixed, "$1$2 $3")
            .to_string();
        fixed = close_tag_space_regex()
            .replace_all(&fixed, "$1 $2")
            .to_string();
        fixed = comma_space_regex()
            .replace_all(&fixed, "$1, $2")
            .to_string();
        fixed = comma_newline_regex()
            .replace_all(&fixed, ", $1")
            .to_string();
        fixed = front_ellipses_regex()
            .replace_all(&fixed, "$1...")
            .to_string();
        fixed = end_ellipses_regex()
            .replace_all(&fixed, "...$1")
            .to_string();
        fixed = fix_leading_speaker_hyphen(&fixed);
        fixed = double_hyphen_regex()
            .replace_all(&fixed, "--$1")
            .to_string();
        fixed = notes_in_tags_regex()
            .replace_all(&fixed, "$2$1")
            .to_string();
        fixed = trailing_spaces_regex()
            .replace_all(&fixed, "")
            .to_string()
            .trim()
            .to_string();

        fixed = line_split1_regex()
            .replace_all(&fixed, "$1$2\n$3")
            .to_string();
        fixed = apply_line_split2(&fixed);
        fixed = weird_linebreak_regex()
            .replace_all(&fixed, "$1$2 ")
            .to_string();
        fixed = add_missing_hyphen(&fixed);
        fixed = crlf_regex().replace_all(&fixed, "\r\n").to_string();
        fixed = multiple_newlines_regex()
            .replace_all(&fixed, "\n")
            .to_string();
        fixed = italic_spaces_regex()
            .replace_all(&fixed, "</i> ")
            .to_string();
        fixed = italic_hyphen_regex().replace_all(&fixed, "-$1").to_string();

        fixed.trim().to_string()
    }

    fn subtract_ts(&self, ts1: TimeDelta, ts2: TimeDelta) -> i64 {
        (ts1 - ts2)
            .num_microseconds()
            .map(|micros| (micros as f64 / 1000.0).round() as i64)
            .unwrap_or_else(|| (ts1 - ts2).num_milliseconds())
    }
}

impl Default for CommonIssuesFixer {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseProcessor for CommonIssuesFixer {
    fn process(
        &self,
        srt: SubRipFile,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        let original_srt = srt.clone();
        let fixed = self.fix_time_codes(srt);
        let mut corrected = self.correct_subtitles(fixed);

        if let Some(language_code) = language_code(language) {
            if RTL_LANGUAGES.contains(&language_code.as_str()) {
                corrected = RTLFixer::new().process(corrected, language)?.0;
            }

            if language_code == "en" {
                corrected = self.normalize_unicode(corrected);
            }
        }

        let changed = corrected != original_srt;
        Ok((corrected, changed))
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl crate::processors::base::AsyncBaseProcessor for CommonIssuesFixer {
    async fn process_async(
        &self,
        srt: SubRipFile,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        let fixer = self.clone();
        let language = language.map(str::to_string);
        crate::async_utils::run_blocking(move || fixer.process(srt, language.as_deref())).await
    }
}

fn language_code(language: Option<&str>) -> Option<String> {
    let raw = language?.trim();
    if raw.is_empty() {
        return None;
    }

    let normalized = raw.to_ascii_lowercase();
    let without_parenthetical = normalized
        .split_once('(')
        .map(|(value, _)| value.trim())
        .unwrap_or(normalized.as_str());
    let primary = without_parenthetical
        .split(['-', '_'])
        .next()
        .unwrap_or(without_parenthetical)
        .trim();
    let first_word = without_parenthetical
        .split_whitespace()
        .next()
        .unwrap_or(without_parenthetical);

    if primary.is_empty() {
        return None;
    }

    map_language_alias(without_parenthetical)
        .or_else(|| map_language_alias(primary))
        .or_else(|| map_language_alias(first_word))
        .map(str::to_string)
        .or_else(|| Some(primary.to_string()))
}

fn fix_mojibake(text: &str) -> String {
    let mut fixed = text.to_string();

    for _ in 0..4 {
        let whole_text = decode_windows_1252_roundtrip(&fixed).unwrap_or_else(|| fixed.clone());
        let decoded_fixed = decode_mojibake_runs(&fixed);
        let decoded_whole = decode_mojibake_runs(&whole_text);
        let improved =
            preferred_mojibake_candidate(&fixed, [&whole_text, &decoded_fixed, &decoded_whole]);

        if improved == fixed {
            break;
        }

        fixed = improved;
    }

    fixed
}

fn map_language_alias(value: &str) -> Option<&'static str> {
    match value {
        "en" | "eng" | "english" => Some("en"),
        "ar" | "ara" | "arabic" => Some("ar"),
        "fa" | "fas" | "per" | "persian" | "farsi" => Some("fa"),
        "he" | "heb" | "hebrew" | "iw" => Some("he"),
        "ps" | "pus" | "pashto" | "pushto" => Some("ps"),
        "syc" | "syr" | "syriac" => Some("syc"),
        "ug" | "uig" | "uyghur" | "uighur" => Some("ug"),
        "ur" | "urd" | "urdu" => Some("ur"),
        _ => None,
    }
}

fn decode_windows_1252_roundtrip(text: &str) -> Option<String> {
    let (encoded, _, had_errors) = WINDOWS_1252.encode(text);
    if had_errors {
        return None;
    }

    String::from_utf8(encoded.into_owned()).ok()
}

fn preferred_mojibake_candidate<const N: usize>(original: &str, candidates: [&str; N]) -> String {
    let mut best = original.to_string();
    let mut best_score = mojibake_score(original);

    for candidate in candidates {
        let candidate_score = mojibake_score(candidate);
        if candidate_score < best_score {
            best = candidate.to_string();
            best_score = candidate_score;
        }
    }

    best
}

fn mojibake_score(text: &str) -> usize {
    text.chars()
        .map(|character| {
            usize::from(matches!(
                character,
                '\u{00A1}'
                    | '\u{00A2}'
                    | '\u{00A3}'
                    | '\u{00AF}'
                    | '\u{00BC}'
                    | '\u{00BD}'
                    | '\u{00BE}'
                    | '\u{00C2}'
                    | '\u{00C3}'
                    | '\u{00D7}'
                    | '\u{00E2}'
                    | '\u{00F0}'
                    | '\u{0192}'
                    | '\u{2044}'
                    | '\u{FFFD}'
            )) + usize::from(character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
        })
        .sum()
}

fn decode_mojibake_runs(text: &str) -> String {
    let mut decoded = String::new();
    let mut run = String::new();

    for character in text.chars() {
        let candidate = character.to_string();
        let (_, _, had_errors) = WINDOWS_1252.encode(&candidate);

        if had_errors {
            flush_mojibake_run(&mut decoded, &mut run);
            decoded.push(character);
            continue;
        }

        run.push(character);
    }

    flush_mojibake_run(&mut decoded, &mut run);
    decoded
}

fn flush_mojibake_run(output: &mut String, run: &mut String) {
    if run.is_empty() {
        return;
    }

    let original = std::mem::take(run);
    let (encoded, _, had_errors) = WINDOWS_1252.encode(&original);

    if had_errors {
        output.push_str(&original);
        return;
    }

    output.push_str(&decode_mojibake_bytes(encoded.as_ref()));
}

fn decode_mojibake_bytes(bytes: &[u8]) -> String {
    let mut decoded = String::new();
    let mut index = 0;

    while index < bytes.len() {
        if let Some(length) = utf8_sequence_length(bytes[index])
            && index + length <= bytes.len()
            && let Ok(chunk) = std::str::from_utf8(&bytes[index..index + length])
        {
            decoded.push_str(chunk);
            index += length;
            continue;
        }

        let (character, _, _) = WINDOWS_1252.decode(&bytes[index..index + 1]);
        decoded.push_str(&character);
        index += 1;
    }

    decoded
}

fn utf8_sequence_length(first_byte: u8) -> Option<usize> {
    match first_byte {
        0x00..=0x7F => Some(1),
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

fn strip_non_italic_tags(text: &str) -> String {
    html_tags_regex()
        .replace_all(text, |captures: &regex::Captures| match &captures[0] {
            "<i>" | "</i>" => captures[0].to_string(),
            _ => String::new(),
        })
        .to_string()
}

fn apply_line_split2(text: &str) -> String {
    let mut split_index = None;
    let chars: Vec<(usize, char)> = text.char_indices().collect();

    for index in 1..chars.len() {
        let split_pos = chars[index].0;
        let current = chars[index].1;
        let previous = chars[index - 1].1;
        let before_previous = index
            .checked_sub(2)
            .and_then(|position| chars.get(position))
            .map(|(_, value)| *value);

        if !current.is_ascii_uppercase() {
            continue;
        }

        if !matches!(previous, '!' | '.' | ';' | ':' | '?') {
            continue;
        }

        if before_previous.is_some_and(|value| {
            value == '.' || value.is_ascii_uppercase() || value.is_whitespace()
        }) {
            continue;
        }

        let prefix = &text[..split_pos];
        if prefix.ends_with("Mr.") || prefix.ends_with("Ms.") || prefix.ends_with("Mrs.") {
            continue;
        }

        let next = text[split_pos..].chars().nth(1);
        if next == Some('.') {
            continue;
        }

        split_index = Some(split_pos);
        break;
    }

    if let Some(index) = split_index {
        format!("- {}\n- {}", &text[..index], &text[index..])
    } else {
        text.to_string()
    }
}

fn fix_leading_speaker_hyphen(text: &str) -> String {
    text.lines()
        .map(fix_leading_speaker_hyphen_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn fix_leading_speaker_hyphen_line(line: &str) -> String {
    let (prefix, rest) = if let Some(rest) = line.strip_prefix("<i>") {
        ("<i>", rest)
    } else if let Some(rest) = line.strip_prefix(r"{\an8}") {
        (r"{\an8}", rest)
    } else {
        ("", line)
    };

    let hyphen_count = rest
        .chars()
        .take_while(|&character| character == '-')
        .count();
    if hyphen_count == 0 {
        return line.to_string();
    }

    let remaining = &rest[hyphen_count..];
    let Some(first_character) = remaining.chars().next() else {
        return line.to_string();
    };

    let target = if first_character == '\'' {
        remaining.chars().nth(1)
    } else {
        Some(first_character)
    };

    if target.is_some_and(is_speaker_hyphen_target) {
        format!("{prefix}- {remaining}")
    } else {
        line.to_string()
    }
}

fn is_speaker_hyphen_target(character: char) -> bool {
    character.is_alphanumeric()
        || matches!(
            character,
            '"' | '[' | '(' | '<' | '{' | '.' | '$' | '¿' | '¡' | '…' | '♪' | 'â'
        )
}

fn add_missing_hyphen(text: &str) -> String {
    let Some((first_line, second_line)) = text.split_once('\n') else {
        return text.to_string();
    };

    if first_line.trim_start().starts_with('-') {
        return text.to_string();
    }

    let Some(second_content) = second_line.strip_prefix("- ") else {
        return text.to_string();
    };

    let mut characters = second_content.chars();
    let Some(first_character) = characters.next() else {
        return text.to_string();
    };
    let Some(second_character) = characters.next() else {
        return text.to_string();
    };

    if first_character.is_ascii_uppercase()
        && second_character.is_ascii_lowercase()
        && !second_content.contains('\n')
    {
        format!("- {}\n- {}", first_line.trim(), second_content)
    } else {
        text.to_string()
    }
}

fn multiple_spaces_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r" {2,}").unwrap())
}

fn leading_space_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^\s*").unwrap())
}

fn newline_space_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\n\s*").unwrap())
}

fn hash_start_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)^((?:\{\\an8\})?(?:<i>)?)(- ?)?[#\*]{1,}(\s+)").unwrap())
}

fn hash_end_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)\s[#\*]{1,3}(</i>$|$)").unwrap())
}

fn only_hash_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)^[#\*]+$").unwrap())
}

fn note_italic_start_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"♪ <i>(.*)").unwrap())
}

fn note_italic_end_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(♪.*)</i>\s*♪").unwrap())
}

fn line_start_pound_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^£ ").unwrap())
}

fn line_end_pound_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r" £$").unwrap())
}

fn duplicate_notes_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"♪{1,}").unwrap())
}

fn note_text_start_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^♪([A-Za-z])").unwrap())
}

fn note_text_end_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"([A-Za-z])♪").unwrap())
}

fn ass_nbsp_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(\\h)+").unwrap())
}

fn leftover_amps_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"&(amp;){1,}").unwrap())
}

fn quote_fixes_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"'[`’]").unwrap())
}

fn ass_position_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(\{\\an[0-9]\}){1,}").unwrap())
}

fn ass_space_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(\{\\an[0-9]\}) +([A-Za-z-])").unwrap())
}

fn hanging_tag_start_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(<[a-z]>)\n").unwrap())
}

fn hanging_tag_end_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)</([a-z])>$\n<([a-z])>").unwrap())
}

fn duplicate_open_tags_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(<[a-z]>){1,}").unwrap())
}

fn duplicate_close_tags_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(</[a-z]>){1,}").unwrap())
}

fn tag_space_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(<[a-z]>) {1,}").unwrap())
}

fn leading_space_after_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^ {1,}").unwrap())
}

fn html_tags_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"</?[a-z]+>").unwrap())
}

fn tag_spacing_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(<[a-z]>|\{\\an8\}) (<[a-z]>|\{\\an8\})").unwrap())
}

fn hanging_open_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(<[a-z]>)\n").unwrap())
}

fn hanging_close_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\n(</[a-z]>)").unwrap())
}

fn space_inside_open_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(<[a-z]>) ").unwrap())
}

fn space_inside_close_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r" (</[a-z]>)").unwrap())
}

fn needless_space_in_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(<[a-z]>) ").unwrap())
}

fn tag_space_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)(?:</[a-z]>)(\s*)(?:<[a-z]>)").unwrap())
}

fn empty_tags_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"<[a-z]>\s*</[a-z]>").unwrap())
}

fn an8_newline_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(\{\\an8\})\n").unwrap())
}

fn opening_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^(?:\{\\an8\})?<([a-z])>").unwrap())
}

fn bracket_spaces_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\( (.*) \)").unwrap())
}

fn br_tags_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"<br ?/?>").unwrap())
}

fn empty_line_dot_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)^\.?\s*$").unwrap())
}

fn empty_line_dash_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)^-?\s*$").unwrap())
}

fn empty_line_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)^(</?i>|\{\\an8\})?\s*$").unwrap())
}

fn single_char_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    // Keep this cleanup rule narrowly scoped to the literal bracketed token.
    REGEX.get_or_init(|| Regex::new(r"^\[A-Za-z0-9\]$").unwrap())
}

fn ellipsis_space_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"([a-z])(\.\.\.)([a-zA-Z][^.])").unwrap())
}

fn close_tag_space_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(</[a-z]>)(\w)").unwrap())
}

fn comma_space_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"([a-z]),([a-zA-Z])").unwrap())
}

fn comma_newline_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r",\n([a-z]+[.\?])\s*$").unwrap())
}

fn front_ellipses_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"(^\s*(?:[<{][/\\]?[a-z0-9.]+[}>])?\s*(-)?\s*(?:[<{][/\\]?[a-z0-9.]+[}>])?\s*)\.{1,}",
        )
        .unwrap()
    })
}

fn end_ellipses_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\.{2,}([<{][/\\]?[a-z0-9.]+[}>])?\s*$").unwrap())
}
fn double_hyphen_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)\s*--(\s*)").unwrap())
}

fn notes_in_tags_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)(</[a-z]>)(\s*♪{1,})$").unwrap())
}

fn trailing_spaces_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m) +$").unwrap())
}

fn line_split1_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(.*)([^.][\]\)])([A-Z][^.])").unwrap())
}

fn weird_linebreak_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(^<[a-z]>|\n<[a-z]>)(\w+)\n").unwrap())
}
fn crlf_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\r\n{1,}").unwrap())
}

fn multiple_newlines_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\n{1,}").unwrap())
}

fn italic_spaces_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r" +</i> +").unwrap())
}

fn italic_hyphen_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"<i>-</i>([^<]+)").unwrap())
}
