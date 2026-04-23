#[cfg(feature = "async")]
use captionrs::{
    AsyncBaseConverter, AsyncBaseProcessor, BaseConverter, BaseProcessor, BilibiliJSONConverter,
    CommonIssuesFixer, ISMTConverter, SAMIConverter, SDHStripper, SMPTEConverter, WVTTConverter,
    WebVTTConverter,
};

#[cfg(feature = "async")]
mod sync_async_equivalence_tests {
    use super::super::{
        BILIBILI_EQUIVALENCE, SAMI_EQUIVALENCE, SMPTE_EQUIVALENCE, WEBVTT_EQUIVALENCE,
        build_segmented_ismt_sample, build_segmented_wvtt_sample,
    };
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_webvtt_sync_async_equivalence() {
        let webvtt_content = WEBVTT_EQUIVALENCE;

        let converter = WebVTTConverter::new();

        // Test sync vs async for all API methods
        let sync_from_string = converter.from_string(webvtt_content).unwrap();
        let async_from_string = converter.from_string_async(webvtt_content).await.unwrap();

        let sync_from_bytes = converter.from_bytes(webvtt_content.as_bytes()).unwrap();
        let async_from_bytes = converter
            .from_bytes_async(webvtt_content.as_bytes())
            .await
            .unwrap();

        let sync_parse = converter
            .parse(Cursor::new(webvtt_content.as_bytes()))
            .unwrap();
        let async_parse = converter
            .parse_async(Cursor::new(webvtt_content.as_bytes()))
            .await
            .unwrap();

        // All sync and async results should be identical
        assert_eq!(
            sync_from_string.export(None),
            async_from_string.export(None),
            "WebVTT from_string sync vs async"
        );
        assert_eq!(
            sync_from_bytes.export(None),
            async_from_bytes.export(None),
            "WebVTT from_bytes sync vs async"
        );
        assert_eq!(
            sync_parse.export(None),
            async_parse.export(None),
            "WebVTT parse sync vs async"
        );

        println!("✅ WebVTT converter: sync and async produce identical results");
    }

    #[tokio::test]
    async fn test_sami_sync_async_equivalence() {
        let sami_content = SAMI_EQUIVALENCE;

        let converter = SAMIConverter::new();

        // Test sync vs async for all API methods
        let sync_from_string = converter.from_string(sami_content).unwrap();
        let async_from_string = converter.from_string_async(sami_content).await.unwrap();

        let sync_from_bytes = converter.from_bytes(sami_content.as_bytes()).unwrap();
        let async_from_bytes = converter
            .from_bytes_async(sami_content.as_bytes())
            .await
            .unwrap();

        let sync_parse = converter
            .parse(Cursor::new(sami_content.as_bytes()))
            .unwrap();
        let async_parse = converter
            .parse_async(Cursor::new(sami_content.as_bytes()))
            .await
            .unwrap();

        // All sync and async results should be identical
        assert_eq!(
            sync_from_string.export(None),
            async_from_string.export(None),
            "SAMI from_string sync vs async"
        );
        assert_eq!(
            sync_from_bytes.export(None),
            async_from_bytes.export(None),
            "SAMI from_bytes sync vs async"
        );
        assert_eq!(
            sync_parse.export(None),
            async_parse.export(None),
            "SAMI parse sync vs async"
        );

        println!("✅ SAMI converter: sync and async produce identical results");
    }

    #[tokio::test]
    async fn test_bilibili_json_sync_async_equivalence() {
        let json_content = BILIBILI_EQUIVALENCE;

        let converter = BilibiliJSONConverter::new();

        // Test sync vs async for all API methods
        let sync_from_string = converter.from_string(json_content).unwrap();
        let async_from_string = converter.from_string_async(json_content).await.unwrap();

        let sync_from_bytes = converter.from_bytes(json_content.as_bytes()).unwrap();
        let async_from_bytes = converter
            .from_bytes_async(json_content.as_bytes())
            .await
            .unwrap();

        let sync_parse = converter
            .parse(Cursor::new(json_content.as_bytes()))
            .unwrap();
        let async_parse = converter
            .parse_async(Cursor::new(json_content.as_bytes()))
            .await
            .unwrap();

        // All sync and async results should be identical
        assert_eq!(
            sync_from_string.export(None),
            async_from_string.export(None),
            "Bilibili JSON from_string sync vs async"
        );
        assert_eq!(
            sync_from_bytes.export(None),
            async_from_bytes.export(None),
            "Bilibili JSON from_bytes sync vs async"
        );
        assert_eq!(
            sync_parse.export(None),
            async_parse.export(None),
            "Bilibili JSON parse sync vs async"
        );

        println!("✅ Bilibili JSON converter: sync and async produce identical results");
    }

