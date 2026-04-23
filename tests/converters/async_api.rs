#[cfg(feature = "async")]
mod async_converter_tests {
    use super::super::{
        BILIBILI_ALIGNMENT_TEST, BILIBILI_API_TEST, BILIBILI_EMPTY, BILIBILI_SAMPLE,
        FILE_API_WEBVTT_SAMPLE, SAMI_API_TEST, SAMI_SAMPLE, SINGLE_FRAGMENT_WVTT_SAMPLE_SRT,
        SMPTE_FRAME_TIMING_SAMPLE, SMPTE_MULTIPLE_DOCUMENTS_SAMPLE, SMPTE_SAMPLE,
        SMPTE_TICK_TIMING_SAMPLE, WEBVTT_API_TEST, WEBVTT_SAMPLE, build_segmented_ismt_sample,
        build_segmented_wvtt_sample, build_single_fragment_wvtt_sample,
    };
    use captionrs::{
        AsyncBaseConverter, BilibiliJSONConverter, ISMTConverter, SAMIConverter, SMPTEConverter,
        WVTTConverter, WebVTTConverter,
    };
    use chrono::TimeDelta;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_webvtt_async_converter() {
        let converter = WebVTTConverter::new();
        let cursor = Cursor::new(WEBVTT_SAMPLE.as_bytes());

        let result = converter.parse_async(cursor).await;
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

    #[tokio::test]
    async fn test_smpte_async_converter() {
        let converter = SMPTEConverter::new();
        let cursor = Cursor::new(SMPTE_SAMPLE.as_bytes());

        let result = converter.parse_async(cursor).await;
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

    #[tokio::test]
    async fn test_smpte_async_converter_supports_multiple_documents() {
        let converter = SMPTEConverter::new();
        let srt = converter
            .from_string_async(SMPTE_MULTIPLE_DOCUMENTS_SAMPLE)
            .await
            .unwrap();

        assert_eq!(srt.len(), 2);
        assert_eq!(srt[0].content, "First document cue");
        assert_eq!(srt[1].content, "Second document cue");
    }

    #[tokio::test]
    async fn test_smpte_async_converter_supports_tick_timing() {
        let converter = SMPTEConverter::new();
        let srt = converter
            .from_string_async(SMPTE_TICK_TIMING_SAMPLE)
            .await
            .unwrap();

        assert_eq!(srt.len(), 1);
        assert_eq!(srt[0].start, TimeDelta::milliseconds(500));
        assert_eq!(srt[0].end, TimeDelta::milliseconds(1500));
        assert_eq!(srt[0].content, "Tick based cue");
    }

    #[tokio::test]
    async fn test_smpte_async_converter_supports_frame_timing() {
        let converter = SMPTEConverter::new();
        let srt = converter
            .from_string_async(SMPTE_FRAME_TIMING_SAMPLE)
            .await
            .unwrap();

        assert_eq!(srt.len(), 1);
        assert_eq!(srt[0].start, TimeDelta::milliseconds(1500));
        assert_eq!(srt[0].end, TimeDelta::milliseconds(2000));
        assert_eq!(srt[0].content, "Frame based cue");
    }

    #[tokio::test]
    async fn test_webvtt_async_from_string() {
        let converter = WebVTTConverter::new();

        let result = converter.from_string_async(WEBVTT_SAMPLE).await;
        assert!(result.is_ok());

        let srt = result.unwrap();
        assert_eq!(srt.len(), 3);
    }

    #[tokio::test]
    async fn test_webvtt_async_from_bytes() {
        let converter = WebVTTConverter::new();
        let data = WEBVTT_SAMPLE.as_bytes();

        let result = converter.from_bytes_async(data).await;
        assert!(result.is_ok());

        let srt = result.unwrap();
        assert_eq!(srt.len(), 3);
    }

    #[tokio::test]
    async fn test_sami_async_converter() {
        let converter = SAMIConverter::new();
        let cursor = Cursor::new(SAMI_SAMPLE.as_bytes());

        let result = converter.parse_async(cursor).await;
        assert!(result.is_ok());

        let srt = result.unwrap();
        assert!(srt.len() >= 2); // Should have at least 2 subtitles
    }

    #[tokio::test]
    async fn test_bilibili_json_async_converter() {
        let converter = BilibiliJSONConverter::new();
        let cursor = Cursor::new(BILIBILI_SAMPLE.as_bytes());

        let result = converter.parse_async(cursor).await;
        assert!(result.is_ok());

        let srt = result.unwrap();
        assert_eq!(srt.len(), 3);

        let first_subtitle = srt.get(0).unwrap();
        assert_eq!(first_subtitle.content, "Hello, world!");
    }

    #[tokio::test]
    async fn test_bilibili_alignment_does_not_insert_extra_space_async() {
        let converter = BilibiliJSONConverter::new();
        let srt = converter
            .parse_async(Cursor::new(BILIBILI_ALIGNMENT_TEST.as_bytes()))
            .await
            .unwrap();

        assert_eq!(srt.len(), 1);
        assert_eq!(srt[0].content, "{\\an8}Top aligned");
    }

    #[tokio::test]
    async fn test_ismt_segmented_mp4_converter_async_matches_segment_boundary_behavior() {
        let converter = ISMTConverter::new();
        let srt = converter
            .parse_async(Cursor::new(build_segmented_ismt_sample()))
            .await
            .unwrap();

        assert_eq!(srt.len(), 1);
        assert_eq!(srt[0].content, "First cue");
        assert_eq!(srt[0].start, TimeDelta::seconds(1));
        assert_eq!(srt[0].end, TimeDelta::seconds(2));
    }

    #[tokio::test]
    async fn test_wvtt_segmented_mp4_converter_async_matches_segment_boundary_behavior() {
        let converter = WVTTConverter::new();
        let srt = converter
            .parse_async(Cursor::new(build_segmented_wvtt_sample()))
            .await
            .unwrap();

        assert!(srt.is_empty());
    }

    #[tokio::test]
    async fn test_wvtt_single_fragment_mp4_converter_async_skips_zero_duration_cues_on_export() {
        let converter = WVTTConverter::new();
        let srt = converter
            .parse_async(Cursor::new(build_single_fragment_wvtt_sample()))
            .await
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

    #[tokio::test]
    async fn test_all_async_converter_interfaces() {
        // Verify all converters implement the async trait correctly
        use std::io::Cursor;

        // Test each converter individually since arrays need same type
        let empty_data = Vec::<u8>::new();

        // WebVTT
        let result = WebVTTConverter::new()
            .parse_async(Cursor::new(empty_data.clone()))
            .await;
        assert!(result.is_ok(), "WebVTT converter should handle empty input");

        // SMPTE
        let result = SMPTEConverter::new()
            .parse_async(Cursor::new(empty_data.clone()))
            .await;
        assert!(result.is_ok(), "SMPTE converter should handle empty input");

        // SAMI
        let result = SAMIConverter::new()
            .parse_async(Cursor::new(empty_data.clone()))
            .await;
        assert!(result.is_ok(), "SAMI converter should handle empty input");

        // BilibiliJSON (expects valid JSON, so use minimal valid JSON)
        let result = BilibiliJSONConverter::new()
            .parse_async(Cursor::new(BILIBILI_EMPTY.as_bytes()))
            .await;
        assert!(
            result.is_ok(),
            "BilibiliJSON converter should handle minimal JSON input"
        );

        // ISMT
        let result = ISMTConverter::new()
            .parse_async(Cursor::new(empty_data.clone()))
            .await;
        assert!(result.is_ok(), "ISMT converter should handle empty input");

        // WVTT
        let result = WVTTConverter::new()
            .parse_async(Cursor::new(empty_data.clone()))
            .await;
        assert!(result.is_ok(), "WVTT converter should handle empty input");
    }

    #[tokio::test]
    async fn test_sami_async_invalid_plain_text_errors() {
        let invalid_data = "This is not a SAMI file";
        let invalid_bytes = invalid_data.as_bytes();
        let converter = SAMIConverter::new();

        assert!(converter.from_string_async(invalid_data).await.is_err());
        assert!(converter.from_bytes_async(invalid_bytes).await.is_err());
        assert!(
            converter
                .parse_async(Cursor::new(invalid_bytes))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_async_mp4_converters_treat_plain_text_bytes_as_empty() {
        let invalid_bytes = b"this is not an mp4 file";

        let ismt_from_bytes = ISMTConverter::new()
            .from_bytes_async(invalid_bytes)
            .await
            .unwrap();
        let ismt_parse = ISMTConverter::new()
            .parse_async(Cursor::new(invalid_bytes))
            .await
            .unwrap();
        let wvtt_from_bytes = WVTTConverter::new()
            .from_bytes_async(invalid_bytes)
            .await
            .unwrap();
        let wvtt_parse = WVTTConverter::new()
            .parse_async(Cursor::new(invalid_bytes))
            .await
            .unwrap();

        assert!(ismt_from_bytes.is_empty());
        assert!(ismt_parse.is_empty());
        assert!(wvtt_from_bytes.is_empty());
        assert!(wvtt_parse.is_empty());
    }

    #[tokio::test]
    async fn test_async_webvtt_converter_api_methods() {
        // Test that WebVTT async converter API methods work correctly
        use std::io::Cursor;

        let webvtt_content = WEBVTT_API_TEST;
        let webvtt_bytes = webvtt_content.as_bytes();

        let converter = WebVTTConverter::new();

        // Test from_string_async
        let srt1 = converter
            .from_string_async(webvtt_content)
            .await
            .expect("Should convert from string async successfully");

        // Test from_bytes_async
        let srt2 = converter
            .from_bytes_async(webvtt_bytes)
            .await
            .expect("Should convert from bytes async successfully");

        // Test parse_async (direct method)
        let srt3 = converter
            .parse_async(Cursor::new(webvtt_bytes))
            .await
            .expect("Should parse async successfully");

        // Verify all methods produce equivalent results
        assert_eq!(
            srt1.len(),
            srt2.len(),
            "from_string_async and from_bytes_async should produce same number of subtitles"
        );
        assert_eq!(
            srt1.len(),
            srt3.len(),
            "from_string_async and parse_async should produce same number of subtitles"
        );
        assert_eq!(
            srt1.export(None),
            srt2.export(None),
            "All async methods should produce identical content"
        );
        assert_eq!(
            srt1.export(None),
            srt3.export(None),
            "All async methods should produce identical content"
        );

        println!(
            "✅ WebVTT async API methods work equivalently: {} subtitles",
            srt1.len()
        );
    }

    #[tokio::test]
    async fn test_async_sami_converter_api_methods() {
        // Test that SAMI async converter API methods work correctly
        use std::io::Cursor;

        let sami_content = SAMI_API_TEST;
        let sami_bytes = sami_content.as_bytes();

        let converter = SAMIConverter::new();

        // Test from_string_async
        let srt1 = converter
            .from_string_async(sami_content)
            .await
            .expect("Should convert SAMI from string async successfully");

        // Test from_bytes_async
        let srt2 = converter
            .from_bytes_async(sami_bytes)
            .await
            .expect("Should convert SAMI from bytes async successfully");

        // Test parse_async (direct method)
        let srt3 = converter
            .parse_async(Cursor::new(sami_bytes))
            .await
            .expect("Should parse SAMI async successfully");

        // Verify all methods produce equivalent results
        assert_eq!(
            srt1.len(),
            srt2.len(),
            "SAMI from_string_async and from_bytes_async should produce same number of subtitles"
        );
        assert_eq!(
            srt1.len(),
            srt3.len(),
            "SAMI from_string_async and parse_async should produce same number of subtitles"
        );
        assert_eq!(
            srt1.export(None),
            srt2.export(None),
            "All SAMI async methods should produce identical content"
        );

        println!(
            "✅ SAMI async API methods work equivalently: {} subtitles",
            srt1.len()
        );
    }

    #[tokio::test]
    async fn test_async_bilibili_json_converter_api_methods() {
        // Test that Bilibili JSON async converter API methods work correctly
        use std::io::Cursor;

        let json_content = BILIBILI_API_TEST;
        let json_bytes = json_content.as_bytes();

        let converter = BilibiliJSONConverter::new();

        // Test from_string_async
        let srt1 = converter
            .from_string_async(json_content)
            .await
            .expect("Should convert Bilibili JSON from string async successfully");

        // Test from_bytes_async
        let srt2 = converter
            .from_bytes_async(json_bytes)
            .await
            .expect("Should convert Bilibili JSON from bytes async successfully");

        // Test parse_async (direct method)
        let srt3 = converter
            .parse_async(Cursor::new(json_bytes))
            .await
            .expect("Should parse Bilibili JSON async successfully");

        // Verify all methods produce equivalent results
        assert_eq!(
            srt1.len(),
            srt2.len(),
            "Bilibili JSON from_string_async and from_bytes_async should produce same number of subtitles"
        );
        assert_eq!(
            srt1.len(),
            srt3.len(),
            "Bilibili JSON from_string_async and parse_async should produce same number of subtitles"
        );
        assert_eq!(
            srt1.export(None),
            srt2.export(None),
            "All Bilibili JSON async methods should produce identical content"
        );

        println!(
            "✅ Bilibili JSON async API methods work equivalently: {} subtitles",
            srt1.len()
        );
    }

    #[tokio::test]
    async fn test_async_error_handling() {
        let converter = WebVTTConverter::new();
        let invalid_data = "This is not a WebVTT file";

        let result = converter.from_string_async(invalid_data).await;
        assert!(result.is_ok()); // WebVTT converter is lenient and returns empty file for invalid data

        let srt = result.unwrap();
        assert_eq!(srt.len(), 0);
    }

    #[tokio::test]
    async fn test_async_file_operations() {
        use tempfile::NamedTempFile;

        // Create a temporary WebVTT file
        let temp_file = NamedTempFile::new().unwrap();
        tokio::fs::write(temp_file.path(), WEBVTT_SAMPLE)
            .await
            .unwrap();

        let converter = WebVTTConverter::new();
        let result = converter.from_file_async(temp_file.path()).await;
        assert!(result.is_ok());

        let srt = result.unwrap();
        assert_eq!(srt.len(), 3);
    }

    #[tokio::test]
    async fn test_async_converter_api_equivalence() {
        use tempfile::tempdir;
        use tokio::fs;

        // Equivalent to sync test but using async methods
        let temp_dir = tempdir().unwrap();
        let converter = WebVTTConverter::new();
        let file_path = temp_dir.path().join("example_test.vtt");
        fs::write(&file_path, FILE_API_WEBVTT_SAMPLE)
            .await
            .expect("Should create temp WebVTT file");

        // Test from_file_async - equivalent to: srt = converter.from_file_async(file)
        let srt1 = converter
            .from_file_async(&file_path)
            .await
            .expect("Should convert from file async successfully");

        // Test from_string_async - equivalent to: srt = converter.from_string_async(file.read_text())
        let file_content = fs::read_to_string(&file_path)
            .await
            .expect("Should read file content");
        let srt2 = converter
            .from_string_async(&file_content)
            .await
            .expect("Should convert from string async successfully");

        // Test from_bytes_async - equivalent to: srt = converter.from_bytes_async(file.read_bytes())
        let file_bytes = fs::read(&file_path).await.expect("Should read file bytes");
        let srt3 = converter
            .from_bytes_async(&file_bytes)
            .await
            .expect("Should convert from bytes async successfully");

        // Verify all three async methods produce equivalent results
        assert_eq!(
            srt1.len(),
            srt2.len(),
            "from_file_async and from_string_async should produce same number of subtitles"
        );
        assert_eq!(
            srt1.len(),
            srt3.len(),
            "from_file_async and from_bytes_async should produce same number of subtitles"
        );

        // Check we got the expected number of subtitles (same as sync test)
        assert_eq!(srt1.len(), 6, "Should have 6 subtitles from WebVTT");

        // Verify first subtitle content (same as sync test)
        assert_eq!(srt1[0].content, "Hello World");

        // Verify italic text handling (same as sync test)
        assert_eq!(srt1[1].content, "<i>This is italic text</i>");

        // Verify centered subtitle (same as sync test)
        assert_eq!(srt1[2].content, "Centered subtitle");

        // Verify speaker tag removal (same as sync test)
        assert_eq!(srt1[3].content, "Speaker name example");

        // Verify SDH content is preserved (same as sync test)
        assert_eq!(srt1[4].content, "[background music playing]");

        // Verify ruby text conversion (same as sync test)
        assert_eq!(srt1[5].content, "Line with Ruby(annotation) text");

        // Test save functionality with a temp output path.
        let output_path = temp_dir.path().join("example_test_async.srt");

        srt1.save(&output_path, Some("utf-8"), None)
            .expect("Should save SRT file successfully");

        // Verify the saved file exists and has content
        assert!(output_path.exists(), "Output SRT file should be created");

        let saved_content = fs::read_to_string(&output_path)
            .await
            .expect("Should read saved SRT file");

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

        println!("✅ Saved async SRT file to: {}", output_path.display());
        println!("✅ All WebVTT async converter methods work equivalently!");
        println!(
            "✅ Successfully converted {} subtitles with async API",
            srt1.len()
        );
    }
}
