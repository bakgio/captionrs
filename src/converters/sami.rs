use std::collections::HashMap;
use std::io::Read;

use crate::converters::BaseConverter;
use crate::subripfile::{SubRipFile, Subtitle, SubtitleError};
use crate::utils::time::timedelta_from_ms;

#[cfg(feature = "async")]
use crate::converters::base::AsyncBaseConverter;
#[cfg(feature = "async")]
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Clone)]
pub struct SAMIConverter;

impl SAMIConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SAMIConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseConverter for SAMIConverter {
    /// SAMI subtitle converter
    fn parse<R: Read>(&self, mut stream: R) -> Result<SubRipFile, SubtitleError> {
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer)?;

        // Remove BOM if present
        let content = buffer.strip_prefix('\u{feff}').unwrap_or(&buffer);

        SAMIParser::new(content).parse()
    }
}

struct SAMIParser {
    lines: Vec<SAMILine>,
    tags: Vec<Tag>,
    line_list: Vec<ProcessedLine>,
    saw_text_before_first_tag: bool,
}

#[derive(Debug, Clone)]
struct SAMILine {
    text: String,
    start: Option<f64>,
    end: Option<f64>,
    attributes: HashMap<String, String>,
}

impl SAMILine {}

#[derive(Debug, Clone)]
struct Tag {
    name: String,
}

#[derive(Debug, Clone)]
struct ProcessedLine {
    start: f64,
    end: f64,
    content: String,
}

impl SAMIParser {
    fn new(content: &str) -> Self {
        let mut parser = Self {
            lines: Vec::new(),
            tags: Vec::new(),
            line_list: Vec::new(),
            saw_text_before_first_tag: false,
        };

        let corrected_content = correct_tags(content);
        parser.feed(&corrected_content);
        parser
    }

    fn parse(mut self) -> Result<SubRipFile, SubtitleError> {
        if self.saw_text_before_first_tag {
            return Err(SubtitleError::Parse(
                "Invalid SAMI content before the first tag".to_string(),
            ));
        }

        self.convert();

        let mut srt = SubRipFile::new(None);

        for (num, line) in self.line_list.iter().enumerate() {
            let subtitle = Subtitle::new(
                (num + 1) as u32,
                timedelta_from_ms(line.start),
                timedelta_from_ms(line.end),
                line.content.clone(),
            );
            srt.push(subtitle);
        }

        Ok(srt)
    }

    fn feed(&mut self, content: &str) {
        let mut pos = 0;
        let chars: Vec<char> = content.chars().collect();

        while pos < chars.len() {
            if chars[pos] == '<' {
                // Find the end of the tag
                let mut end_pos = pos + 1;
                while end_pos < chars.len() && chars[end_pos] != '>' {
                    end_pos += 1;
                }

                if end_pos < chars.len() {
                    let tag_content: String = chars[pos + 1..end_pos].iter().collect();
                    self.parse_tag(&tag_content);
                    pos = end_pos + 1;
                } else {
                    // Malformed tag, treat as text
                    self.handle_data(&chars[pos].to_string());
                    pos += 1;
                }
            } else {
                // Collect text until next tag
                let mut text = String::new();
                while pos < chars.len() && chars[pos] != '<' {
                    text.push(chars[pos]);
                    pos += 1;
                }
                if !text.is_empty() {
                    self.handle_data(&text);
                }
            }
        }
    }

    fn parse_tag(&mut self, tag_content: &str) {
        let tag_content = tag_content.trim();

        if let Some(stripped_tag) = tag_content.strip_prefix('/') {
            // End tag
            let tag_name = &stripped_tag.trim().to_lowercase();
            self.handle_endtag(tag_name);
        } else {
            // Start tag - find the first space to separate tag name from attributes
            let (tag_name, attrs_str) =
                if let Some(space_pos) = tag_content.find(char::is_whitespace) {
                    let name = tag_content[..space_pos].trim().to_lowercase();
                    let attrs = tag_content[space_pos..].trim();
                    (name, attrs)
                } else {
                    (tag_content.to_lowercase(), "")
                };

            let attrs = self.parse_attributes(attrs_str);
            self.handle_starttag(&tag_name, attrs);
        }
    }

