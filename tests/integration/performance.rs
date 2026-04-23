#[cfg(feature = "async")]
mod performance_tests {
    use super::super::create_large_webvtt_sample;
    use captionrs::{AsyncBaseConverter, SAMIConverter, SMPTEConverter, WebVTTConverter};
    use std::time::Instant;
    use tokio::task;

    #[tokio::test]
    async fn test_async_performance() {
        // Create a larger WebVTT sample for performance testing
        let large_webvtt = create_large_webvtt_sample(100);

        let converter = WebVTTConverter::new();
        let start = Instant::now();

        let result = converter.from_string_async(&large_webvtt).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        let srt = result.unwrap();
        assert_eq!(srt.len(), 100);

        println!("Async processing of 100 subtitles took: {:?}", duration);
        // Performance should be reasonable (less than 100ms for 100 subtitles)
        assert!(duration.as_millis() < 1000);
    }

    #[tokio::test]
    async fn test_concurrent_processing() {
        use super::super::{SMPTE_SAMPLE, WEBVTT_SAMPLE};

        // Test concurrent processing of multiple subtitle files
        let tasks = vec![
            task::spawn(async {
                let converter = WebVTTConverter::new();
                converter.from_string_async(WEBVTT_SAMPLE).await.unwrap()
            }),
            task::spawn(async {
                let converter = SMPTEConverter::new();
                converter.from_string_async(SMPTE_SAMPLE).await.unwrap()
            }),
        ];

        let results = futures::future::try_join_all(tasks).await.unwrap();

        assert_eq!(results.len(), 2);
        for srt in results {
            assert_eq!(srt.len(), 3);
        }
    }

    #[tokio::test]
    async fn test_concurrent_converter_processing() {
        use super::super::{SMPTE_SAMPLE, WEBVTT_SAMPLE};

        // Test concurrent processing of different converter types
        let tasks = vec![
            task::spawn(async move {
                let converter = WebVTTConverter::new();
                converter.from_string_async(WEBVTT_SAMPLE).await.unwrap()
            }),
            task::spawn(async move {
                let converter = SMPTEConverter::new();
                converter.from_string_async(SMPTE_SAMPLE).await.unwrap()
            }),
            task::spawn(async move {
                let converter = SAMIConverter::new();
                let sami_sample = r#"<SAMI><BODY>
<SYNC Start="1000"><P>Hello from SAMI!</P>
<SYNC Start="4000"><P>Another SAMI line.</P>
</BODY></SAMI>"#;
                converter.from_string_async(sami_sample).await.unwrap()
            }),
        ];

        let results = futures::future::try_join_all(tasks).await.unwrap();

        assert_eq!(results.len(), 3);
        // WebVTT and SMPTE should have 3 subtitles each, SAMI should have at least 1
        assert_eq!(results[0].len(), 3); // WebVTT
        assert_eq!(results[1].len(), 3); // SMPTE
        assert!(!results[2].is_empty()); // SAMI
    }
}
