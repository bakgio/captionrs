use regex::Regex;
use std::collections::VecDeque;
use std::sync::OnceLock;

use crate::processors::base::BaseProcessor;
use crate::regex::CompiledRegexes;
use crate::subripfile::{SubRipFile, Subtitle, SubtitleError};

#[derive(Clone)]
pub struct SDHStripper {
    extra_regexes: Vec<Regex>,
    compiled_regexes: CompiledRegexes,
}

const BLEEP_BRACKET_PLACEHOLDER: &str = "\u{e000}";
const BLEEP_PAREN_PLACEHOLDER: &str = "\u{e001}";

impl SDHStripper {
    pub fn new() -> Self {
        Self {
            extra_regexes: Vec::new(),
            compiled_regexes: CompiledRegexes::default(),
        }
    }

    pub fn with_extra_regexes(regex_patterns: Vec<&str>) -> Result<Self, regex::Error> {
        let mut compiled_regexes = Vec::new();
        for pattern in regex_patterns {
            compiled_regexes.push(Regex::new(pattern)?);
        }
        Ok(Self {
            extra_regexes: compiled_regexes,
            compiled_regexes: CompiledRegexes::default(),
        })
    }
}

impl Default for SDHStripper {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseProcessor for SDHStripper {
    fn process(
        &self,
        srt: SubRipFile,
        _language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        let original_len = srt.len();
        let original_srt = srt.clone();
        let srt = self.protect_bleeps(srt);

        // Process all subtitles through the SDH stripping pipeline
        let mut stripped: VecDeque<Subtitle> = VecDeque::new();

        // Step 1: Clean full line descriptions
        let after_full_line = self.clean_full_line_descriptions(srt);

        // Step 2: Clean new line descriptions
        let after_new_line = self.clean_new_line_descriptions(after_full_line);

        // Step 3: Clean inline descriptions
        let after_inline = self.clean_inline_descriptions(after_new_line);

        // Step 4: Clean speaker names
        let after_speakers = self.clean_speaker_names(after_inline);

        // Step 5: Strip CC speaker tags
        let after_cc = self.strip_cc_speaker_tags(after_speakers);

        // Step 6: Strip notes
        let after_notes = self.strip_notes(after_cc);

        // Step 7: Remove extra hyphens
        let after_hyphens = self.remove_extra_hyphens(after_notes);

        // Step 8: Run extra regexes
        let after_extra = self.run_extra_regexes(after_hyphens);

        // Filter out empty content and collect
        for subtitle in after_extra {
            if !subtitle.content.trim().is_empty() {
                stripped.push_back(subtitle);
            }
        }

        let mut result_srt = self.restore_bleeps(SubRipFile::new(Some(stripped.into())));
        result_srt.clean_indexes();

        let changed = result_srt.len() != original_len || result_srt != original_srt;

        Ok((result_srt, changed))
    }
}

impl SDHStripper {
    fn protect_bleeps(&self, mut srt: SubRipFile) -> SubRipFile {
        for subtitle in srt.iter_mut() {
            subtitle.content = self.protect_inline_bleeps_in_text(&subtitle.content);
        }

        srt
    }

    fn restore_bleeps(&self, mut srt: SubRipFile) -> SubRipFile {
        for subtitle in srt.iter_mut() {
            subtitle.content = subtitle
                .content
                .replace(BLEEP_BRACKET_PLACEHOLDER, "[bleep]")
                .replace(BLEEP_PAREN_PLACEHOLDER, "(bleep)");
        }

        srt
    }

    fn protect_inline_bleeps_in_text(&self, text: &str) -> String {
        let stripped = self.compiled_regexes.strip_tags(text);
        let trimmed = stripped.trim_start_matches('-').trim();
        let has_bleep = trimmed.contains("[bleep]") || trimmed.contains("(bleep)");

        if !has_bleep || matches!(trimmed, "[bleep]" | "(bleep)") {
            return text.to_string();
        }

        text.replace("[bleep]", BLEEP_BRACKET_PLACEHOLDER)
            .replace("(bleep)", BLEEP_PAREN_PLACEHOLDER)
    }

