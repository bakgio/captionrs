use std::env;
use std::path::PathBuf;

use captionrs::{BaseProcessor, CommonIssuesFixer, SDHStripper};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .ok_or("usage: cargo run --example strip_sdh -- <input.srt> <output.srt> [language]")?,
    );
    let output = PathBuf::from(
        args.next()
            .ok_or("usage: cargo run --example strip_sdh -- <input.srt> <output.srt> [language]")?,
    );
    let language = args.next().and_then(|value| value.into_string().ok());

    let stripper = SDHStripper::new();
    let (stripped, changed) = stripper.from_file(&input, language.as_deref())?;

    if !changed {
        println!("No SDH descriptions were found in {}", input.display());
        return Ok(());
    }

    let mut fixer = CommonIssuesFixer::new();
    fixer.remove_gaps = true;
    let (cleaned, _) = fixer.from_srt(stripped, language.as_deref())?;
    cleaned.save(&output, Some("utf-8"), Some("\n"))?;
    println!("Saved stripped subtitles to {}", output.display());

    Ok(())
}
