use captionrs::{
    BaseConverter, BilibiliJSONConverter, ISMTConverter, SAMIConverter, SMPTEConverter,
    WVTTConverter, WebVTTConverter,
};
use chrono::TimeDelta;
use std::fs;
use std::io::Cursor;

use super::{
    BILIBILI_ALIGNMENT_TEST, BILIBILI_API_TEST, BILIBILI_EMPTY, BILIBILI_SAMPLE,
    BILIBILI_SAMPLE_SRT, FILE_API_WEBVTT_SAMPLE, NESTED_ITALICS_TEST, POSITION_SORT_TEST,
    SAMI_API_TEST, SAMI_SAMPLE, SAMI_SAMPLE_SRT, SEGMENTED_ISMT_SAMPLE_SRT,
    SEGMENTED_WVTT_SAMPLE_SRT, SINGLE_FRAGMENT_WVTT_SAMPLE_SRT, SMPTE_FRAME_TIMING_SAMPLE,
    SMPTE_FRAME_TIMING_SAMPLE_SRT, SMPTE_MULTIPLE_DOCUMENTS_SAMPLE,
    SMPTE_MULTIPLE_DOCUMENTS_SAMPLE_SRT, SMPTE_SAMPLE, SMPTE_SAMPLE_SRT, SMPTE_TICK_TIMING_SAMPLE,
    SMPTE_TICK_TIMING_SAMPLE_SRT, SPEAKER_TAG_TEST, WEBVTT_API_TEST, WEBVTT_SAMPLE,
    WEBVTT_SAMPLE_SRT, WEBVTT_STYLE_BLOCK_INLINE_RULE_TEST, WEBVTT_STYLE_BLOCK_SRT,
    WEBVTT_STYLE_BLOCK_TEST, build_segmented_ismt_sample, build_segmented_wvtt_sample,
    build_single_fragment_wvtt_sample,
};

#[test]
fn test_converter_api_equivalence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let converter = WebVTTConverter::new();
    let file_path = temp_dir.path().join("example_test.vtt");
    fs::write(&file_path, FILE_API_WEBVTT_SAMPLE).expect("Should create temp WebVTT file");

    // Test from_file - equivalent to: srt = converter.from_file(file)
    let srt1 = converter
        .from_file(&file_path)
        .expect("Should convert from file successfully");

    // Test from_string - equivalent to: srt = converter.from_string(file.read_text())
    let file_content = fs::read_to_string(&file_path).expect("Should read file content");
    let srt2 = converter
        .from_string(&file_content)
        .expect("Should convert from string successfully");

    // Test from_bytes - equivalent to: srt = converter.from_bytes(file.read_bytes())
    let file_bytes = fs::read(&file_path).expect("Should read file bytes");
    let srt3 = converter
        .from_bytes(&file_bytes)
        .expect("Should convert from bytes successfully");

    // Verify all three methods produce equivalent results
    assert_eq!(
        srt1.len(),
        srt2.len(),
        "from_file and from_string should produce same number of subtitles"
    );
    assert_eq!(
        srt1.len(),
        srt3.len(),
        "from_file and from_bytes should produce same number of subtitles"
    );

    // Check we got the expected number of subtitles
    assert_eq!(srt1.len(), 6, "Should have 6 subtitles from WebVTT");

    // Verify first subtitle content
    assert_eq!(srt1[0].content, "Hello World");

    // Verify italic text handling
    assert_eq!(srt1[1].content, "<i>This is italic text</i>");

    // Verify centered subtitle (no alignment tag in this implementation)
    assert_eq!(srt1[2].content, "Centered subtitle");

    // Verify speaker tag removal (just content, no speaker label formatting)
    assert_eq!(srt1[3].content, "Speaker name example");

    // Verify SDH content is preserved (converter doesn't filter SDH)
    assert_eq!(srt1[4].content, "[background music playing]");

    // Verify ruby text conversion
    assert_eq!(srt1[5].content, "Line with Ruby(annotation) text");

    // Test save functionality with a temp output path.
    let output_path = temp_dir.path().join("example_test.srt");

    srt1.save(&output_path, Some("utf-8"), None)
        .expect("Should save SRT file successfully");

    // Verify the saved file exists and has content
    assert!(output_path.exists(), "Output SRT file should be created");

    let saved_content = fs::read_to_string(&output_path).expect("Should read saved SRT file");

    assert!(
        !saved_content.is_empty(),
        "Saved SRT file should have content"
    );
    assert!(
        saved_content.contains("Hello World"),
        "Saved file should contain first subtitle"
    );
    assert!(
        saved_content.contains("00:00:01,000 --> 00:00:02,000"),
        "Saved file should contain timing"
    );

    println!("✅ Saved SRT file to: {}", output_path.display());

    println!("✅ All WebVTT converter methods work equivalently!");
    println!("✅ Successfully converted {} subtitles", srt1.len());
    println!("✅ Successfully saved to SRT format");
}

