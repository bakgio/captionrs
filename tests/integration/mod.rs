// Common integration test utilities and shared test data

mod end_to_end;
mod error_handling;
mod performance;

// Import shared test constants for integration tests
pub const WEBVTT_SAMPLE: &str = r#"WEBVTT

1
00:00:01.000 --> 00:00:03.000
Hello, world!

2
00:00:04.000 --> 00:00:06.000
<i>This is italicized text.</i>

3
00:00:07.000 --> 00:00:09.000
Multiple lines
on this subtitle."#;

pub const SMPTE_SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" 
    xmlns:tts="http://www.w3.org/ns/ttml#styling"
    xml:lang="en">
  <head>
    <styling>
      <style xml:id="italic" tts:fontStyle="italic"/>
    </styling>
  </head>
  <body>
    <div>
      <p begin="00:00:01.000" end="00:00:03.000">Hello, world!</p>
      <p begin="00:00:04.000" end="00:00:06.000" style="italic">This is italicized text.</p>
      <p begin="00:00:07.000" end="00:00:09.000">Multiple lines<br/>on this subtitle.</p>
    </div>
  </body>
</tt>"#;

// Helper functions for performance testing
pub fn create_large_webvtt_sample(count: usize) -> String {
    let mut large_webvtt = String::from("WEBVTT\n\n");
    for i in 1..=count {
        large_webvtt.push_str(&format!(
            "{}\n{:02}:{:02}:{:02}.000 --> {:02}:{:02}:{:02}.000\nSubtitle line {}\n\n",
            i,
            i / 3600,
            (i % 3600) / 60,
            i % 60,
            i / 3600,
            ((i + 2) % 3600) / 60,
            (i + 2) % 60,
            i
        ));
    }
    large_webvtt
}

// Common test SRT content for processor chains
pub fn create_test_srt_with_issues() -> String {
    r#"1
00:00:01,000 --> 00:00:03,000
Hello,  world!

2
00:00:04,000 --> 00:00:06,000
(music playing)

3
00:00:07,000 --> 00:00:09,000
This  has  spaces."#
        .to_string()
}