    #[tokio::test]
    async fn test_smpte_sync_async_equivalence() {
        let smpte_content = SMPTE_EQUIVALENCE;

        let converter = SMPTEConverter::new();

        // Test sync vs async for all API methods
        let sync_from_string = converter.from_string(smpte_content).unwrap();
        let async_from_string = converter.from_string_async(smpte_content).await.unwrap();

        let sync_from_bytes = converter.from_bytes(smpte_content.as_bytes()).unwrap();
        let async_from_bytes = converter
            .from_bytes_async(smpte_content.as_bytes())
            .await
            .unwrap();

        let sync_parse = converter
            .parse(Cursor::new(smpte_content.as_bytes()))
            .unwrap();
        let async_parse = converter
            .parse_async(Cursor::new(smpte_content.as_bytes()))
            .await
            .unwrap();

        // All sync and async results should be identical
        assert_eq!(
            sync_from_string.export(None),
            async_from_string.export(None),
            "SMPTE from_string sync vs async"
        );
        assert_eq!(
            sync_from_bytes.export(None),
            async_from_bytes.export(None),
            "SMPTE from_bytes sync vs async"
        );
        assert_eq!(
            sync_parse.export(None),
            async_parse.export(None),
            "SMPTE parse sync vs async"
        );

        println!("✅ SMPTE converter: sync and async produce identical results");
    }

    #[tokio::test]
    async fn test_mp4_converters_sync_async_equivalence() {
        // ISMT Converter
        let ismt_converter = ISMTConverter::new();
        let sync_ismt = ismt_converter
            .parse(Cursor::new(build_segmented_ismt_sample()))
            .unwrap();
        let async_ismt = ismt_converter
            .parse_async(Cursor::new(build_segmented_ismt_sample()))
            .await
            .unwrap();
        assert_eq!(
            sync_ismt.export(None),
            async_ismt.export(None),
            "ISMT converter sync vs async"
        );

        // WVTT Converter
        let wvtt_converter = WVTTConverter::new();
        let sync_wvtt = wvtt_converter
            .parse(Cursor::new(build_segmented_wvtt_sample()))
            .unwrap();
        let async_wvtt = wvtt_converter
            .parse_async(Cursor::new(build_segmented_wvtt_sample()))
            .await
            .unwrap();
        assert_eq!(
            sync_wvtt.export(None),
            async_wvtt.export(None),
            "WVTT converter sync vs async"
        );

        println!("✅ MP4 converters (ISMT, WVTT): sync and async produce identical results");
    }

