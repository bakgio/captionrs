use std::collections::HashMap;
use std::io::Read;

use html_escape::decode_html_entities;
use regex::Regex;
use roxmltree::{Document, Node};

use crate::converters::BaseConverter;
use crate::subripfile::{SubRipFile, Subtitle, SubtitleError};
use crate::utils::time::{timedelta_from_ms, timedelta_from_timestamp};

#[cfg(feature = "async")]
use crate::converters::base::AsyncBaseConverter;
#[cfg(feature = "async")]
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Clone)]
pub struct SMPTEConverter {
    timestamp_regex: Regex,
}

impl SMPTEConverter {
    pub fn new() -> Self {
        Self {
            timestamp_regex: Regex::new(r"([0-9]{2}):([0-9]{2}):([0-9]{2})[:\.,]?([0-9]{0,3})?")
                .unwrap(),
        }
    }
}

impl Default for SMPTEConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseConverter for SMPTEConverter {
    /// DFXP/TTML/TTML2/SMPTE subtitle converter
    fn parse<R: Read>(&self, mut stream: R) -> Result<SubRipFile, SubtitleError> {
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer)?;
        self.parse_content(&buffer)
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl AsyncBaseConverter for SMPTEConverter {
    /// Async DFXP/TTML/TTML2/SMPTE subtitle converter
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

impl SMPTEConverter {
    /// Core parsing logic shared between sync and async implementations
    fn parse_content(&self, buffer: &str) -> Result<SubRipFile, SubtitleError> {
        // Handle UTF-8 BOM
        let data = buffer.strip_prefix('\u{feff}').unwrap_or(buffer);

        if data.matches("</tt>").count() == 1 {
            return self.parse_single_document(data);
        }

        // Support for multiple XML documents in a single file
        let mut srt = SubRipFile::new(None);
        let documents: Vec<String> = data
            .trim()
            .split("</tt>")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| format!("{}</tt>", s))
            .collect();

        for doc in documents {
            let sub_srt = self.parse_single_document(&doc)?;
            srt.extend(sub_srt);
        }

        Ok(srt)
    }

    fn parse_single_document(&self, data: &str) -> Result<SubRipFile, SubtitleError> {
        // Try parsing as-is first, then with HTML unescaping and common markup repair.
        match InternalSMPTEConverter::new(data, &self.timestamp_regex) {
            Ok(mut converter) => converter.convert(),
            Err(_) => {
                let unescaped = decode_html_entities(data).to_string();
                match InternalSMPTEConverter::new(&unescaped, &self.timestamp_regex) {
                    Ok(mut converter) => converter.convert(),
                    Err(_) => {
                        let repaired = repair_common_ttml_markup(&unescaped);
                        let mut converter =
                            InternalSMPTEConverter::new(&repaired, &self.timestamp_regex)?;
                        converter.convert()
                    }
                }
            }
        }
    }
}

fn repair_common_ttml_markup(data: &str) -> String {
    data.replace("</span></p>", "</p>")
}

struct InternalSMPTEConverter<'a> {
    doc: Document<'a>,
    tickrate: u32,
    frame_duration: f64,
    italics: HashMap<String, bool>,
    an8: HashMap<String, bool>,
    ruby: HashMap<String, bool>,
    all_span_italics: bool,
    timestamp_regex: &'a Regex,
}