#[test]
fn test_converter_instantiation() {
    // Test that all converters implement the same interface
    let _bilibili = BilibiliJSONConverter::new();
    let _sami = SAMIConverter::new();
    let _smpte = SMPTEConverter::new();

    println!("✅ All converter types instantiate successfully");
}

#[test]
fn test_speaker_tag_stripping() {
    let converter = WebVTTConverter::new();
    let stream = Cursor::new(SPEAKER_TAG_TEST);
    let srt = converter.parse(stream).unwrap();

    // Verify that speaker tag is stripped
    assert_eq!(srt.len(), 1);
    assert_eq!(
        srt[0].content,
        "- TESTY TESTERSON:\nThis is a test, if my name isn't Testy Testerson!"
    );
}

#[test]
fn test_nested_italics_tag() {
    let converter = WebVTTConverter::new();
    let srt = converter.parse(Cursor::new(NESTED_ITALICS_TEST)).unwrap();

    assert_eq!(srt.len(), 1);
    assert_eq!(srt[0].content, "<i>He'll open up your heart</i>");
}

#[test]
fn test_style_block_italics_tag() {
    let converter = WebVTTConverter::new();
    let srt = converter.from_string(WEBVTT_STYLE_BLOCK_TEST).unwrap();

    assert_eq!(srt.export(None), WEBVTT_STYLE_BLOCK_SRT);
}

#[test]
fn test_style_block_parses_single_line_css_rule() {
    let converter = WebVTTConverter::new();
    let srt = converter
        .from_string(WEBVTT_STYLE_BLOCK_INLINE_RULE_TEST)
        .unwrap();

    assert_eq!(srt.export(None), WEBVTT_STYLE_BLOCK_SRT);
}

#[test]
fn test_sorting_same_times_by_position() {
    let converter = WebVTTConverter::new();
    let srt = converter.parse(Cursor::new(POSITION_SORT_TEST)).unwrap();

    assert_eq!(srt[0].content, "\"What is the worst thing");
    assert_eq!(srt[1].content, "that can happen in sports?\"");
    assert_eq!(srt[2].content, "that can happen in sports?\"");
    assert_eq!(srt[3].content, "\"What is the worst thing");
    assert_eq!(srt[4].content, "First line.");
    assert_eq!(srt[5].content, "Second line.");
    assert_eq!(srt[6].content, "Third line.");
}

#[test]
fn test_webvtt_converter() {
    let converter = WebVTTConverter::new();
    let cursor = Cursor::new(WEBVTT_SAMPLE.as_bytes());

    let result = converter.parse(cursor);
    assert!(result.is_ok());

    let srt = result.unwrap();
    assert_eq!(srt.len(), 3);

    let first_subtitle = srt.get(0).unwrap();
    assert_eq!(first_subtitle.content, "Hello, world!");

    let second_subtitle = srt.get(1).unwrap();
    assert_eq!(second_subtitle.content, "<i>This is italicized text.</i>");

    let third_subtitle = srt.get(2).unwrap();
    assert_eq!(third_subtitle.content, "Multiple lines\non this subtitle.");
}

#[test]
fn test_webvtt_converter_export_matches_golden_fixture() {
    let converter = WebVTTConverter::new();
    let srt = converter.from_string(WEBVTT_SAMPLE).unwrap();

    assert_eq!(srt.export(None), WEBVTT_SAMPLE_SRT);
}

#[test]
fn test_smpte_converter() {
    let converter = SMPTEConverter::new();
    let cursor = Cursor::new(SMPTE_SAMPLE.as_bytes());

    let result = converter.parse(cursor);
    assert!(result.is_ok());

    let srt = result.unwrap();
    assert_eq!(srt.len(), 3);

    let first_subtitle = srt.get(0).unwrap();
    assert_eq!(first_subtitle.content, "Hello, world!");

    let second_subtitle = srt.get(1).unwrap();
    assert_eq!(second_subtitle.content, "<i>This is italicized text.</i>");

    let third_subtitle = srt.get(2).unwrap();
    assert_eq!(third_subtitle.content, "Multiple lines\non this subtitle.");
}