    fn clean_new_line_description_content(&self, content: &str) -> String {
        let position = self
            .compiled_regexes
            .position_tags
            .find(content)
            .map(|m| m.as_str().to_string());

        let mut cleaned = self
            .compiled_regexes
            .new_line_description_bracket
            .replace_all(content, "")
            .to_string();
        cleaned = self
            .compiled_regexes
            .new_line_description_parentheses
            .replace_all(&cleaned, "")
            .to_string();
        cleaned = cleaned.trim().to_string();

        if let Some(position) = position
            && !cleaned.contains(&position)
        {
            cleaned = format!("{position}{cleaned}");
        }

        cleaned
    }

    fn clean_inline_descriptions_content(&self, content: &str) -> String {
        let mut cleaned = tagged_front_description_bracket_regex()
            .replace_all(content, "$1")
            .to_string();
        cleaned = tagged_front_description_parentheses_regex()
            .replace_all(&cleaned, "$1")
            .to_string();
        cleaned = self
            .compiled_regexes
            .front_description_bracket
            .replace_all(&cleaned, "$1")
            .to_string();
        cleaned = self
            .compiled_regexes
            .front_description_parentheses
            .replace_all(&cleaned, "$1")
            .to_string();
        cleaned = self
            .compiled_regexes
            .end_description_bracket
            .replace_all(&cleaned, "")
            .to_string();
        cleaned = self
            .compiled_regexes
            .end_description_parentheses
            .replace_all(&cleaned, "")
            .to_string();
        cleaned = self
            .compiled_regexes
            .inline_description
            .replace_all(&cleaned, "")
            .to_string();

        cleaned.trim().to_string()
    }

