use std::fs::File;
use std::io::Write;
use std::path::Path;

#[cfg(feature = "async")]
use tokio::fs;

use chrono::TimeDelta;
use encoding_rs::Encoding;
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SubtitleError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Subtitle {
    pub index: u32,
    pub start: TimeDelta,
    pub end: TimeDelta,
    pub content: String,
}

impl Subtitle {
    pub fn new(index: u32, start: TimeDelta, end: TimeDelta, content: String) -> Self {
        Self {
            index,
            start,
            end,
            content,
        }
    }

    pub fn duration(&self) -> TimeDelta {
        self.end - self.start
    }
}

#[derive(Debug, Clone)]
pub struct SubRipFile {
    data: Vec<Subtitle>,
}

impl SubRipFile {
    pub fn new(data: Option<Vec<Subtitle>>) -> Self {
        Self {
            data: data.unwrap_or_default(),
        }
    }

    pub fn from_string(source: &str) -> Result<Self, SubtitleError> {
        let mut subtitles = Vec::new();
        let mut current_index = 0;
        let mut current_start = None;
        let mut current_end = None;
        let mut current_content = Vec::new();

        let time_regex = Regex::new(
            r"(\d{2}):(\d{2}):(\d{2})[,.](\d{3}) --> (\d{2}):(\d{2}):(\d{2})[,.](\d{3})",
        )
        .unwrap();
        let index_regex = Regex::new(r"^\d+$").unwrap();

        for line in source.lines() {
            let line = line.trim_end_matches('\r');
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            if index_regex.is_match(trimmed) {
                if current_start.is_some() && current_end.is_some() && !current_content.is_empty() {
                    push_current_subtitle(
                        &mut subtitles,
                        current_index,
                        current_start.take(),
                        current_end.take(),
                        &mut current_content,
                    );
                    current_index = 0;
                }
                current_index = trimmed.parse().unwrap_or(current_index + 1);
            } else if let Some(captures) = time_regex.captures(trimmed) {
                if current_start.is_some() && current_end.is_some() && !current_content.is_empty() {
                    push_current_subtitle(
                        &mut subtitles,
                        current_index,
                        current_start.take(),
                        current_end.take(),
                        &mut current_content,
                    );
                    current_index = 0;
                }
                let start = parse_timestamp(&captures, 1)?;
                let end = parse_timestamp(&captures, 5)?;
                current_start = Some(start);
                current_end = Some(end);
            } else if current_start.is_some() && current_end.is_some() {
                current_content.push(line.to_string());
            }
        }

        push_current_subtitle(
            &mut subtitles,
            current_index,
            current_start.take(),
            current_end.take(),
            &mut current_content,
        );

        Ok(Self::new(Some(subtitles)))
    }

    pub fn clean_indexes(&mut self) {
        self.sort();
    }

    pub fn sort(&mut self) {
        self.data.sort_by(|left, right| {
            left.start
                .cmp(&right.start)
                .then(left.end.cmp(&right.end))
                .then(left.index.cmp(&right.index))
        });
        for (i, subtitle) in self.data.iter_mut().enumerate() {
            subtitle.index = (i + 1) as u32;
        }
    }

    pub fn offset(&mut self, offset: TimeDelta) {
        for subtitle in &mut self.data {
            subtitle.start += offset;
            subtitle.end += offset;
        }
    }

