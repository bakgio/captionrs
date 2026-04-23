use std::env;
use std::path::PathBuf;

use captionrs::{BaseProcessor, CommonIssuesFixer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .ok_or("usage: cargo run --example mend_srt -- <input.srt> <output.srt> [language]")?,
    );
    let output = PathBuf::from(
        args.next()
            .ok_or("usage: cargo run --example mend_srt -- <input.srt> <output.srt> [language]")?,
    );
    let language = args.next().and_then(|value| value.into_string().ok());

    let mut fixer = CommonIssuesFixer::new();
    fixer.remove_gaps = true;

    let (srt, changed) = fixer.from_file(&input, language.as_deref())?;
    if changed {
        srt.save(&output, Some("utf-8"), Some("\n"))?;
        println!("Saved repaired subtitles to {}", output.display());
    } else {
        println!("No repairable issues were found in {}", input.display());
    }

    Ok(())
}
