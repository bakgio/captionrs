use std::env;
use std::fs;
use std::path::Path;

use captionrs::{BaseConverter, WebVTTConverter};

fn webvtt_to_srt(
    webvtt_file: impl AsRef<Path>,
    srt_file: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let converter = WebVTTConverter::new();
    let srt = converter.from_file(webvtt_file.as_ref())?;

    srt.save(srt_file.as_ref(), Some("utf-8"), Some("\n"))?;
    fs::remove_file(webvtt_file.as_ref())?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let webvtt_file = args
        .next()
        .ok_or("usage: cargo run --example webvtt_to_srt -- <input.webvtt> <output.srt>")?;
    let srt_file = args
        .next()
        .ok_or("usage: cargo run --example webvtt_to_srt -- <input.webvtt> <output.srt>")?;

    webvtt_to_srt(webvtt_file, srt_file)
}