#[test]
fn test_smpte_converter_export_matches_golden_fixture() {
    let converter = SMPTEConverter::new();
    let srt = converter.from_string(SMPTE_SAMPLE).unwrap();

    assert_eq!(srt.export(None), SMPTE_SAMPLE_SRT);
}

#[test]
fn test_smpte_converter_supports_multiple_documents() {
    let converter = SMPTEConverter::new();
    let srt = converter
        .from_string(SMPTE_MULTIPLE_DOCUMENTS_SAMPLE)
        .unwrap();

    assert_eq!(srt.export(None), SMPTE_MULTIPLE_DOCUMENTS_SAMPLE_SRT);
}

#[test]
fn test_smpte_converter_supports_tick_timing() {
    let converter = SMPTEConverter::new();
    let srt = converter.from_string(SMPTE_TICK_TIMING_SAMPLE).unwrap();

    assert_eq!(srt.export(None), SMPTE_TICK_TIMING_SAMPLE_SRT);
}

#[test]
fn test_smpte_converter_supports_frame_timing() {
    let converter = SMPTEConverter::new();
    let srt = converter.from_string(SMPTE_FRAME_TIMING_SAMPLE).unwrap();

    assert_eq!(srt.export(None), SMPTE_FRAME_TIMING_SAMPLE_SRT);
}

#[test]
fn test_webvtt_from_string() {
    let converter = WebVTTConverter::new();

    let result = converter.from_string(WEBVTT_SAMPLE);
    assert!(result.is_ok());

    let srt = result.unwrap();
    assert_eq!(srt.len(), 3);
}

#[test]
fn test_webvtt_from_bytes() {
    let converter = WebVTTConverter::new();
    let data = WEBVTT_SAMPLE.as_bytes();

    let result = converter.from_bytes(data);
    assert!(result.is_ok());

    let srt = result.unwrap();
    assert_eq!(srt.len(), 3);
}

#[test]
fn test_sami_converter() {
    let converter = SAMIConverter::new();
    let cursor = Cursor::new(SAMI_SAMPLE.as_bytes());

    let result = converter.parse(cursor);
    assert!(result.is_ok());

    let srt = result.unwrap();
    assert_eq!(srt.len(), 3);
    assert_eq!(srt[0].index, 1);
    assert_eq!(srt[0].start, TimeDelta::seconds(1));
    assert_eq!(srt[0].end, TimeDelta::seconds(5));
    assert_eq!(srt[1].index, 2);
    assert_eq!(srt[1].start, TimeDelta::seconds(4));
    assert_eq!(srt[1].end, TimeDelta::seconds(8));
    assert_eq!(srt[2].index, 3);
    assert_eq!(srt[2].content, "Multiple lines\non this subtitle.");
}

#[test]
fn test_sami_converter_export_matches_golden_fixture() {
    let converter = SAMIConverter::new();
    let srt = converter.from_string(SAMI_SAMPLE).unwrap();

    assert_eq!(srt.export(None), SAMI_SAMPLE_SRT);
}

#[test]
fn test_bilibili_json_converter() {
    let converter = BilibiliJSONConverter::new();
    let cursor = Cursor::new(BILIBILI_SAMPLE.as_bytes());

    let result = converter.parse(cursor);
    assert!(result.is_ok());

    let srt = result.unwrap();
    assert_eq!(srt.len(), 3);

    let first_subtitle = srt.get(0).unwrap();
    assert_eq!(first_subtitle.index, 1);
    assert_eq!(first_subtitle.content, "Hello, world!");
    assert_eq!(srt.get(1).unwrap().index, 2);
    assert_eq!(srt.get(2).unwrap().index, 3);
}

#[test]
fn test_bilibili_json_converter_export_matches_golden_fixture() {
    let converter = BilibiliJSONConverter::new();
    let srt = converter.from_string(BILIBILI_SAMPLE).unwrap();

    assert_eq!(srt.export(None), BILIBILI_SAMPLE_SRT);
}

#[test]
fn test_bilibili_alignment_does_not_insert_extra_space() {
    let converter = BilibiliJSONConverter::new();
    let srt = converter
        .parse(Cursor::new(BILIBILI_ALIGNMENT_TEST.as_bytes()))
        .unwrap();

    assert_eq!(srt.len(), 1);
    assert_eq!(srt[0].content, "{\\an8}Top aligned");
}

