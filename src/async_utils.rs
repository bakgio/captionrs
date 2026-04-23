use crate::subripfile::SubtitleError;

/// Runs CPU-bound subtitle work on Tokio's blocking pool.
///
/// The async library APIs use this helper when parsing or processing work would
/// otherwise occupy a runtime worker thread for an extended period.
pub async fn run_blocking<T, F>(task: F) -> Result<T, SubtitleError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, SubtitleError> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|error| SubtitleError::Parse(format!("Async task failed: {error}")))?
}