impl<'a> InternalSMPTEConverter<'a> {
    fn new(data: &'a str, timestamp_regex: &'a Regex) -> Result<Self, SubtitleError> {
        // Try parsing as-is first
        let doc = Document::parse(data)
            .map_err(|e| SubtitleError::Parse(format!("Failed to parse XML: {}", e)))?;

        // Find root tt element
        let root = doc.root_element();
        let tt_element = if root.tag_name().name() == "tt" {
            root
        } else {
            root.descendants()
                .find(|n| n.is_element() && n.tag_name().name() == "tt")
                .ok_or_else(|| SubtitleError::Parse("No tt element found".to_string()))?
        };

        let tickrate = attribute_value(&tt_element, "tickRate")
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        let frame_duration = if let Some(rate_str) = attribute_value(&tt_element, "frameRate") {
            let rate: f64 = rate_str.parse().unwrap_or(25.0);
            let multiplier_str =
                attribute_value(&tt_element, "frameRateMultiplier").unwrap_or("1 1");
            let parts: Vec<&str> = multiplier_str.split_whitespace().collect();
            if parts.len() == 2 {
                let num: f64 = parts[0].parse().unwrap_or(1.0);
                let denom: f64 = parts[1].parse().unwrap_or(1.0);
                let framerate = rate * num / denom;
                (1.0 / framerate) * 1000.0 // ms
            } else {
                (1.0 / rate) * 1000.0
            }
        } else {
            1.0
        };

        let all_span_italics = !data.contains(r#"<span tts:fontStyle="italic">"#);

        let mut converter = Self {
            doc,
            tickrate,
            frame_duration,
            italics: HashMap::new(),
            an8: HashMap::new(),
            ruby: HashMap::new(),
            all_span_italics,
            timestamp_regex,
        };

        converter.parse_styles()?;

        Ok(converter)
    }

    fn convert(&mut self) -> Result<SubRipFile, SubtitleError> {
        let mut srt = SubRipFile::new(None);

        // Find the body/div structure
        let root = self.doc.root_element();
        let body = root
            .descendants()
            .find(|n| n.is_element() && n.tag_name().name() == "body");

        let Some(body) = body else {
            return Ok(srt);
        };

        let div = body
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "div");

        let Some(div) = div else {
            return Ok(srt);
        };

        // Process each paragraph element
        for (num, p_element) in div
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "p")
            .enumerate()
        {
            let index = (num + 1) as u32;

            // Parse timing attributes
            let begin_attr = p_element.attribute("begin");
            let end_attr = p_element.attribute("end");

            let (Some(begin_str), Some(end_str)) = (begin_attr, end_attr) else {
                continue;
            };

            let begin_time = self.parse_time_attribute(begin_str)?;
            let end_time = self.parse_time_attribute(end_str)?;

            // Parse content
            let mut line_text = String::new();
            self.parse_element_content(&p_element, &mut line_text);

            // Apply paragraph-level formatting
            if self.is_italic(&p_element) && !line_text.trim().is_empty() {
                line_text = line_text.replace("<i>", "").replace("</i>", "");
                line_text = format!("<i>{}</i>", line_text.trim());
            }

            if self.is_an8(&p_element) && !line_text.trim().is_empty() {
                line_text = format!("{{\\an8}}{}", line_text.trim());
            }

            let content = line_text.trim().trim_end_matches('\n').to_string();
            if !content.is_empty() {
                srt.push(Subtitle::new(index, begin_time, end_time, content));
            }
        }