#[test]
fn test_ismt_segmented_mp4_converter_matches_segment_boundary_behavior() {
    let converter = ISMTConverter::new();
    let srt = converter
        .parse(Cursor::new(build_segmented_ismt_sample()))
        .unwrap();

    assert_eq!(srt.len(), 1);
    assert_eq!(srt[0].content, "First cue");
    assert_eq!(srt[0].start, TimeDelta::seconds(1));
    assert_eq!(srt[0].end, TimeDelta::seconds(2));
    assert_eq!(srt.export(None), SEGMENTED_ISMT_SAMPLE_SRT);
}

#[test]
fn test_wvtt_segmented_mp4_converter_matches_segment_boundary_behavior() {
    let converter = WVTTConverter::new();
    let srt = converter
        .parse(Cursor::new(build_segmented_wvtt_sample()))
        .unwrap();

    assert!(srt.is_empty());
    assert_eq!(srt.export(None), SEGMENTED_WVTT_SAMPLE_SRT);
}

#[test]
fn test_wvtt_single_fragment_mp4_converter_skips_zero_duration_cues_on_export() {
    let converter = WVTTConverter::new();
    let srt = converter
        .parse(Cursor::new(build_single_fragment_wvtt_sample()))
        .unwrap();

    assert_eq!(srt.len(), 2);
    assert_eq!(srt[0].content, "{\\an8}Single fragment top cue");
    assert_eq!(srt[0].start, TimeDelta::milliseconds(1500));
    assert_eq!(srt[0].end, TimeDelta::milliseconds(1500));
    assert_eq!(srt[1].content, "Single fragment plain cue");
    assert_eq!(srt[1].start, TimeDelta::milliseconds(1500));
    assert_eq!(srt[1].end, TimeDelta::milliseconds(1900));
    assert_eq!(srt.export(None), SINGLE_FRAGMENT_WVTT_SAMPLE_SRT);
}

#[test]
fn test_all_converter_interfaces() {
    // Verify all converters implement the trait correctly
    use std::io::Cursor;

    // Test each converter individually since arrays need same type
    let empty_data = Vec::<u8>::new();

    // WebVTT
    let result = WebVTTConverter::new().parse(Cursor::new(empty_data.clone()));
    assert!(result.is_ok(), "WebVTT converter should handle empty input");

    // SMPTE
    let result = SMPTEConverter::new().parse(Cursor::new(empty_data.clone()));
    assert!(result.is_ok(), "SMPTE converter should handle empty input");

    // SAMI
    let result = SAMIConverter::new().parse(Cursor::new(empty_data.clone()));
    assert!(result.is_ok(), "SAMI converter should handle empty input");

    // BilibiliJSON (expects valid JSON, so use minimal valid JSON)
    let result = BilibiliJSONConverter::new().parse(Cursor::new(BILIBILI_EMPTY.as_bytes()));
    assert!(
        result.is_ok(),
        "BilibiliJSON converter should handle minimal JSON input"
    );

    // ISMT
    let result = ISMTConverter::new().parse(Cursor::new(empty_data.clone()));
    assert!(result.is_ok(), "ISMT converter should handle empty input");

    // WVTT
    let result = WVTTConverter::new().parse(Cursor::new(empty_data.clone()));
    assert!(result.is_ok(), "WVTT converter should handle empty input");
}

#[test]
fn test_sami_invalid_plain_text_errors() {
    let invalid_data = "This is not a SAMI file";
    let invalid_bytes = invalid_data.as_bytes();
    let converter = SAMIConverter::new();

    assert!(converter.from_string(invalid_data).is_err());
    assert!(converter.from_bytes(invalid_bytes).is_err());
    assert!(converter.parse(Cursor::new(invalid_bytes)).is_err());
}

#[test]
fn test_mp4_converters_treat_plain_text_bytes_as_empty() {
    let invalid_bytes = b"this is not an mp4 file";

    let ismt_from_bytes = ISMTConverter::new().from_bytes(invalid_bytes).unwrap();
    let ismt_parse = ISMTConverter::new()
        .parse(Cursor::new(invalid_bytes))
        .unwrap();
    let wvtt_from_bytes = WVTTConverter::new().from_bytes(invalid_bytes).unwrap();
    let wvtt_parse = WVTTConverter::new()
        .parse(Cursor::new(invalid_bytes))
        .unwrap();

    assert!(ismt_from_bytes.is_empty());
    assert!(ismt_parse.is_empty());
    assert!(wvtt_from_bytes.is_empty());
    assert!(wvtt_parse.is_empty());
}

