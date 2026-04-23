use std::fs;
use std::path::Path;

use crate::subripfile::{SubRipFile, SubtitleError};

#[cfg(feature = "async")]
use tokio::fs as async_fs;

#[allow(clippy::wrong_self_convention)]
pub trait BaseProcessor {
    /// Applies subtitle cleanup or transformation passes to [`SubRipFile`] values.
    fn from_srt(
        &self,
        srt: SubRipFile,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        self.process(srt, language)
    }

    /// Reads an SRT file and processes it synchronously.
    fn from_file<P: AsRef<Path>>(
        &self,
        file: P,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        let content = fs::read_to_string(file)?;
        self.from_string(&content, language)
    }

    /// Parses an SRT string and processes it synchronously.
    fn from_string(
        &self,
        data: &str,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        let srt = SubRipFile::from_string(data)?;
        self.process(srt, language)
    }

    /// Processes a [`SubRipFile`] and reports whether the output changed.
    fn process(
        &self,
        srt: SubRipFile,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
#[allow(clippy::wrong_self_convention)]
pub trait AsyncBaseProcessor: Send + Sync {
    /// Applies subtitle cleanup or transformation passes in async applications.
    ///
    /// The async helpers perform file I/O asynchronously and move CPU-heavy parsing or
    /// processing off the runtime thread when necessary.
    async fn from_srt_async(
        &self,
        srt: SubRipFile,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        self.process_async(srt, language).await
    }

    /// Reads an SRT file asynchronously and processes it.
    async fn from_file_async<P: AsRef<Path> + Send>(
        &self,
        file: P,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        let content = async_fs::read_to_string(file).await?;
        self.from_string_async(&content, language).await
    }

    /// Parses an SRT string and processes it in an async context.
    async fn from_string_async(
        &self,
        data: &str,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        let owned_data = data.to_string();
        let srt =
            crate::async_utils::run_blocking(move || SubRipFile::from_string(&owned_data)).await?;
        self.process_async(srt, language).await
    }

    /// Processes a [`SubRipFile`] and reports whether the output changed.
    async fn process_async(
        &self,
        srt: SubRipFile,
        language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError>;
}
