use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Read;

use html_escape::decode_html_entities;
use regex::Regex;

use crate::converters::BaseConverter;
use crate::subripfile::{SubRipFile, Subtitle, SubtitleError};
use crate::utils::time::timedelta_from_timestamp;

#[cfg(feature = "async")]
use crate::converters::base::AsyncBaseConverter;
#[cfg(feature = "async")]
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Clone)]
pub struct WebVTTConverter {
    html_tag_regex: Regex,
    style_tag_regex: Regex,
    speaker_tag_regex: Regex,
    ruby_text_tag_regex: Regex,
    ruby_parenthesis_tag_regex: Regex,
}

impl WebVTTConverter {
    pub fn new() -> Self {
        Self {
            html_tag_regex: Regex::new(r"</?[^>\s]+>").unwrap(),
            style_tag_regex: Regex::new(r"<c(\.[^>]+)>([^<]+)</c>").unwrap(),
            speaker_tag_regex: Regex::new(r"<v\s+[^>]+>").unwrap(),
            ruby_text_tag_regex: Regex::new(r"<rt>([^<]+)</rt>").unwrap(),
            ruby_parenthesis_tag_regex: Regex::new(r"<rp>([^<]+)</rp>").unwrap(),
        }
    }
}

impl Default for WebVTTConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseConverter for WebVTTConverter {
    /// WebVTT subtitle converter
    fn parse<R: Read>(&self, mut stream: R) -> Result<SubRipFile, SubtitleError> {
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer)?;
        self.parse_content(&buffer)
    }
}

impl WebVTTConverter {
    /// Core parsing logic shared between sync and async implementations
    fn parse_content(&self, buffer: &str) -> Result<SubRipFile, SubtitleError> {
        let mut srt = SubRipFile::new(None);
        let mut cue_positions = Vec::new();
        let mut looking_for_text = false;
        let mut looking_for_style = false;
        let mut text = Vec::new();
        let mut position: Option<f64> = None;
        let mut line_number = 1u32;
        let mut styles: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut current_style = Vec::new();

        let skip_words = ["WEBVTT", "NOTE", "/*", "X-TIMESTAMP-MAP"];

        for line in buffer.lines() {
            let line = line.trim();

            // Skip processing any unnecessary lines
            if skip_words.iter().any(|&word| line.starts_with(word)) {
                continue;
            }

            // Empty line separates cues
            if line.is_empty() {
                // Parse current style
                if looking_for_style {
                    self.parse_css_styles(&current_style, &mut styles);
                    looking_for_style = false;
                    current_style.clear();
                }

                // Keep looking for text if last line has none
                // this will only happen if there's an unexpected line break
                if text.is_empty() {
                    continue;
                }

                if let Some(last_subtitle) = srt.get_mut(srt.len() - 1) {
                    last_subtitle.content = text.join("\n");
                }
                text.clear();
                looking_for_text = false;

            // Check for style start
            } else if line.contains("STYLE") {
                looking_for_style = true;

            // Check for style content
            } else if looking_for_style {
                current_style.push(line.to_string());

            // Check for time line
            } else if line.contains(" --> ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                position = self.get_position(
                    &parts[3..]
                        .iter()
                        .filter(|p| p.contains(':'))
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                );

                let mut start = parts[0].to_string();
                let mut end = parts[2].to_string();

                // Fix short timecodes (no hour)
                if start.matches(':').count() == 1 {
                    start = format!("00:{}", start);
                }
                if end.matches(':').count() == 1 {
                    end = format!("00:{}", end);
                }

                let start_time = timedelta_from_timestamp(&start)?;
                let end_time = timedelta_from_timestamp(&end)?;

                srt.push(Subtitle::new(
                    line_number,
                    start_time,
                    end_time,
                    String::new(),
                ));
                cue_positions.push(position.unwrap_or(100.0));

                looking_for_text = true;
                line_number += 1;

            // Append text if we're inside a line
            } else if looking_for_text {
                // Unescape html entities
                let mut line = decode_html_entities(line).to_string();

                // Remove speaker tags here
                line = self.speaker_tag_regex.replace_all(&line, "").to_string();

                // Set \an8 tag if position is below 25
                // (value taken from SubtitleEdit)
                if let Some(pos) = position
                    && pos < 25.0
                {
                    line = format!("{{\\an8}}{}", line);
                    position = None;
                }

                text.push(line.trim().to_string());
            }
        }

        // Add any leftover text to the last line
        if !text.is_empty()
            && let Some(last_subtitle) = srt.get_mut(srt.len() - 1)
        {
            last_subtitle.content = text.join("\n");
        }

        let mut sorted_subtitles = srt
            .into_iter()
            .zip(cue_positions)
            .collect::<Vec<(Subtitle, f64)>>();

        sorted_subtitles.sort_by(
            |(left_subtitle, left_position), (right_subtitle, right_position)| {
                left_subtitle
                    .start
                    .cmp(&right_subtitle.start)
                    .then(left_subtitle.end.cmp(&right_subtitle.end))
                    .then_with(|| {
                        left_position
                            .partial_cmp(right_position)
                            .unwrap_or(Ordering::Equal)
                    })
            },
        );

