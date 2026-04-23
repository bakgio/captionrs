#[cfg(feature = "async")]
use std::env;
#[cfg(feature = "async")]
use std::path::PathBuf;

#[cfg(feature = "async")]
use captionrs::{AsyncBaseConverter, WebVTTConverter};

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let input = PathBuf::from(args.next().ok_or(
        "usage: cargo run --features async --example async_webvtt_to_srt -- <input.vtt> <output.srt>",
    )?);
    let output = PathBuf::from(args.next().ok_or(
        "usage: cargo run --features async --example async_webvtt_to_srt -- <input.vtt> <output.srt>",
    )?);

    let converter = WebVTTConverter::new();
    let srt = converter.from_file_async(&input).await?;
    srt.save_async(&output, Some("utf-8"), Some("\n")).await?;

    println!("Saved converted subtitles to {}", output.display());

    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the optional `async` feature.");
    std::process::exit(1);
}