    pub fn export(&self, eol: Option<&str>) -> String {
        let eol = eol.unwrap_or("\n");
        let mut result = String::new();
        let mut ordered_subtitles = self.data.clone();

        ordered_subtitles.sort_by(|left, right| {
            left.start
                .cmp(&right.start)
                .then(left.end.cmp(&right.end))
                .then(left.index.cmp(&right.index))
        });

        let exportable_subtitles = ordered_subtitles
            .iter()
            .filter(|subtitle| subtitle.start < subtitle.end)
            .collect::<Vec<_>>();

        for (position, subtitle) in exportable_subtitles.iter().enumerate() {
            let normalized_content = subtitle.content.replace("\r\n", "\n").replace('\r', "\n");
            result.push_str(&(position as u32 + 1).to_string());
            result.push_str(eol);
            result.push_str(&format_timestamp(subtitle.start));
            result.push_str(" --> ");
            result.push_str(&format_timestamp(subtitle.end));
            result.push_str(eol);
            result.push_str(&normalized_content.replace('\n', eol));
            result.push_str(eol);
            result.push_str(eol);
        }

        result
    }

    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        encoding: Option<&str>,
        eol: Option<&str>,
    ) -> Result<(), SubtitleError> {
        let mut file = File::create(path)?;
        let data = self.encoded_bytes(encoding, eol)?;
        file.write_all(&data)?;

        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn save_async<P: AsRef<Path>>(
        &self,
        path: P,
        encoding: Option<&str>,
        eol: Option<&str>,
    ) -> Result<(), SubtitleError> {
        let data = self.encoded_bytes(encoding, eol)?;
        fs::write(path, data).await?;

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn push(&mut self, subtitle: Subtitle) {
        self.data.push(subtitle);
    }

    pub fn extend(&mut self, other: SubRipFile) {
        self.data.extend(other.data);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Subtitle> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Subtitle> {
        self.data.iter_mut()
    }

    pub fn get(&self, index: usize) -> Option<&Subtitle> {
        self.data.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Subtitle> {
        self.data.get_mut(index)
    }

    fn encoded_bytes(
        &self,
        encoding: Option<&str>,
        eol: Option<&str>,
    ) -> Result<Vec<u8>, SubtitleError> {
        let content = self.export(eol);
        encode_subrip_content(&content, encoding)
    }
}

fn push_current_subtitle(
    subtitles: &mut Vec<Subtitle>,
    current_index: u32,
    current_start: Option<TimeDelta>,
    current_end: Option<TimeDelta>,
    current_content: &mut Vec<String>,
) {
    if let (Some(start), Some(end)) = (current_start, current_end)
        && !current_content.is_empty()
    {
        let index = if current_index == 0 {
            subtitles.len() as u32 + 1
        } else {
            current_index
        };

        subtitles.push(Subtitle::new(index, start, end, current_content.join("\n")));
    }

    current_content.clear();
}

impl std::ops::Index<usize> for SubRipFile {
    type Output = Subtitle;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl std::ops::IndexMut<usize> for SubRipFile {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl IntoIterator for SubRipFile {
    type Item = Subtitle;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

fn parse_timestamp(
    captures: &regex::Captures,
    start_group: usize,
) -> Result<TimeDelta, SubtitleError> {
    let hours: u32 = captures[start_group]
        .parse()
        .map_err(|e| SubtitleError::Parse(format!("Invalid hours: {}", e)))?;
    let minutes: u32 = captures[start_group + 1]
        .parse()
        .map_err(|e| SubtitleError::Parse(format!("Invalid minutes: {}", e)))?;
    let seconds: u32 = captures[start_group + 2]
        .parse()
        .map_err(|e| SubtitleError::Parse(format!("Invalid seconds: {}", e)))?;
    let milliseconds: u32 = captures[start_group + 3]
        .parse()
        .map_err(|e| SubtitleError::Parse(format!("Invalid milliseconds: {}", e)))?;

    let total_ms = (hours * 3600 + minutes * 60 + seconds) * 1000 + milliseconds;
    Ok(TimeDelta::milliseconds(total_ms as i64))
}

fn format_timestamp(delta: TimeDelta) -> String {
    let total_ms = delta.num_milliseconds();
    let hours = total_ms / 3600000;
    let minutes = (total_ms % 3600000) / 60000;
    let seconds = (total_ms % 60000) / 1000;
    let milliseconds = total_ms % 1000;

    format!(
        "{:02}:{:02}:{:02},{:03}",
        hours, minutes, seconds, milliseconds
    )
}

impl PartialEq for SubRipFile {
    fn eq(&self, other: &Self) -> bool {
        self.export(Some("\n")) == other.export(Some("\n"))
    }
}

fn encode_subrip_content(content: &str, encoding: Option<&str>) -> Result<Vec<u8>, SubtitleError> {
    let encoding = encoding.unwrap_or("utf-8");

    if encoding.eq_ignore_ascii_case("utf-8-sig") {
        let mut data = Vec::with_capacity(3 + content.len());
        data.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        data.extend_from_slice(content.as_bytes());
        return Ok(data);
    }

    if encoding.eq_ignore_ascii_case("utf-16") {
        let mut data = Vec::from([0xFF, 0xFE]);
        data.extend_from_slice(&encode_utf16_le(content));
        return Ok(data);
    }

    if encoding.eq_ignore_ascii_case("utf-16le") {
        return Ok(encode_utf16_le(content));
    }

    if encoding.eq_ignore_ascii_case("utf-16be") {
        return Ok(encode_utf16_be(content));
    }

    let encoder = Encoding::for_label(encoding.as_bytes())
        .ok_or_else(|| SubtitleError::InvalidFormat(format!("Unsupported encoding: {encoding}")))?;

    encode_strict(content, encoder, encoding)
}

fn encode_strict(
    content: &str,
    encoder: &'static Encoding,
    encoding_name: &str,
) -> Result<Vec<u8>, SubtitleError> {
    let (encoded, _, had_errors) = encoder.encode(content);
    if had_errors {
        return Err(SubtitleError::InvalidFormat(format!(
            "Content cannot be encoded as {encoding_name}"
        )));
    }

    Ok(encoded.into_owned())
}

fn encode_utf16_le(content: &str) -> Vec<u8> {
    content
        .encode_utf16()
        .flat_map(|unit| unit.to_le_bytes())
        .collect()
}

fn encode_utf16_be(content: &str) -> Vec<u8> {
    content
        .encode_utf16()
        .flat_map(|unit| unit.to_be_bytes())
        .collect()
}
