#[cfg(feature = "async")]
mod async_utils;
mod cli;
pub mod converters;
pub mod processors;
pub mod regex;
pub mod subripfile;
pub mod utils;

pub use converters::{
    BaseConverter, BilibiliJSONConverter, ISMTConverter, SAMIConverter, SMPTEConverter,
    WVTTConverter, WebVTTConverter,
};
pub use processors::{BaseProcessor, CommonIssuesFixer, SDHStripper};

/// Synchronous CLI entrypoint used by the `captionrs` binary.
pub use cli::run as run_cli;
#[cfg(feature = "async")]
/// Async converter trait for library integrations that run inside Tokio.
pub use converters::base::AsyncBaseConverter;
#[cfg(feature = "async")]
/// Async processor trait for library integrations that run inside Tokio.
pub use processors::base::AsyncBaseProcessor;
pub use subripfile::SubRipFile;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
