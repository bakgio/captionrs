#[cfg(feature = "async")]
mod async_processor_tests {
    use super::super::{
        ADDING_LINE_BREAKS_EXAMPLE, DUPE_ALIGNMENT_EXAMPLE, DUPLICATE_EXAMPLE,
        ELIPSES_FIXING_EXAMPLE, GAP_REMOVAL_EXAMPLE, INVALID_TIMESTAMP_EXAMPLE, ITERATION_EXAMPLE,
        MUSICAL_NOTE_EXAMPLE, OVERLAPPING_TIME_EXAMPLE, SDH_BLEEP_TEST, SDH_COMPREHENSIVE_TEST,
        SDH_EDGE_CASE_TEST, SDH_EMPTY_CONTENT_TEST, SDH_EXAMPLE, SDH_EXTRA_REGEX_TEST,
        SPACE_REMOVAL_EXAMPLE, SPACES_AFTER_HYPHENS_EXAMPLE, TAG_CORRECTIONS_EXAMPLE,
        misdecode_utf8_as_windows_1252,
    };
    use captionrs::{AsyncBaseProcessor, CommonIssuesFixer, SDHStripper, SubRipFile};
    use chrono::TimeDelta;

    #[tokio::test]
    async fn test_common_issues_fixer_async() {
        let mut srt = SubRipFile::new(None);
        srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::milliseconds(1000),
            TimeDelta::milliseconds(3000),
            "Hello,  world!".to_string(),
        ));
        srt.push(captionrs::subripfile::Subtitle::new(
            2,
            TimeDelta::milliseconds(4000),
            TimeDelta::milliseconds(6000),
            "This   has   extra   spaces.".to_string(),
        ));

        let fixer = CommonIssuesFixer::new();
        let result = fixer.process_async(srt, None).await;
        assert!(result.is_ok());

        let (processed_srt, changed) = result.unwrap();
        assert!(changed);
        assert_eq!(processed_srt.len(), 2);

        let first = processed_srt.get(0).unwrap();
        assert_eq!(first.content, "Hello, world!");

        let second = processed_srt.get(1).unwrap();
        assert_eq!(second.content, "This has extra spaces.");
    }

    #[tokio::test]
    async fn test_sdh_stripper_async() {
        let mut srt = SubRipFile::new(None);
        srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::milliseconds(1000),
            TimeDelta::milliseconds(3000),
            "Hello, world!".to_string(),
        ));
        srt.push(captionrs::subripfile::Subtitle::new(
            2,
            TimeDelta::milliseconds(4000),
            TimeDelta::milliseconds(6000),
            "(music playing)".to_string(),
        ));
        srt.push(captionrs::subripfile::Subtitle::new(
            3,
            TimeDelta::milliseconds(7000),
            TimeDelta::milliseconds(9000),
            "This is dialogue.".to_string(),
        ));

        let stripper = SDHStripper::new();
        let result = stripper.process_async(srt, None).await;
        assert!(result.is_ok());

        let (processed_srt, changed) = result.unwrap();
        assert!(changed);
        assert_eq!(processed_srt.len(), 2); // Should remove the music description

        let first = processed_srt.get(0).unwrap();
        assert_eq!(first.content, "Hello, world!");

        let second = processed_srt.get(1).unwrap();
        assert_eq!(second.content, "This is dialogue.");
    }

    #[tokio::test]
    async fn test_processor_from_string_async() {
        let input_srt = r#"1
00:00:01,000 --> 00:00:03,000
Hello,  world!

2
00:00:04,000 --> 00:00:06,000
(music playing)

3
00:00:07,000 --> 00:00:09,000
This  has  spaces."#;

        let fixer = CommonIssuesFixer::new();
        let result = fixer.from_string_async(input_srt, None).await;
        assert!(result.is_ok());

        let (processed_srt, changed) = result.unwrap();
        assert!(changed);
        assert_eq!(processed_srt.len(), 3);

        let first = processed_srt.get(0).unwrap();
        assert_eq!(first.content, "Hello, world!");

        let third = processed_srt.get(2).unwrap();
        assert_eq!(third.content, "This has spaces.");
    }

    #[tokio::test]
    async fn test_processor_from_file_async() {
        use tempfile::NamedTempFile;

        let input_srt = r#"1
00:00:01,000 --> 00:00:03,000
Hello,  world!

2
00:00:04,000 --> 00:00:06,000
(music playing)"#;

        // Create a temporary SRT file
        let temp_file = NamedTempFile::new().unwrap();
        tokio::fs::write(temp_file.path(), input_srt).await.unwrap();

        let fixer = CommonIssuesFixer::new();
        let result = fixer.from_file_async(temp_file.path(), None).await;
        assert!(result.is_ok());

        let (processed_srt, changed) = result.unwrap();
        assert!(changed);
        assert_eq!(processed_srt.len(), 2);
    }

    #[tokio::test]
    async fn test_all_async_processor_interfaces() {
        // Verify all processors implement the async trait correctly
        let empty_srt = SubRipFile::new(None);

        // CommonIssuesFixer
        let result = CommonIssuesFixer::new()
            .process_async(empty_srt.clone(), None)
            .await;
        assert!(
            result.is_ok(),
            "CommonIssuesFixer should handle empty input"
        );
        let (processed_srt, _changed) = result.unwrap();
        assert_eq!(
            processed_srt.len(),
            0,
            "Empty input should produce empty output"
        );

        // SDHStripper
        let result = SDHStripper::new()
            .process_async(empty_srt.clone(), None)
            .await;
        assert!(result.is_ok(), "SDHStripper should handle empty input");
        let (processed_srt, _changed) = result.unwrap();
        assert_eq!(
            processed_srt.len(),
            0,
            "Empty input should produce empty output"
        );
    }

    #[tokio::test]
    async fn test_musical_notes_async() {
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(MUSICAL_NOTE_EXAMPLE, None)
            .await
            .unwrap();

        // Test correct musical note conversion (same as sync test)
        assert_eq!(srt[0].content, "#TestData");
        assert_eq!(srt[1].content, "#TestData#");
        assert_eq!(srt[2].content, "♪ #TestData ♪");
        assert_eq!(srt[3].content, "♪ Song Lyrics ♪");
        assert_eq!(srt[4].content, "We are #1!");
        assert_eq!(srt[5].content, "<i>♪ Song Lyrics</i>");
        assert_eq!(srt[6].content, "♪ Song Lyrics\nOn two separate lines ♪");
        assert_eq!(srt[7].content, "#1 Radio Station");
        assert_eq!(srt[8].content, "ABCD FM\n#1 Radio Station");
        assert_eq!(srt[9].content, "#One Radio Station");
        assert_eq!(srt[10].content, "<i>♪ Fire ♪</i>");
        assert_eq!(srt[11].content, "*Schnaub*");
        assert_eq!(srt[12].content, "♪ Schnaub ♪");
        assert_eq!(srt[13].content, "♪ Thunder");
    }

    #[tokio::test]
    async fn test_adding_line_breaks_async() {
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(ADDING_LINE_BREAKS_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(srt[0].content, "- It's chocolate.\n- Hmm?");
        assert_eq!(
            srt[1].content,
            "- We can't just leave him.\n- He's already gone."
        );
        assert_eq!(srt[2].content, "- Test. Mr.Teufel...\n- Test...");
    }

    #[tokio::test]
    async fn test_elipses_fixing_async() {
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(ELIPSES_FIXING_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(srt[0].content, "...noooooooooooooo...");
        assert_eq!(srt[1].content, "<i>Stop this...</i>");
    }

    #[tokio::test]
    async fn test_tag_corrections_async() {
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(TAG_CORRECTIONS_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(srt[0].content, "<i>Test line1\nTest</i> line2");
        assert_eq!(srt[1].content, "{\\an8}<i>Test line1\nTest line2</i>");
        assert_eq!(srt[2].content, "test");
        assert_eq!(srt[3].content, "<i>test</i>");
    }

    #[tokio::test]
    async fn test_gap_removal_async() {
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(GAP_REMOVAL_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(srt[0].end, TimeDelta::milliseconds(19 * 60 * 1000 + 182));
        assert_eq!(srt[1].start, TimeDelta::milliseconds(19 * 60 * 1000 + 183));

        let mut fixer2 = CommonIssuesFixer::new();
        fixer2.remove_gaps = false;
        let (srt2, _) = fixer2
            .from_string_async(GAP_REMOVAL_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(srt2[0].end, TimeDelta::milliseconds(19 * 60 * 1000 + 100));
        assert_eq!(srt2[1].start, TimeDelta::milliseconds(19 * 60 * 1000 + 183));
    }

    #[tokio::test]
    async fn test_redundant_space_removal_async() {
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(SPACE_REMOVAL_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(
            srt[0].content,
            "<i>SOMETHING:\nSynthetic test.\nDefinitely not real.</i>"
        );
    }

    #[tokio::test]
    async fn test_adding_spaces_after_frontal_hyphens_async() {
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(SPACES_AFTER_HYPHENS_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(srt[0].content, "- Well.\n- $5000?");
    }

    #[tokio::test]
    async fn test_invalid_timestamp_fixing_async() {
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(INVALID_TIMESTAMP_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(srt[0].start, TimeDelta::milliseconds(27 * 60 * 1000));
        assert_eq!(srt[1].start, TimeDelta::milliseconds((60 + 27) * 60 * 1000));
    }

    #[tokio::test]
    async fn test_fix_overlapping_time_async() {
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(OVERLAPPING_TIME_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(srt[0].end, TimeDelta::milliseconds(104));
        assert_eq!(srt[1].start, TimeDelta::milliseconds(105));

        let mut fixer2 = CommonIssuesFixer::new();
        fixer2.remove_gaps = false;
        let (srt2, _) = fixer2
            .from_string_async(OVERLAPPING_TIME_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(srt2[0].end, TimeDelta::milliseconds(104));
        assert_eq!(srt2[1].start, TimeDelta::milliseconds(105));
    }

    #[tokio::test]
    async fn test_dupe_alignment_tags_async() {
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(DUPE_ALIGNMENT_EXAMPLE, None)
            .await
            .unwrap();

        assert_eq!(srt.len(), 1);
        assert_eq!(
            srt[0].content,
            "{\\an8}I'm only nineteen\nbut my mind is old"
        );
    }

    #[tokio::test]
    async fn test_duplicate_removal_async() {
        // Test duplicate detection using duration() method
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(DUPLICATE_EXAMPLE, None)
            .await
            .unwrap();

        // Should remove duplicate with same start, end, and duration
        assert_eq!(srt.len(), 2);
        assert_eq!(srt[0].content, "First subtitle");
        assert_eq!(srt[1].content, "Second subtitle");

        // Verify duration method is working
        assert_eq!(srt[0].duration(), TimeDelta::seconds(2));
        assert_eq!(srt[1].duration(), TimeDelta::seconds(1));
    }

    #[tokio::test]
    async fn test_subtitle_iteration_async() {
        // Test the iter() method by iterating over subtitles
        let fixer = CommonIssuesFixer::new();
        let (srt, _) = fixer
            .from_string_async(ITERATION_EXAMPLE, None)
            .await
            .unwrap();

        // Use iter() method to iterate through subtitles
        let mut count = 0;
        let mut total_duration = TimeDelta::zero();

        for subtitle in srt.iter() {
            count += 1;
            total_duration += subtitle.duration();
        }

        assert_eq!(count, 3);
        assert_eq!(total_duration, TimeDelta::seconds(3)); // Each subtitle is 1 second
    }

    #[tokio::test]
    async fn test_sdh_stripping_detailed_async() {
        let stripper = SDHStripper::new();
        let fixer = CommonIssuesFixer::new();
        let (stripped_srt, _) = stripper.from_string_async(SDH_EXAMPLE, None).await.unwrap();
        let (srt, _) = fixer.from_srt_async(stripped_srt, None).await.unwrap();

        assert_eq!(srt.len(), 8);
        assert_eq!(srt[0].content, r#"<i>"W" who?</i>"#);
        assert_eq!(srt[1].content, "- ♪ Hey, boo ♪\n- ♪ Hey, boo ♪");
        assert_eq!(srt[2].content, "It's zoo time!");
        assert_eq!(srt[3].content, r#"{\an8}Spooky!"#);
        assert_eq!(srt[4].content, "Hmm?\n<i>Hello!</i>");
        assert_eq!(srt[5].content, "I did on magnets this summer.");
        assert_eq!(srt[6].content, "- Boo!\n- No, thanks.");
        assert_eq!(
            srt[7].content,
            "SO THIS IS MY HOME OFFICE\nHERE. IN THIS OFFICE ARE A LOT"
        );
    }

    #[tokio::test]
    async fn test_sdh_stripping_removes_standalone_bleeps_but_keeps_dialogue_async() {
        let stripper = SDHStripper::new();
        let (srt, _) = stripper
            .from_string_async(SDH_BLEEP_TEST, None)
            .await
            .unwrap();

        assert_eq!(srt.len(), 1);
        assert_eq!(srt[0].content, "Move!");
    }

    #[tokio::test]
    async fn test_sdh_stripping_keeps_timestamp_like_prefixes_async() {
        let stripper = SDHStripper::new();
        let input = r#"1
00:00:01,000 --> 00:00:02,000
CHAPTER:00:12:34

2
00:00:03,000 --> 00:00:04,000
NARRATOR: Actual dialogue"#;

        let (srt, _) = stripper.from_string_async(input, None).await.unwrap();

        assert_eq!(srt.len(), 2);
        assert_eq!(srt[0].content, "CHAPTER:00:12:34");
        assert_eq!(srt[1].content, "Actual dialogue");
    }

    #[tokio::test]
    async fn test_sdh_stripping_removes_multi_word_inline_descriptions_async() {
        let stripper = SDHStripper::new();
        let fixer = CommonIssuesFixer::new();
        let input = r#"1
00:00:01,000 --> 00:00:02,000
We [crowd cheering loudly] won.

2
00:00:03,000 --> 00:00:04,000
(softly whispering) Move now.

3
00:00:05,000 --> 00:00:06,000
[bleep] happens."#;

        let (stripped, _) = stripper.from_string_async(input, None).await.unwrap();
        let (srt, _) = fixer.from_srt_async(stripped, None).await.unwrap();

        assert_eq!(srt.len(), 3);
        assert_eq!(srt[0].content, "We won.");
        assert_eq!(srt[1].content, "Move now.");
        assert_eq!(srt[2].content, "[bleep] happens.");
    }

    #[tokio::test]
    async fn test_sdh_stripping_with_extra_regexes_async() {
        // Test with extra regexes constructor parameter
        let stripper = SDHStripper::with_extra_regexes(vec![r"\bTEST\b"]).unwrap();

        let test_srt = SDH_EXTRA_REGEX_TEST;

        let (stripped_srt, _) = stripper.from_string_async(test_srt, None).await.unwrap();

        // The extra regex should remove "TEST"
        assert_eq!(stripped_srt.len(), 2);
        assert_eq!(stripped_srt[0].content, "This is a  line.");
        assert_eq!(stripped_srt[1].content, "Normal line.");
    }

    #[tokio::test]
    async fn test_extra_regexes_comprehensive_async() {
        // Test with multiple regexes
        let regexes = vec![
            r"\bAD\b",    // Remove "AD" (advertisement)
            r"\[MUSIC\]", // Remove [MUSIC] tags
            r"www\.\S+",  // Remove website URLs
        ];

        let stripper = SDHStripper::with_extra_regexes(regexes).unwrap();

        let test_srt = SDH_COMPREHENSIVE_TEST;

        let (result, _) = stripper.from_string_async(test_srt, None).await.unwrap();

        assert_eq!(result.len(), 3);
        // First subtitle should have AD, URL, and [MUSIC] removed
        assert_eq!(result[0].content, "This is an  for ");
        // Second subtitle unchanged
        assert_eq!(result[1].content, "Normal subtitle content");
        // Third subtitle should have AD and [MUSIC] removed
        assert_eq!(result[2].content, "Another  with  playing");
    }

    #[tokio::test]
    async fn test_extra_regexes_error_cases_async() {
        // Test with invalid regex pattern
        let invalid_regexes = vec![r"[unclosed_bracket"];
        let result = SDHStripper::with_extra_regexes(invalid_regexes);

        assert!(result.is_err());

        // Test with valid and invalid mixed
        let mixed_regexes = vec![r"\btest\b", r"[invalid"];
        let result2 = SDHStripper::with_extra_regexes(mixed_regexes);

        assert!(result2.is_err());

        // Test with empty regexes (should work)
        let empty_regexes: Vec<&str> = vec![];
        let result3 = SDHStripper::with_extra_regexes(empty_regexes);

        assert!(result3.is_ok());
    }

    #[tokio::test]
    async fn test_extra_regexes_edge_cases_async() {
        // Test with complex regex patterns
        let complex_regexes = vec![
            r"(?i)\b(advertisement|promo)\b", // Case insensitive
            r"\d{1,2}:\d{2}",                 // Time patterns
            r"\s+",                           // Multiple whitespace (normalize to single space)
        ];

        let stripper = SDHStripper::with_extra_regexes(complex_regexes).unwrap();

        let test_srt = SDH_EDGE_CASE_TEST;

        let (result, _) = stripper.from_string_async(test_srt, None).await.unwrap();

        assert_eq!(result.len(), 2);
        // Should remove "Advertisement", "12:34" and normalize whitespace
        assert_eq!(result[0].content, "Thisisanat");
        // Should remove "PROMO" and normalize whitespace
        assert_eq!(result[1].content, "Normalsubtitlewith");
    }

    #[tokio::test]
    async fn test_extra_regexes_empty_content_async() {
        // Test regex that removes entire content
        let aggressive_regex = vec![r".*"]; // Remove everything
        let stripper = SDHStripper::with_extra_regexes(aggressive_regex).unwrap();

        let test_srt = SDH_EMPTY_CONTENT_TEST;

        let (result, _) = stripper.from_string_async(test_srt, None).await.unwrap();

        // Should filter out subtitles with empty content after regex processing
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_async_processor_chain() {
        // Create a test SRT with issues that both processors can fix
        let mut srt = SubRipFile::new(None);
        srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::milliseconds(1000),
            TimeDelta::milliseconds(3000),
            "Hello,  world!".to_string(), // Extra spaces for CommonIssuesFixer
        ));
        srt.push(captionrs::subripfile::Subtitle::new(
            2,
            TimeDelta::milliseconds(4000),
            TimeDelta::milliseconds(6000),
            "(music playing)".to_string(), // SDH description for SDHStripper
        ));
        srt.push(captionrs::subripfile::Subtitle::new(
            3,
            TimeDelta::milliseconds(7000),
            TimeDelta::milliseconds(9000),
            "This  has  spaces   and  (sound effects).".to_string(), // Both issues
        ));

        // Chain async processors
        let common_fixer = CommonIssuesFixer::new();
        let sdh_stripper = SDHStripper::new();

        // First fix common issues
        let (intermediate_srt, _) = common_fixer.process_async(srt, None).await.unwrap();

        // Then strip SDH
        let (final_srt, _) = sdh_stripper
            .process_async(intermediate_srt, None)
            .await
            .unwrap();

        // Should have fewer subtitles due to SDH removal
        assert!(final_srt.len() < 3);

        // Remaining subtitles should have fixed spacing
        for subtitle in final_srt.iter() {
            assert!(!subtitle.content.contains("  ")); // No double spaces
            assert!(!subtitle.content.contains("(music playing)")); // No SDH descriptions
        }
    }

    #[tokio::test]
    async fn test_unicode_normalization_is_english_only_async() {
        let mut srt = SubRipFile::new(None);
        srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::seconds(1),
            TimeDelta::seconds(2),
            "ＡＢＣ".to_string(),
        ));

        let fixer = CommonIssuesFixer::new();
        let (english, _) = fixer.process_async(srt.clone(), Some("eng")).await.unwrap();
        let (french, _) = fixer.process_async(srt, Some("fr")).await.unwrap();

        assert_eq!(english[0].content, "ABC");
        assert_eq!(french[0].content, "ＡＢＣ");
    }

    #[tokio::test]
    async fn test_unicode_normalization_accepts_language_names_async() {
        let mut srt = SubRipFile::new(None);
        srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::seconds(1),
            TimeDelta::seconds(2),
            "ＡＢＣ".to_string(),
        ));

        let fixer = CommonIssuesFixer::new();
        let (english_name, _) = fixer
            .process_async(srt.clone(), Some("English (US)"))
            .await
            .unwrap();
        let (english_code, _) = fixer.process_async(srt, Some("eng")).await.unwrap();

        assert_eq!(english_name[0].content, "ABC");
        assert_eq!(english_code[0].content, "ABC");
    }

    #[tokio::test]
    async fn test_rtl_markers_are_injected_for_rtl_languages_async() {
        let mut srt = SubRipFile::new(None);
        srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::seconds(1),
            TimeDelta::seconds(2),
            "\u{200f}שלום\nעולם".to_string(),
        ));

        let fixer = CommonIssuesFixer::new();
        let (processed, _) = fixer.process_async(srt, Some("heb")).await.unwrap();

        assert_eq!(processed[0].content, "\u{202b}שלום\n\u{202b}עולם");
    }

    #[tokio::test]
    async fn test_rtl_language_aliases_are_recognized_async() {
        let mut srt = SubRipFile::new(None);
        srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::seconds(1),
            TimeDelta::seconds(2),
            "שלום\nעולם".to_string(),
        ));

        let fixer = CommonIssuesFixer::new();
        let (hebrew_name, _) = fixer
            .process_async(srt.clone(), Some("Hebrew"))
            .await
            .unwrap();
        let (hebrew_alias, _) = fixer.process_async(srt, Some("iw")).await.unwrap();
        let expected = "\u{202b}שלום\n\u{202b}עולם";

        assert_eq!(hebrew_name[0].content, expected);
        assert_eq!(hebrew_alias[0].content, expected);
    }

    #[tokio::test]
    async fn test_rtl_processing_repairs_mojibake_before_inserting_markers_async() {
        let mut srt = SubRipFile::new(None);
        srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::seconds(1),
            TimeDelta::seconds(2),
            "\u{200f}\u{00d7}\u{00a9}\u{00d7}\u{0153}\u{00d7}\u{2022}\u{00d7}\u{009d}\n\u{00d7}\u{00a2}\u{00d7}\u{2022}\u{00d7}\u{0153}\u{00d7}\u{009d}".to_string(),
        ));

        let fixer = CommonIssuesFixer::new();
        let (processed, _) = fixer.process_async(srt, Some("heb")).await.unwrap();

        assert_eq!(processed[0].content, "\u{202b}שלום\n\u{202b}עולם");
    }

    #[tokio::test]
    async fn test_mojibake_repair_handles_double_encoded_text_async() {
        let expected = "prot\u{00E9}g\u{00E9} \u{266A}".to_string();
        let corrupted = misdecode_utf8_as_windows_1252(&misdecode_utf8_as_windows_1252(&expected));
        let mut srt = SubRipFile::new(None);
        srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::seconds(1),
            TimeDelta::seconds(2),
            corrupted,
        ));

        let fixer = CommonIssuesFixer::new();
        let (processed, _) = fixer.process_async(srt, Some("English")).await.unwrap();

        assert_eq!(processed[0].content, expected);
    }

    #[tokio::test]
    async fn test_mojibake_repair_handles_triple_encoded_fullwidth_english_text_async() {
        let mut srt = SubRipFile::new(None);
        srt.push(captionrs::subripfile::Subtitle::new(
            1,
            TimeDelta::seconds(1),
            TimeDelta::seconds(2),
            "ÃƒÂ¯Ã‚Â¼Ã‚Â¡ÃƒÂ¯Ã‚Â¼Ã‚Â¢ÃƒÂ¯Ã‚Â¼Ã‚Â£".to_string(),
        ));

        let fixer = CommonIssuesFixer::new();
        let (processed, _) = fixer.process_async(srt, Some("English")).await.unwrap();

        assert_eq!(processed[0].content, "ABC");
    }
}