#[test]
fn test_webvtt_converter_api_methods() {
    // Test that WebVTT converter API methods work correctly
    use std::io::Cursor;

    let webvtt_content = WEBVTT_API_TEST;
    let webvtt_bytes = webvtt_content.as_bytes();

    let converter = WebVTTConverter::new();

    // Test from_string
    let srt1 = converter
        .from_string(webvtt_content)
        .expect("Should convert from string successfully");

    // Test from_bytes
    let srt2 = converter
        .from_bytes(webvtt_bytes)
        .expect("Should convert from bytes successfully");

    // Test parse (direct method)
    let srt3 = converter
        .parse(Cursor::new(webvtt_bytes))
        .expect("Should parse successfully");

    // Verify all methods produce equivalent results
    assert_eq!(
        srt1.len(),
        srt2.len(),
        "from_string and from_bytes should produce same number of subtitles"
    );
    assert_eq!(
        srt1.len(),
        srt3.len(),
        "from_string and parse should produce same number of subtitles"
    );
    assert_eq!(
        srt1.export(None),
        srt2.export(None),
        "All methods should produce identical content"
    );
    assert_eq!(
        srt1.export(None),
        srt3.export(None),
        "All methods should produce identical content"
    );

    println!(
        "✅ WebVTT API methods work equivalently: {} subtitles",
        srt1.len()
    );
}

#[test]
fn test_sami_converter_api_methods() {
    // Test that SAMI converter API methods work correctly
    use std::io::Cursor;

    let sami_content = SAMI_API_TEST;
    let sami_bytes = sami_content.as_bytes();

    let converter = SAMIConverter::new();

    // Test from_string
    let srt1 = converter
        .from_string(sami_content)
        .expect("Should convert SAMI from string successfully");

    // Test from_bytes
    let srt2 = converter
        .from_bytes(sami_bytes)
        .expect("Should convert SAMI from bytes successfully");

    // Test parse (direct method)
    let srt3 = converter
        .parse(Cursor::new(sami_bytes))
        .expect("Should parse SAMI successfully");

    // Verify all methods produce equivalent results
    assert_eq!(
        srt1.len(),
        srt2.len(),
        "SAMI from_string and from_bytes should produce same number of subtitles"
    );
    assert_eq!(
        srt1.len(),
        srt3.len(),
        "SAMI from_string and parse should produce same number of subtitles"
    );
    assert_eq!(
        srt1.export(None),
        srt2.export(None),
        "All SAMI methods should produce identical content"
    );

    println!(
        "✅ SAMI API methods work equivalently: {} subtitles",
        srt1.len()
    );
}

#[test]
fn test_bilibili_json_converter_api_methods() {
    // Test that Bilibili JSON converter API methods work correctly
    use std::io::Cursor;

    let json_content = BILIBILI_API_TEST;
    let json_bytes = json_content.as_bytes();

    let converter = BilibiliJSONConverter::new();

    // Test from_string
    let srt1 = converter
        .from_string(json_content)
        .expect("Should convert Bilibili JSON from string successfully");

    // Test from_bytes
    let srt2 = converter
        .from_bytes(json_bytes)
        .expect("Should convert Bilibili JSON from bytes successfully");

    // Test parse (direct method)
    let srt3 = converter
        .parse(Cursor::new(json_bytes))
        .expect("Should parse Bilibili JSON successfully");

    // Verify all methods produce equivalent results
    assert_eq!(
        srt1.len(),
        srt2.len(),
        "Bilibili JSON from_string and from_bytes should produce same number of subtitles"
    );
    assert_eq!(
        srt1.len(),
        srt3.len(),
        "Bilibili JSON from_string and parse should produce same number of subtitles"
    );
    assert_eq!(
        srt1.export(None),
        srt2.export(None),
        "All Bilibili JSON methods should produce identical content"
    );

    println!(
        "✅ Bilibili JSON API methods work equivalently: {} subtitles",
        srt1.len()
    );
}

#[test]
fn test_error_handling() {
    let converter = WebVTTConverter::new();
    let invalid_data = "This is not a WebVTT file";

    let result = converter.from_string(invalid_data);
    assert!(result.is_ok()); // WebVTT converter is lenient and returns empty file for invalid data

    let srt = result.unwrap();
    assert_eq!(srt.len(), 0);
}
