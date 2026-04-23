use std::env;
use std::path::PathBuf;

use captionrs::{BaseConverter, BaseProcessor, CommonIssuesFixer, WebVTTConverter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let input = PathBuf::from(args.next().ok_or(
        "usage: cargo run --example webvtt_pipeline -- <input.vtt> <output.srt> [language]",
    )?);
    let output = PathBuf::from(args.next().ok_or(
        "usage: cargo run --example webvtt_pipeline -- <input.vtt> <output.srt> [language]",
    )?);
    let language = args.next().and_then(|value| value.into_string().ok());

    let converter = WebVTTConverter::new();
    let mut fixer = CommonIssuesFixer::new();
    fixer.remove_gaps = true;

    let converted = converter.from_file(&input)?;
    let (cleaned, _) = fixer.from_srt(converted, language.as_deref())?;
    cleaned.save(&output, Some("utf-8"), Some("\n"))?;

    println!("Saved converted subtitles to {}", output.display());

    Ok(())
}
