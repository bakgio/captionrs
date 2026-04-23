use crate::processors::base::BaseProcessor;
use crate::subripfile::{SubRipFile, SubtitleError};

/// Right-to-left languages that need special handling
pub const RTL_LANGUAGES: &[&str] = &["ar", "fa", "he", "ps", "syc", "ug", "ur"];

/// RTL control characters to be removed
pub const RTL_CONTROL_CHARS: &[char] = &[
    '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}',
];

/// RTL control character to be added (Right-to-Left Mark)
pub const RTL_CHAR: char = '\u{202b}';

/// Processor for fixing right-to-left language tagging
#[derive(Debug)]
pub struct RTLFixer;

impl RTLFixer {
    pub fn new() -> Self {
        Self
    }

    fn correct_subtitles(&self, mut srt: SubRipFile) -> SubRipFile {
        for subtitle in srt.iter_mut() {
            // Remove previous RTL-related formatting
            for &control_char in RTL_CONTROL_CHARS {
                subtitle.content = subtitle.content.replace(control_char, "");
            }

            // Add RTL char at the start of every line
            subtitle.content = format!(
                "{}{}",
                RTL_CHAR,
                subtitle.content.replace('\n', &format!("\n{}", RTL_CHAR))
            );
        }

        srt
    }
}

impl Default for RTLFixer {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseProcessor for RTLFixer {
    fn process(
        &self,
        srt: SubRipFile,
        _language: Option<&str>,
    ) -> Result<(SubRipFile, bool), SubtitleError> {
        let corrected = self.correct_subtitles(srt);
        // Preserve the shared public contract for the direct RTL processor.
        Ok((corrected, false))
    }
}
