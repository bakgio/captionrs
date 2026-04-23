use std::io::Read;

use chrono::TimeDelta;
use serde::{Deserialize, Serialize};

use crate::converters::BaseConverter;
use crate::subripfile::{SubRipFile, Subtitle, SubtitleError};

#[cfg(feature = "async")]
use crate::converters::base::AsyncBaseConverter;
#[cfg(feature = "async")]
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, Deserialize, Serialize)]
struct BilibiliSubtitle {
    from: f64,
    to: f64,
    location: u32,
    content: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct BilibiliJSON {
    body: Vec<BilibiliSubtitle>,
}

#[derive(Clone)]
pub struct BilibiliJSONConverter;

impl BilibiliJSONConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BilibiliJSONConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseConverter for BilibiliJSONConverter {
    /// Bilibili JSON subtitle converter
    fn parse<R: Read>(&self, mut stream: R) -> Result<SubRipFile, SubtitleError> {
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer)?;
        self.parse_content(&buffer)
    }
}

impl BilibiliJSONConverter {
    fn parse_content(&self, buffer: &str) -> Result<SubRipFile, SubtitleError> {
        let json_data: BilibiliJSON = serde_json::from_str(buffer)
            .map_err(|e| SubtitleError::Parse(format!("Invalid JSON format: {}", e)))?;

        let mut srt = SubRipFile::new(None);

        for (i, line) in json_data.body.iter().enumerate() {
            let mut content = line.content.clone();

            if line.location != 2 {
                content = format!("{{\\an{}}}{}", line.location, content);
            }

            let start = TimeDelta::milliseconds((line.from * 1000.0) as i64);
            let end = TimeDelta::milliseconds((line.to * 1000.0) as i64);

            srt.push(Subtitle::new((i + 1) as u32, start, end, content));
        }

        Ok(srt)
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl AsyncBaseConverter for BilibiliJSONConverter {
    /// Bilibili JSON subtitle converter (async)
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