        let srt = SubRipFile::new(Some(
            sorted_subtitles
                .into_iter()
                .enumerate()
                .map(|(index, (mut subtitle, _))| {
                    subtitle.index = (index + 1) as u32;
                    subtitle
                })
                .collect(),
        ));

        // Post-process subtitles
        let mut srt = srt;
        for subtitle in srt.iter_mut() {
            // Replace styles with italics tag when appropriate
            subtitle.content = self.replace_style_tags(&subtitle.content, &styles);

            // Add parentheses around ruby text
            subtitle.content = self
                .ruby_text_tag_regex
                .replace_all(&subtitle.content, "($1)")
                .to_string();
            subtitle.content = self
                .ruby_parenthesis_tag_regex
                .replace_all(&subtitle.content, "")
                .to_string();

            // Strip non-italic tags
            subtitle.content = self.strip_non_italic_tags(&subtitle.content);
        }

        Ok(srt)
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl AsyncBaseConverter for WebVTTConverter {
    /// Async WebVTT subtitle converter
    async fn parse_async<R: AsyncRead + Unpin + Send>(
        &self,
        mut stream: R,
    ) -> Result<SubRipFile, SubtitleError> {
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer).await?;
        let converter = self.clone();
        crate::async_utils::run_blocking(move || converter.parse_content(&buffer)).await
    }
}

impl WebVTTConverter {
    /// Parses list of cue settings and extracts position offset as a float
    /// Line number based offset and alignment strings are ignored
    ///
    /// https://www.w3.org/TR/webvtt1/#webvtt-line-cue-setting
    fn get_position(&self, cue_settings: &[String]) -> Option<f64> {
        if cue_settings.is_empty() || (cue_settings.len() == 1 && cue_settings[0] == "None") {
            return None;
        }

        for setting in cue_settings {
            if let Some((key, val)) = setting.split_once(':')
                && key == "line"
                && !val.is_empty()
            {
                let val = val.split(',').next().unwrap_or(val);
                if let Some(percent) = val.strip_suffix('%') {
                    if let Ok(position) = percent.parse::<f64>() {
                        return Some(position);
                    }
                } else if val == "0" {
                    return Some(0.0);
                }
            }
        }

        None
    }

    fn parse_css_styles(
        &self,
        style_lines: &[String],
        styles: &mut HashMap<String, HashMap<String, String>>,
    ) {
        let css_content = style_lines.join("\n");

        let mut remaining = css_content.as_str();
        while let Some(open_brace) = remaining.find('{') {
            let selector = remaining[..open_brace].trim();
            let body_start = &remaining[open_brace + 1..];
            let Some(close_brace) = body_start.find('}') else {
                break;
            };

            let declarations = &body_start[..close_brace];
            remaining = &body_start[close_brace + 1..];

            let Some(class_name) = self.extract_class_name(selector) else {
                continue;
            };

            let mut properties = HashMap::new();
            for declaration in declarations.split(';') {
                let declaration = declaration.trim();
                if declaration.is_empty() {
                    continue;
                }

                if let Some((property, value)) = declaration.split_once(':') {
                    properties.insert(property.trim().to_string(), value.trim().to_string());
                }
            }

            styles.insert(class_name, properties);
        }
    }

    fn extract_class_name(&self, selector: &str) -> Option<String> {
        // Keep STYLE handling narrowly focused on cue class names, which is all
        // the italic replacement step consumes.
        let cue_selector = selector.split("::cue(").nth(1)?.split(')').next()?;

        cue_selector
            .split(|character: char| {
                !(character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
            })
            .find(|value| !value.is_empty())
            .map(str::to_string)
    }

    fn replace_style_tags(
        &self,
        content: &str,
        styles: &HashMap<String, HashMap<String, String>>,
    ) -> String {
        let mut replaced = content.to_string();

        loop {
            let next = self
                .style_tag_regex
                .replace_all(&replaced, |caps: &regex::Captures| {
                    let class_names = &caps[1];
                    let text_content = &caps[2];

                    for class_name in class_names.split('.').filter(|value| !value.is_empty()) {
                        if let Some(style_props) = styles.get(class_name)
                            && style_props.get("font-style").map(|value| value.as_str())
                                == Some("italic")
                        {
                            return format!("<i>{}</i>", text_content);
                        }

                        if class_name == "font-style_italic" {
                            return format!("<i>{}</i>", text_content);
                        }
                    }

                    caps[0].to_string()
                })
                .to_string();

            if next == replaced {
                return next;
            }

            replaced = next;
        }
    }

    fn strip_non_italic_tags(&self, content: &str) -> String {
        self.html_tag_regex
            .replace_all(content, |caps: &regex::Captures| {
                let tag = caps[0].to_string();
                // Keep italic tags, remove everything else
                if tag == "<i>" || tag == "</i>" {
                    tag
                } else {
                    String::new()
                }
            })
            .to_string()
    }
}