        Ok(srt)
    }

    fn parse_styles(&mut self) -> Result<(), SubtitleError> {
        let root = self.doc.root_element();

        // Parse style elements
        for style in root
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "style")
        {
            let attrs: Vec<_> = style.attributes().map(|a| (a.name(), a.value())).collect();

            // Find the id attribute directly from the collected attributes
            // Handle both namespaced (xml:id, tts:ruby) and non-namespaced (id, ruby) forms
            let id = attrs
                .iter()
                .find(|(name, _)| *name == "id")
                .map(|(_, value)| *value);
            let ruby_attr = attrs
                .iter()
                .find(|(name, _)| *name == "ruby")
                .map(|(_, value)| *value);

            if let Some(id) = id {
                self.italics.insert(id.to_string(), self.is_italic(&style));
                if ruby_attr == Some("text") {
                    self.ruby.insert(id.to_string(), true);
                }
            }
        }

        // Parse region elements
        for region in root
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "region")
        {
            if let Some(id) = attribute_value(&region, "id") {
                self.an8.insert(id.to_string(), self.is_an8(&region));
            }
        }

        Ok(())
    }

    fn parse_element_content(&self, element: &Node, text: &mut String) {
        for child in element.children() {
            if child.is_text() {
                text.push_str(child.text().unwrap_or(""));
            } else if child.is_element() {
                let tag_name = child.tag_name().name();

                let mut child_text = String::new();
                self.parse_element_content(&child, &mut child_text);

                match tag_name {
                    "br" => {
                        text.push('\n');
                    }
                    _ => {
                        let mut formatted_text = child_text;

                        // Apply element-specific formatting
                        if self.is_italic(&child) && !formatted_text.trim().is_empty() {
                            formatted_text = formatted_text.replace("<i>", "").replace("</i>", "");
                            formatted_text = format!("<i>{}</i>", formatted_text);
                        }

                        if self.is_an8(&child) && !formatted_text.trim().is_empty() {
                            formatted_text = format!("{{\\an8}}{}", formatted_text);
                        }

                        if self.is_ruby(&child) && !formatted_text.trim().is_empty() {
                            formatted_text = format!("({})", formatted_text);
                        }

                        text.push_str(&formatted_text);
                    }
                }
            }
        }
    }

    fn is_italic(&self, element: &Node) -> bool {
        if let Some(font_style) = attribute_value(element, "fontStyle") {
            return font_style == "italic";
        }

        if let Some(style_id) = element.attribute("style")
            && let Some(&is_italic) = self.italics.get(style_id)
        {
            return is_italic;
        }

        // Special case for span elements without attributes
        if element.tag_name().name() == "span"
            && element.attributes().len() == 0
            && self.all_span_italics
            && let Some(parent) = element.parent()
        {
            return !self.is_italic(&parent);
        }

        false
    }

    fn is_an8(&self, element: &Node) -> bool {
        if let Some(display_align) = attribute_value(element, "displayAlign") {
            return display_align == "before";
        }

        if let Some(region_id) = element.attribute("region")
            && let Some(&is_an8) = self.an8.get(region_id)
        {
            return is_an8;
        }

        false
    }

    fn is_ruby(&self, element: &Node) -> bool {
        if attribute_value(element, "ruby") == Some("text") {
            return true;
        }

        if let Some(style_id) = element.attribute("style")
            && let Some(&is_ruby) = self.ruby.get(style_id)
        {
            return is_ruby;
        }

        false
    }

    fn parse_time_attribute(&self, time_str: &str) -> Result<chrono::TimeDelta, SubtitleError> {
        if time_str.ends_with('t') {
            // Tick-based timing
            self.convert_ticks(time_str)
        } else if let Some(ms_str) = time_str.strip_suffix("ms") {
            // Millisecond timing
            let ms: f64 = ms_str.parse().map_err(|_| {
                SubtitleError::Parse(format!("Invalid millisecond value: {}", ms_str))
            })?;
            Ok(timedelta_from_ms(ms))
        } else {
            // Standard timestamp or frame-based timing
            self.parse_timestamp(time_str)
        }
    }

    fn convert_ticks(&self, ticks_str: &str) -> Result<chrono::TimeDelta, SubtitleError> {
        let ticks_str = &ticks_str[..ticks_str.len() - 1]; // Remove 't'
        let ticks: u32 = ticks_str
            .parse()
            .map_err(|_| SubtitleError::Parse(format!("Invalid tick value: {}", ticks_str)))?;

        if self.tickrate == 0 {
            return Err(SubtitleError::Parse("Tickrate is zero".to_string()));
        }

        let offset = 1.0 / self.tickrate as f64;
        let seconds = offset * ticks as f64 * 1000.0; // Convert to ms

        Ok(timedelta_from_ms(seconds))
    }

    fn parse_timestamp(&self, timestamp: &str) -> Result<chrono::TimeDelta, SubtitleError> {
        if let Some(captures) = self.timestamp_regex.captures(timestamp) {
            let hours: u32 = captures
                .get(1)
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| SubtitleError::Parse(format!("Invalid hours: {}", timestamp)))?;
            let minutes: u32 = captures
                .get(2)
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| SubtitleError::Parse(format!("Invalid minutes: {}", timestamp)))?;
            let seconds: u32 = captures
                .get(3)
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| SubtitleError::Parse(format!("Invalid seconds: {}", timestamp)))?;

            let milliseconds = if let Some(frames_match) = captures.get(4) {
                let frames_str = frames_match.as_str();
                if !frames_str.is_empty() {
                    let frames: u32 = frames_str.parse().map_err(|_| {
                        SubtitleError::Parse(format!("Invalid frames: {}", timestamp))
                    })?;
                    (self.frame_duration * frames as f64) as u32
                } else {
                    0
                }
            } else {
                0
            };

            let total_ms = (hours * 3600 + minutes * 60 + seconds) * 1000 + milliseconds;
            Ok(chrono::TimeDelta::milliseconds(total_ms as i64))
        } else {
            // Try as a standard timestamp
            timedelta_from_timestamp(timestamp)
        }
    }
}

fn attribute_value<'a, 'input>(node: &Node<'a, 'input>, local_name: &str) -> Option<&'a str> {
    node.attributes()
        .find(|attribute| attribute.name() == local_name)
        .map(|attribute| attribute.value())
}