    fn clean_speaker_names_content(&self, content: &str) -> String {
        content
            .lines()
            .map(|line| {
                let without_parenthetical =
                    strip_speaker_prefix(line, &self.compiled_regexes.speaker_parentheses, false);
                strip_speaker_prefix(&without_parenthetical, &self.compiled_regexes.speaker, true)
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    /// Step 1: Remove full line descriptions (brackets and parentheses)
    fn clean_full_line_descriptions(&self, srt: SubRipFile) -> impl Iterator<Item = Subtitle> {
        srt.into_iter().filter_map(move |subtitle| {
            let text = self.compiled_regexes.strip_tags(&subtitle.content);

            // Check if entire line is a description
            let is_full_bracket = self
                .compiled_regexes
                .full_line_description_bracket
                .is_match(&text);
            let is_full_paren = self
                .compiled_regexes
                .full_line_description_parentheses
                .is_match(&text);

            if is_full_bracket || is_full_paren {
                None // Remove entirely descriptive lines
            } else {
                Some(subtitle)
            }
        })
    }

    /// Step 2: Remove line descriptions taking up an entire line break
    fn clean_new_line_descriptions(
        &self,
        iter: impl Iterator<Item = Subtitle>,
    ) -> impl Iterator<Item = Subtitle> {
        iter.map(move |mut subtitle| {
            subtitle.content = self.clean_new_line_description_content(&subtitle.content);
            subtitle
        })
    }

    /// Step 3: Remove inline descriptions
    fn clean_inline_descriptions(
        &self,
        iter: impl Iterator<Item = Subtitle>,
    ) -> impl Iterator<Item = Subtitle> {
        iter.map(move |mut subtitle| {
            subtitle.content = self.clean_inline_descriptions_content(&subtitle.content);
            subtitle
        })
    }

    /// Step 4: Remove speaker names while retaining frontal tags/hyphens
    fn clean_speaker_names(
        &self,
        iter: impl Iterator<Item = Subtitle>,
    ) -> impl Iterator<Item = Subtitle> {
        iter.map(move |mut subtitle| {
            subtitle.content = self.clean_speaker_names_content(&subtitle.content);
            subtitle
        })
    }

    /// Step 5: Remove US closed caption-style speaker tags (>>)
    fn strip_cc_speaker_tags(
        &self,
        iter: impl Iterator<Item = Subtitle>,
    ) -> impl Iterator<Item = Subtitle> {
        let cc_speaker_regex = Regex::new(r"(^|\n)(</?[a-z]>|\{\\an8\})?>> ").unwrap();
        let cc_only_regex = Regex::new(r"(^|\n)(</?[a-z]>|\{\\an8\})?>>($|\n)").unwrap();

        iter.filter_map(move |mut subtitle| {
            // Remove ">> " before text
            subtitle.content = cc_speaker_regex
                .replace_all(&subtitle.content, "$1$2")
                .to_string();

            // Filter out lines consisting only of ">>"
            if cc_only_regex.is_match(&subtitle.content) {
                None
            } else {
                Some(subtitle)
            }
        })
    }

    /// Step 6: Remove lines with just musical notes
    fn strip_notes(&self, iter: impl Iterator<Item = Subtitle>) -> impl Iterator<Item = Subtitle> {
        let notes_regex = Regex::new(r"^♪+$").unwrap();

        iter.filter_map(move |subtitle| {
            let stripped_tags = self.compiled_regexes.strip_tags(&subtitle.content);
            let stripped_content = notes_regex.replace_all(&stripped_tags, "");
            let cleaned_content = stripped_content.trim().replace(" ", "");

            if cleaned_content.is_empty() {
                None // Remove lines with only musical notes
            } else {
                Some(subtitle)
            }
        })
    }

    /// Step 7: Remove speaker hyphens if there's only one line
    fn remove_extra_hyphens(
        &self,
        iter: impl Iterator<Item = Subtitle>,
    ) -> impl Iterator<Item = Subtitle> {
        let hyphen_regex = Regex::new(r"^(<i>|\{\\an8\})?-\s*").unwrap();
        let line_hyphen_regex = Regex::new(r"(?:^|\n)(<i>|\{\\an8\})?-\s*").unwrap();

        iter.map(move |mut subtitle| {
            // Count occurrences of leading hyphens (at start of string or after newlines)
            let hyphen_count = line_hyphen_regex.find_iter(&subtitle.content).count();

            // Only remove the leading hyphen if there's exactly one hyphen in the entire subtitle
            if hyphen_count == 1 {
                subtitle.content = hyphen_regex
                    .replace(&subtitle.content, "$1")
                    .to_string()
                    .trim()
                    .to_string();
            }

            subtitle
        })
    }

    /// Step 8: Run extra regexes provided by user
    fn run_extra_regexes<'a>(
        &'a self,
        iter: impl Iterator<Item = Subtitle> + 'a,
    ) -> impl Iterator<Item = Subtitle> + 'a {
        iter.map(move |mut subtitle| {
            for regex in &self.extra_regexes {
                subtitle.content = regex.replace_all(&subtitle.content, "").to_string();
            }
            subtitle
        })
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl crate::processors::base::AsyncBaseProcessor for SDHStripper {
    /// Processes given SubRipFile asynchronously without blocking the runtime.
    async fn process_async(
        &self,
        srt: SubRipFile,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        let _language = language;
        let original_srt = srt.clone();
        let original_len = srt.len();
        let srt = self.protect_bleeps(srt);

        let result_srt = if srt.len() <= 200 {
            self.process_async_with_yields(srt).await
        } else {
            let stripper = self.clone();
            crate::async_utils::run_blocking(move || Ok(stripper.process_large_sync(srt))).await?
        };

        let changed = result_srt.len() != original_len || result_srt != original_srt;
        Ok((result_srt, changed))
    }
}

#[cfg(feature = "async")]
impl SDHStripper {
    /// Process with yield points for smaller-medium files
    async fn process_async_with_yields(&self, srt: SubRipFile) -> SubRipFile {
        const CHUNK_SIZE: usize = 50; // Process in chunks of 50 subtitles
        let mut processed = 0;

        // Step 1: Clean full line descriptions with yield points
        let mut after_full_line = Vec::new();
        for subtitle in srt.into_iter() {
            let text = self.compiled_regexes.strip_tags(&subtitle.content);

            // Check if entire line is a description
            let is_full_bracket = self
                .compiled_regexes
                .full_line_description_bracket
                .is_match(&text);
            let is_full_paren = self
                .compiled_regexes
                .full_line_description_parentheses
                .is_match(&text);

            if !(is_full_bracket || is_full_paren) {
                after_full_line.push(subtitle);
            }

            processed += 1;
            if processed % CHUNK_SIZE == 0 {
                tokio::task::yield_now().await;
            }
        }

        // Step 2: Clean new line descriptions with yield points
        let mut after_new_line = Vec::new();
        processed = 0;
        for mut subtitle in after_full_line {
            subtitle.content = self.clean_new_line_description_content(&subtitle.content);
            after_new_line.push(subtitle);

            processed += 1;
            if processed % CHUNK_SIZE == 0 {
                tokio::task::yield_now().await;
            }
        }

        // Step 3: Clean inline descriptions with yield points
        let mut after_inline = Vec::new();
        processed = 0;
        for mut subtitle in after_new_line {
            subtitle.content = self.clean_inline_descriptions_content(&subtitle.content);
            after_inline.push(subtitle);

            processed += 1;
            if processed % CHUNK_SIZE == 0 {
                tokio::task::yield_now().await;
            }
        }

        // Step 4: Clean speaker names with yield points
        let mut after_speakers = Vec::new();
        processed = 0;
        for mut subtitle in after_inline {
            subtitle.content = self.clean_speaker_names_content(&subtitle.content);
            after_speakers.push(subtitle);

            processed += 1;
            if processed % CHUNK_SIZE == 0 {
                tokio::task::yield_now().await;
            }
        }

        // Step 5: Strip CC speaker tags with yield points
        let mut after_cc = Vec::new();
        let cc_speaker_regex = regex::Regex::new(r"(^|\n)(</?[a-z]>|\{\\an8\})?>> ").unwrap();
        let cc_only_regex = regex::Regex::new(r"(^|\n)(</?[a-z]>|\{\\an8\})?>>($|\n)").unwrap();
        processed = 0;
        for mut subtitle in after_speakers {
            // Remove ">> " before text
            subtitle.content = cc_speaker_regex
                .replace_all(&subtitle.content, "$1$2")
                .to_string();

            // Filter out lines consisting only of ">>"
            if !cc_only_regex.is_match(&subtitle.content) {
                after_cc.push(subtitle);
            }

            processed += 1;
            if processed % CHUNK_SIZE == 0 {
                tokio::task::yield_now().await;
            }
        }

        // Step 6: Strip notes with yield points
        let mut after_notes = Vec::new();
        let notes_regex = regex::Regex::new(r"^♪+$").unwrap();
        processed = 0;
        for subtitle in after_cc {
            let stripped_tags = self.compiled_regexes.strip_tags(&subtitle.content);
            let stripped_content = notes_regex.replace_all(&stripped_tags, "");
            let cleaned_content = stripped_content.trim().replace(" ", "");

            if !cleaned_content.is_empty() {
                after_notes.push(subtitle);
            }

            processed += 1;
            if processed % CHUNK_SIZE == 0 {
                tokio::task::yield_now().await;
            }
        }

        // Step 7: Remove extra hyphens with yield points
        let mut after_hyphens = Vec::new();
        let hyphen_regex = regex::Regex::new(r"^(<i>|\{\\an8\})?-\s*").unwrap();
        let line_hyphen_regex = regex::Regex::new(r"(?:^|\n)(<i>|\{\\an8\})?-\s*").unwrap();
        processed = 0;
        for mut subtitle in after_notes {
            // Count occurrences of leading hyphens (at start of string or after newlines)
            let hyphen_count = line_hyphen_regex.find_iter(&subtitle.content).count();

            // Only remove the leading hyphen if there's exactly one hyphen in the entire subtitle
            if hyphen_count == 1 {
                subtitle.content = hyphen_regex
                    .replace(&subtitle.content, "$1")
                    .to_string()
                    .trim()
                    .to_string();
            }

            after_hyphens.push(subtitle);

            processed += 1;
            if processed % CHUNK_SIZE == 0 {
                tokio::task::yield_now().await;
            }
        }

        // Step 8: Apply extra regex patterns with yield points
        let mut after_extra = Vec::new();
        processed = 0;
        for mut subtitle in after_hyphens {
            for regex in &self.extra_regexes {
                subtitle.content = regex.replace_all(&subtitle.content, "").to_string();
            }

            // Filter out empty content
            if !subtitle.content.trim().is_empty() {
                after_extra.push(subtitle);
            }

            processed += 1;
            if processed % CHUNK_SIZE == 0 {
                tokio::task::yield_now().await;
            }
        }

        let mut result_srt = self.restore_bleeps(SubRipFile::new(Some(after_extra)));
        result_srt.clean_indexes();
        result_srt
    }

    /// Synchronous processing for very large files (moved to thread pool)
    fn process_large_sync(&self, srt: SubRipFile) -> SubRipFile {
        let _original_len = srt.len();

        let stripped: Vec<Subtitle> = self
            .clean_full_line_descriptions(srt)
            .flat_map(|s| self.clean_new_line_descriptions(std::iter::once(s)))
            .flat_map(|s| self.clean_inline_descriptions(std::iter::once(s)))
            .flat_map(|s| self.clean_speaker_names(std::iter::once(s)))
            .flat_map(|s| self.strip_cc_speaker_tags(std::iter::once(s)))
            .flat_map(|s| self.strip_notes(std::iter::once(s)))
            .flat_map(|s| self.remove_extra_hyphens(std::iter::once(s)))
            .flat_map(|s| self.run_extra_regexes(std::iter::once(s)))
            .filter(|s| !s.content.trim().is_empty())
            .collect();

        let mut result_srt = self.restore_bleeps(SubRipFile::new(Some(stripped)));
        result_srt.clean_indexes();
        result_srt
    }
}

fn tagged_front_description_bracket_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)^((?:</?[a-z]+>|\{\\+an8\})?-?\s*)\[[^\]]+\]:?").unwrap())
}

fn tagged_front_description_parentheses_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?m)^((?:</?[a-z]+>|\{\\+an8\})?-?\s*)\([^\)]+\):?").unwrap())
}

