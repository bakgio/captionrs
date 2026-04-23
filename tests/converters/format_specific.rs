use super::{TTML_RUBY_SAMPLE, WEBVTT_RUBY_SAMPLE};
use captionrs::converters::{BaseConverter, SMPTEConverter, WebVTTConverter};
use std::io::Cursor;

#[test]
fn test_webvtt_ruby_handling() {
    let converter = WebVTTConverter::new();
    let stream = Cursor::new(WEBVTT_RUBY_SAMPLE.as_bytes());
    let srt = converter.parse(stream).unwrap();

    assert_eq!(srt.len(), 1);
    assert_eq!(
        srt[0].content,
        "第二九龍(クーロン)って 決して\n住みやすい場所じゃないと思うけど"
    );
}

#[test]
fn test_ttml_ruby_handling() {
    let converter = SMPTEConverter::new();
    let stream = Cursor::new(TTML_RUBY_SAMPLE.as_bytes());
    let srt = converter.parse(stream).unwrap();

    assert_eq!(srt.len(), 1);
    assert_eq!(srt[0].content, "（風子(ふうこ)）あっ…");
}