    #[tokio::test]
    async fn test_common_issues_fixer_sync_async_equivalence() {
        use captionrs::SubRipFile;
        use chrono::TimeDelta;

        // Create test data with common issues
        let mut test_srt = SubRipFile::new(None);
        test_srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::milliseconds(1000),
            TimeDelta::milliseconds(3000),
            "Hello,  world   with  extra  spaces!".to_string(),
        ));
        test_srt.push(captionrs::subripfile::Subtitle::new(
            2,
            TimeDelta::milliseconds(4000),
            TimeDelta::milliseconds(6000),
            "£ Musical  notes  and  it'`s  quotes".to_string(),
        ));
        test_srt.push(captionrs::subripfile::Subtitle::new(
            3,
            TimeDelta::milliseconds(7000),
            TimeDelta::milliseconds(9000),
            "Text with&amp;leftovers".to_string(),
        ));

        let fixer = CommonIssuesFixer::new();

        // Test sync vs async
        let (sync_result, sync_changed) = fixer.process(test_srt.clone(), None).unwrap();
        let (async_result, async_changed) = fixer.process_async(test_srt, None).await.unwrap();

        // Results should be identical
        assert_eq!(
            sync_changed, async_changed,
            "CommonIssuesFixer changed flag should match"
        );
        assert_eq!(
            sync_result.len(),
            async_result.len(),
            "CommonIssuesFixer result length should match"
        );
        assert_eq!(
            sync_result.export(None),
            async_result.export(None),
            "CommonIssuesFixer sync vs async results"
        );

        // Verify processing actually worked
        assert!(sync_changed, "Should have detected changes");
        assert_eq!(
            sync_result.get(0).unwrap().content,
            "Hello, world with extra spaces!"
        );
        assert_eq!(
            sync_result.get(1).unwrap().content,
            "♪ Musical notes and it's quotes"
        );
        assert_eq!(sync_result.get(2).unwrap().content, "Text with&leftovers");

        println!("✅ CommonIssuesFixer processor: sync and async produce identical results");
    }

    #[tokio::test]
    async fn test_sdh_stripper_sync_async_equivalence() {
        use captionrs::SubRipFile;
        use chrono::TimeDelta;

        // Create test data with SDH elements
        let mut test_srt = SubRipFile::new(None);
        test_srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::milliseconds(1000),
            TimeDelta::milliseconds(3000),
            "Hello, world!".to_string(),
        ));
        test_srt.push(captionrs::subripfile::Subtitle::new(
            2,
            TimeDelta::milliseconds(4000),
            TimeDelta::milliseconds(6000),
            "[background music playing]".to_string(),
        ));
        test_srt.push(captionrs::subripfile::Subtitle::new(
            3,
            TimeDelta::milliseconds(7000),
            TimeDelta::milliseconds(9000),
            "NARRATOR: Another line of text".to_string(),
        ));
        test_srt.push(captionrs::subripfile::Subtitle::new(
            4,
            TimeDelta::milliseconds(10000),
            TimeDelta::milliseconds(12000),
            ">> More dialogue here".to_string(),
        ));
        test_srt.push(captionrs::subripfile::Subtitle::new(
            5,
            TimeDelta::milliseconds(13000),
            TimeDelta::milliseconds(15000),
            "♪ Musical notes only ♪".to_string(),
        ));

        let stripper = SDHStripper::new();

        // Test sync vs async
        let (sync_result, sync_changed) = stripper.process(test_srt.clone(), None).unwrap();
        let (async_result, async_changed) = stripper.process_async(test_srt, None).await.unwrap();

        // Results should be identical
        assert_eq!(
            sync_changed, async_changed,
            "SDHStripper changed flag should match"
        );
        assert_eq!(
            sync_result.len(),
            async_result.len(),
            "SDHStripper result length should match"
        );
        assert_eq!(
            sync_result.export(None),
            async_result.export(None),
            "SDHStripper sync vs async results"
        );

        // Verify processing actually worked (should remove SDH elements)
        assert!(sync_changed, "Should have detected changes");
        assert!(
            sync_result.len() < 5,
            "Should have removed some SDH elements"
        );

        // Verify specific SDH processing
        for subtitle in sync_result.iter() {
            assert!(
                !subtitle.content.contains("[background music playing]"),
                "Should remove description brackets"
            );
            assert!(
                !subtitle.content.contains("NARRATOR:"),
                "Should remove speaker names"
            );
            assert!(
                !subtitle.content.starts_with(">>"),
                "Should remove CC speaker tags"
            );
        }

        println!("✅ SDHStripper processor: sync and async produce identical results");
    }

    #[tokio::test]
    async fn test_comprehensive_sync_async_pipeline() {
        // Test the complete pipeline: conversion + post-processing
        let webvtt_content = "WEBVTT\n\n00:00:01.000 --> 00:00:03.000\nHello,  world   with  extra  spaces\n\n00:00:04.000 --> 00:00:06.000\n[background music playing]\n\n00:00:07.000 --> 00:00:09.000\nNARRATOR: Final  line  with  £ notes\n";

        // Conversion step
        let converter = WebVTTConverter::new();
        let sync_srt = converter.from_string(webvtt_content).unwrap();
        let async_srt = converter.from_string_async(webvtt_content).await.unwrap();

        // Post-processing step 1: CommonIssuesFixer
        let fixer = CommonIssuesFixer::new();
        let (sync_fixed, _) = fixer.process(sync_srt, None).unwrap();
        let (async_fixed, _) = fixer.process_async(async_srt, None).await.unwrap();

        // Post-processing step 2: SDHStripper
        let stripper = SDHStripper::new();
        let (sync_final, _) = stripper.process(sync_fixed, None).unwrap();
        let (async_final, _) = stripper.process_async(async_fixed, None).await.unwrap();

        // Final results should be identical
        assert_eq!(
            sync_final.export(None),
            async_final.export(None),
            "Complete pipeline sync vs async"
        );

        // Verify the pipeline worked correctly
        assert!(sync_final.len() < 3, "Should have removed SDH elements");
        for subtitle in sync_final.iter() {
            assert!(
                !subtitle.content.contains("  "),
                "Should fix multiple spaces"
            );
            assert!(
                !subtitle.content.contains("["),
                "Should remove SDH descriptions"
            );
            assert!(
                !subtitle.content.contains("NARRATOR:"),
                "Should remove speaker names"
            );
        }

        println!(
            "✅ Complete conversion + processing pipeline: sync and async produce identical results"
        );
    }
}