    fn parse_attributes(&self, attrs_str: &str) -> HashMap<String, String> {
        let mut attrs = HashMap::new();

        if attrs_str.is_empty() {
            return attrs;
        }

        let mut chars = attrs_str.chars().peekable();

        while chars.peek().is_some() {
            // Skip whitespace
            while chars.peek().is_some_and(|c| c.is_whitespace()) {
                chars.next();
            }

            if chars.peek().is_none() {
                break;
            }

            // Parse attribute name
            let mut name = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() || ch == '=' {
                    break;
                }
                name.push(chars.next().unwrap());
            }

            if name.is_empty() {
                break;
            }

            // Skip whitespace and check for '='
            while chars.peek().is_some_and(|c| c.is_whitespace()) {
                chars.next();
            }

            if chars.peek() != Some(&'=') {
                // Attribute without value
                attrs.insert(name.to_lowercase(), String::new());
                continue;
            }

            chars.next(); // consume '='

            // Skip whitespace after '='
            while chars.peek().is_some_and(|c| c.is_whitespace()) {
                chars.next();
            }

            // Parse attribute value
            let mut value = String::new();

            if let Some(&quote_char) = chars.peek() {
                if quote_char == '"' || quote_char == '\'' {
                    chars.next(); // consume opening quote

                    for ch in chars.by_ref() {
                        if ch == quote_char {
                            break;
                        }
                        value.push(ch);
                    }
                } else {
                    // Unquoted value - read until whitespace
                    while let Some(&ch) = chars.peek() {
                        if ch.is_whitespace() {
                            break;
                        }
                        value.push(chars.next().unwrap());
                    }
                }
            }

            attrs.insert(name.to_lowercase(), value);
        }

        attrs
    }

    fn handle_starttag(&mut self, tag: &str, attrs: HashMap<String, String>) {
        if tag == "sync" {
            let mut data = SAMILine {
                text: String::new(),
                start: None,
                end: None,
                attributes: HashMap::new(),
            };

            // Extract start and end times while preserving all attributes
            if let Some(start_val) = attrs.get("start")
                && let Ok(start_time) = start_val.parse::<f64>()
            {
                data.start = Some(start_time);
            }

            if let Some(end_val) = attrs.get("end")
                && let Ok(end_time) = end_val.parse::<f64>()
            {
                data.end = Some(end_time);
            }

            // Preserve ALL attributes
            data.attributes = attrs.clone();

            self.lines.push(data);
        }

        self.tags.push(Tag {
            name: tag.to_string(),
        });
    }

    fn handle_endtag(&mut self, _tag: &str) {
        // Keep handle_endtag minimal
    }

    fn handle_data(&mut self, data: &str) {
        if self.tags.is_empty() {
            self.saw_text_before_first_tag = true;
            return;
        }

        let last_tag = &self.tags.last().unwrap().name;

        if last_tag == "br" {
            if let Some(last_line) = self.lines.last_mut() {
                last_line.text.push('\n');
            }
            return;
        }

        if last_tag == "i" && !data.trim().is_empty() {
            if let Some(last_line) = self.lines.last_mut() {
                last_line.text.push_str(&format!("<i>{}</i>", data));
            }
            return;
        }

        if last_tag != "sync"
            && !self.lines.is_empty()
            && let Some(last_line) = self.lines.last_mut()
        {
            last_line.text.push_str(data);
        }
    }

    fn convert(&mut self) {
        for line in &self.lines {
            if line.text.trim().is_empty() {
                if let Some(end_time) = line.start
                    && let Some(previous_line) = self.line_list.last_mut()
                {
                    previous_line.end = end_time;
                }
                continue;
            }

            let Some(start_time) = line.start else {
                continue;
            };

            let end_time = line.end.unwrap_or(start_time + 4000.0);
            self.line_list.push(ProcessedLine {
                start: start_time,
                end: end_time,
                content: line.text.trim().to_string(),
            });
        }
    }
}

fn correct_tags(data: &str) -> String {
    let mut corrected = data.to_string();

    // Apply tag corrections
    corrected = corrected.replace("<i/>", "<i>");
    corrected = corrected.replace(";>", ">");
    corrected = corrected.replace("<br>", "\n");
    corrected = corrected.replace("<br/>", "\n");
    corrected = corrected.replace("<br >", "\n");

    corrected
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl AsyncBaseConverter for SAMIConverter {
    /// SAMI subtitle converter (async)
    async fn parse_async<R: AsyncRead + Unpin + Send>(
        &self,
        mut stream: R,
    ) -> Result<SubRipFile, SubtitleError> {
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer).await?;

        crate::async_utils::run_blocking(move || {
            let content = buffer.strip_prefix('\u{feff}').unwrap_or(&buffer);
            SAMIParser::new(content).parse()
        })
        .await
    }
}
