use super::super::create_test_srt_with_issues;
use captionrs::{BaseConverter, BaseProcessor, CommonIssuesFixer, SDHStripper, WebVTTConverter};

#[test]
fn test_complete_sync_pipeline() {
    let webvtt_content = "WEBVTT\n\n00:00:01.000 --> 00:00:03.000\nHello,  world   with  extra  spaces\n\n00:00:04.000 --> 00:00:06.000\n[background music playing]\n\n00:00:07.000 --> 00:00:09.000\nNARRATOR: Final  line  with  text\n";

    let converter = WebVTTConverter::new();
    let srt = converter.from_string(webvtt_content).unwrap();

    let fixer = CommonIssuesFixer::new();
    let (fixed_srt, _) = fixer.process(srt, None).unwrap();

    let stripper = SDHStripper::new();
    let (final_srt, _) = stripper.process(fixed_srt, None).unwrap();

    assert!(final_srt.len() < 3, "Should have removed SDH elements");
    for subtitle in final_srt.iter() {
        assert!(
            !subtitle.content.contains("  "),
            "Should fix multiple spaces"
        );
        assert!(
            !subtitle.content.contains('['),
            "Should remove SDH descriptions"
        );
        assert!(
            !subtitle.content.contains("NARRATOR:"),
            "Should remove speaker names"
        );
    }
}

#[test]
fn test_processor_chain_from_string() {
    let srt_content = create_test_srt_with_issues();

    let fixer = CommonIssuesFixer::new();
    let (fixed_srt, _) = fixer.from_string(&srt_content, None).unwrap();

    let stripper = SDHStripper::new();
    let (final_srt, _) = stripper.from_srt(fixed_srt, None).unwrap();

    assert!(!final_srt.is_empty());
    for subtitle in final_srt.iter() {
        assert!(
            !subtitle.content.contains("  "),
            "Should fix spacing issues"
        );
    }
}

#[cfg(feature = "async")]
mod async_end_to_end_tests {
    use super::*;
    use captionrs::{AsyncBaseConverter, AsyncBaseProcessor};

    #[tokio::test]
    async fn test_complete_async_pipeline() {
        let webvtt_content = "WEBVTT\n\n00:00:01.000 --> 00:00:03.000\nHello,  world   with  extra  spaces\n\n00:00:04.000 --> 00:00:06.000\n[background music playing]\n\n00:00:07.000 --> 00:00:09.000\nNARRATOR: Final  line  with  text\n";

        let converter = WebVTTConverter::new();
        let srt = converter.from_string_async(webvtt_content).await.unwrap();

        let fixer = CommonIssuesFixer::new();
        let (fixed_srt, _) = fixer.process_async(srt, None).await.unwrap();

        let stripper = SDHStripper::new();
        let (final_srt, _) = stripper.process_async(fixed_srt, None).await.unwrap();

        assert!(final_srt.len() < 3, "Should have removed SDH elements");
        for subtitle in final_srt.iter() {
            assert!(
                !subtitle.content.contains("  "),
                "Should fix multiple spaces"
            );
            assert!(
                !subtitle.content.contains('['),
                "Should remove SDH descriptions"
            );
            assert!(
                !subtitle.content.contains("NARRATOR:"),
                "Should remove speaker names"
            );
        }
    }
}
