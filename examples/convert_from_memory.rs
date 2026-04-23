use captionrs::{BaseConverter, WebVTTConverter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"WEBVTT

00:00:01.000 --> 00:00:03.000
Hello from memory.

00:00:04.000 --> 00:00:06.000
Second cue.
"#;

    let converter = WebVTTConverter::new();
    let srt = converter.from_string(source)?;

    println!("{}", srt.export(Some("\n")));

    Ok(())
}