fn strip_speaker_prefix(line: &str, regex: &Regex, guard_timestamp: bool) -> String {
    let Some(captures) = regex.captures(line) else {
        return line.to_string();
    };
    let Some(full_match) = captures.get(0) else {
        return line.to_string();
    };

    if guard_timestamp && has_timestamp_after_speaker_prefix(line, full_match) {
        return line.to_string();
    }

    let mut cleaned = String::new();
    cleaned.push_str(capture_group_text(&captures, 2));
    cleaned.push_str(capture_group_text(&captures, 3));
    cleaned.push_str(&line[full_match.end()..]);
    cleaned
}

fn capture_group_text<'a>(captures: &'a regex::Captures<'_>, index: usize) -> &'a str {
    captures.get(index).map_or("", |capture| capture.as_str())
}

fn has_timestamp_after_speaker_prefix(line: &str, full_match: regex::Match<'_>) -> bool {
    let relative_colon_index = full_match.as_str().rfind(':');
    let absolute_colon_index = relative_colon_index.map(|index| full_match.start() + index);

    absolute_colon_index
        .and_then(|index| line.get(index + 1..))
        .is_some_and(starts_with_two_ascii_digits)
}

fn starts_with_two_ascii_digits(text: &str) -> bool {
    text.as_bytes()
        .get(0..2)
        .is_some_and(|bytes| bytes.iter().all(u8::is_ascii_digit))
}
