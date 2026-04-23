use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

use crate::subripfile::{SubRipFile, SubtitleError};

#[cfg(feature = "async")]
use tokio::fs::File as AsyncFile;
#[cfg(feature = "async")]
use tokio::io::{AsyncRead, AsyncReadExt};

#[allow(clippy::wrong_self_convention)]
pub trait BaseConverter {
    /// Converts supported subtitle inputs into [`SubRipFile`] values.
    ///
    /// These synchronous helpers fully read the input before delegating to the
    /// concrete parser implementation.
    fn from_file<P: AsRef<Path>>(&self, file: P) -> Result<SubRipFile, SubtitleError> {
        let mut file = File::open(file)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        self.parse(Cursor::new(buffer))
    }

    /// Converts an in-memory string into [`SubRipFile`].
    fn from_string(&self, data: &str) -> Result<SubRipFile, SubtitleError> {
        let bytes = data.as_bytes();
        self.parse(Cursor::new(bytes))
    }

    /// Converts an in-memory byte slice into [`SubRipFile`].
    fn from_bytes(&self, data: &[u8]) -> Result<SubRipFile, SubtitleError> {
        self.parse(Cursor::new(data))
    }

    /// Parses subtitle data from a reader.
    fn parse<R: Read>(&self, stream: R) -> Result<SubRipFile, SubtitleError>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
#[allow(clippy::wrong_self_convention)]
pub trait AsyncBaseConverter: Send + Sync {
    /// Converts supported subtitle inputs into [`SubRipFile`] values in async applications.
    ///
    /// The async helpers perform file and stream I/O asynchronously. Implementations may
    /// still move CPU-heavy parsing onto the blocking pool when that keeps the runtime responsive.
    async fn from_file_async<P: AsRef<Path> + Send>(
        &self,
        file: P,
    ) -> Result<SubRipFile, SubtitleError> {
        let mut file = AsyncFile::open(file).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        self.parse_async(Cursor::new(buffer)).await
    }

    /// Converts an in-memory string into [`SubRipFile`].
    async fn from_string_async(&self, data: &str) -> Result<SubRipFile, SubtitleError> {
        let bytes = data.as_bytes();
        self.parse_async(Cursor::new(bytes)).await
    }

    /// Converts an in-memory byte slice into [`SubRipFile`].
    async fn from_bytes_async(&self, data: &[u8]) -> Result<SubRipFile, SubtitleError> {
        self.parse_async(Cursor::new(data)).await
    }

    /// Parses subtitle data from an async reader.
    async fn parse_async<R: AsyncRead + Unpin + Send>(
        &self,
        stream: R,
    ) -> Result<SubRipFile, SubtitleError>;
}
