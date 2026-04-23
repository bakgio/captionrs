use super::{
    ADDING_LINE_BREAKS_EXAMPLE, DUPE_ALIGNMENT_EXAMPLE, DUPLICATE_EXAMPLE, ELIPSES_FIXING_EXAMPLE,
    GAP_REMOVAL_EXAMPLE, INVALID_TIMESTAMP_EXAMPLE, ITERATION_EXAMPLE, MUSICAL_NOTE_EXAMPLE,
    OVERLAPPING_TIME_EXAMPLE, SDH_BLEEP_TEST, SDH_COMPREHENSIVE_TEST, SDH_EDGE_CASE_TEST,
    SDH_EMPTY_CONTENT_TEST, SDH_EXAMPLE, SDH_EXTRA_REGEX_TEST,
    SINGLE_CHARACTER_LITERAL_PATTERN_EXAMPLE, SPACE_REMOVAL_EXAMPLE, SPACES_AFTER_HYPHENS_EXAMPLE,
    TAG_CORRECTIONS_EXAMPLE, misdecode_utf8_as_windows_1252,
};
use captionrs::SubRipFile;
use captionrs::processors::{BaseProcessor, CommonIssuesFixer, SDHStripper};
use chrono::TimeDelta;

#[test]
fn test_musical_notes() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(MUSICAL_NOTE_EXAMPLE, None).unwrap();

    // Test correct musical note conversion
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

#[test]
fn test_adding_line_breaks() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(ADDING_LINE_BREAKS_EXAMPLE, None).unwrap();

    assert_eq!(srt[0].content, "- It's chocolate.\n- Hmm?");
    assert_eq!(
        srt[1].content,
        "- We can't just leave him.\n- He's already gone."
    );
    assert_eq!(srt[2].content, "- Test. Mr.Teufel...\n- Test...");
}

#[test]
fn test_elipses_fixing() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(ELIPSES_FIXING_EXAMPLE, None).unwrap();

    assert_eq!(srt[0].content, "...noooooooooooooo...");
    assert_eq!(srt[1].content, "<i>Stop this...</i>");
}

#[test]
fn test_tag_corrections() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(TAG_CORRECTIONS_EXAMPLE, None).unwrap();

    assert_eq!(srt[0].content, "<i>Test line1\nTest</i> line2");
    assert_eq!(srt[1].content, "{\\an8}<i>Test line1\nTest line2</i>");
    assert_eq!(srt[2].content, "test");
    assert_eq!(srt[3].content, "<i>test</i>");
}

#[test]
fn test_gap_removal() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(GAP_REMOVAL_EXAMPLE, None).unwrap();

    assert_eq!(srt[0].end, TimeDelta::milliseconds(19 * 60 * 1000 + 182));
    assert_eq!(srt[1].start, TimeDelta::milliseconds(19 * 60 * 1000 + 183));

    let mut fixer2 = CommonIssuesFixer::new();
    fixer2.remove_gaps = false;
    let (srt2, _) = fixer2.from_string(GAP_REMOVAL_EXAMPLE, None).unwrap();

    assert_eq!(srt2[0].end, TimeDelta::milliseconds(19 * 60 * 1000 + 100));
    assert_eq!(srt2[1].start, TimeDelta::milliseconds(19 * 60 * 1000 + 183));
}

#[test]
fn test_redundant_space_removal() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(SPACE_REMOVAL_EXAMPLE, None).unwrap();

    assert_eq!(
        srt[0].content,
        "<i>SOMETHING:\nSynthetic test.\nDefinitely not real.</i>"
    );
}

#[test]
fn test_adding_spaces_after_frontal_hyphens() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer
        .from_string(SPACES_AFTER_HYPHENS_EXAMPLE, None)
        .unwrap();

    assert_eq!(srt[0].content, "- Well.\n- $5000?");
}

#[test]
fn test_invalid_timestamp_fixing() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(INVALID_TIMESTAMP_EXAMPLE, None).unwrap();

    assert_eq!(srt[0].start, TimeDelta::milliseconds(27 * 60 * 1000));
    assert_eq!(srt[1].start, TimeDelta::milliseconds((60 + 27) * 60 * 1000));
}

#[test]
fn test_fix_overlapping_time() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(OVERLAPPING_TIME_EXAMPLE, None).unwrap();

    assert_eq!(srt[0].end, TimeDelta::milliseconds(104));
    assert_eq!(srt[1].start, TimeDelta::milliseconds(105));

    let mut fixer2 = CommonIssuesFixer::new();
    fixer2.remove_gaps = false;
    let (srt2, _) = fixer2.from_string(OVERLAPPING_TIME_EXAMPLE, None).unwrap();

    assert_eq!(srt2[0].end, TimeDelta::milliseconds(104));
    assert_eq!(srt2[1].start, TimeDelta::milliseconds(105));
}

#[test]
fn test_dupe_alignment_tags() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(DUPE_ALIGNMENT_EXAMPLE, None).unwrap();

    assert_eq!(srt.len(), 1);
    assert_eq!(
        srt[0].content,
        "{\\an8}I'm only nineteen\nbut my mind is old"
    );
}

#[test]
fn test_duplicate_removal() {
    // Test duplicate detection using duration() method
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(DUPLICATE_EXAMPLE, None).unwrap();

    // Should remove duplicate with same start, end, and duration
    assert_eq!(srt.len(), 2);
    assert_eq!(srt[0].content, "First subtitle");
    assert_eq!(srt[1].content, "Second subtitle");

    // Verify duration method is working
    assert_eq!(srt[0].duration(), TimeDelta::seconds(2));
    assert_eq!(srt[1].duration(), TimeDelta::seconds(1));
}

#[test]
fn test_subtitle_iteration() {
    // Test the iter() method by iterating over subtitles
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer.from_string(ITERATION_EXAMPLE, None).unwrap();

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

#[test]
fn test_sdh_stripping() {
    let stripper = SDHStripper::new();
    let fixer = CommonIssuesFixer::new();
    let (stripped_srt, _) = stripper.from_string(SDH_EXAMPLE, None).unwrap();
    let (srt, _) = fixer.from_srt(stripped_srt, None).unwrap();

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

#[test]
fn test_sdh_stripping_removes_standalone_bleeps_but_keeps_dialogue() {
    let stripper = SDHStripper::new();
    let (srt, _) = stripper.from_string(SDH_BLEEP_TEST, None).unwrap();

    assert_eq!(srt.len(), 1);
    assert_eq!(srt[0].content, "Move!");
}

#[test]
fn test_sdh_stripping_keeps_timestamp_like_prefixes() {
    let stripper = SDHStripper::new();
    let input = r#"1
00:00:01,000 --> 00:00:02,000
CHAPTER:00:12:34

2
00:00:03,000 --> 00:00:04,000
NARRATOR: Actual dialogue"#;

    let (srt, _) = stripper.from_string(input, None).unwrap();

    assert_eq!(srt.len(), 2);
    assert_eq!(srt[0].content, "CHAPTER:00:12:34");
    assert_eq!(srt[1].content, "Actual dialogue");
}

#[test]
fn test_sdh_stripping_removes_multi_word_inline_descriptions() {
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

    let (stripped, _) = stripper.from_string(input, None).unwrap();
    let (srt, _) = fixer.from_srt(stripped, None).unwrap();

    assert_eq!(srt.len(), 3);
    assert_eq!(srt[0].content, "We won.");
    assert_eq!(srt[1].content, "Move now.");
    assert_eq!(srt[2].content, "[bleep] happens.");
}

#[test]
fn test_sdh_stripping_without_follow_up_cleanup_keeps_current_spacing() {
    let stripper = SDHStripper::new();
    let (srt, _) = stripper.from_string(SDH_EXAMPLE, None).unwrap();

    assert_eq!(srt.len(), 8);
    assert_eq!(srt[3].content, r#"{\an8} Spooky!"#);
    assert_eq!(srt[4].content, "Hmm?\n<i> Hello!</i>");
    assert_eq!(srt[6].content, "- Boo!\n-No, thanks.");
}

#[test]
fn test_unicode_normalization_is_english_only() {
    let mut srt = SubRipFile::new(None);
    srt.push(captionrs::subripfile::Subtitle::new(
        1,
        TimeDelta::seconds(1),
        TimeDelta::seconds(2),
        "ＡＢＣ".to_string(),
    ));

    let fixer = CommonIssuesFixer::new();
    let (english, _) = fixer.process(srt.clone(), Some("eng")).unwrap();
    let (french, _) = fixer.process(srt, Some("fr")).unwrap();

    assert_eq!(english[0].content, "ABC");
    assert_eq!(french[0].content, "ＡＢＣ");
}

#[test]
fn test_unicode_normalization_accepts_language_names() {
    let mut srt = SubRipFile::new(None);
    srt.push(captionrs::subripfile::Subtitle::new(
        1,
        TimeDelta::seconds(1),
        TimeDelta::seconds(2),
        "ＡＢＣ".to_string(),
    ));

    let fixer = CommonIssuesFixer::new();
    let (english_name, _) = fixer.process(srt.clone(), Some("English (US)")).unwrap();
    let (english_code, _) = fixer.process(srt, Some("eng")).unwrap();

    assert_eq!(english_name[0].content, "ABC");
    assert_eq!(english_code[0].content, "ABC");
}

#[test]
fn test_rtl_markers_are_injected_for_rtl_languages() {
    let mut srt = SubRipFile::new(None);
    srt.push(captionrs::subripfile::Subtitle::new(
        1,
        TimeDelta::seconds(1),
        TimeDelta::seconds(2),
        "\u{200f}שלום\nעולם".to_string(),
    ));

    let fixer = CommonIssuesFixer::new();
    let (processed, _) = fixer.process(srt, Some("heb")).unwrap();

    assert_eq!(processed[0].content, "\u{202b}שלום\n\u{202b}עולם");
}

#[test]
fn test_rtl_language_aliases_are_recognized() {
    let mut srt = SubRipFile::new(None);
    srt.push(captionrs::subripfile::Subtitle::new(
        1,
        TimeDelta::seconds(1),
        TimeDelta::seconds(2),
        "שלום\nעולם".to_string(),
    ));

    let fixer = CommonIssuesFixer::new();
    let (hebrew_name, _) = fixer.process(srt.clone(), Some("Hebrew")).unwrap();
    let (hebrew_alias, _) = fixer.process(srt, Some("iw")).unwrap();
    let expected = "\u{202b}שלום\n\u{202b}עולם";

    assert_eq!(hebrew_name[0].content, expected);
    assert_eq!(hebrew_alias[0].content, expected);
}

#[test]
fn test_rtl_processing_repairs_mojibake_before_inserting_markers() {
    let mut srt = SubRipFile::new(None);
    srt.push(captionrs::subripfile::Subtitle::new(
        1,
        TimeDelta::seconds(1),
        TimeDelta::seconds(2),
        "\u{200f}\u{00d7}\u{00a9}\u{00d7}\u{0153}\u{00d7}\u{2022}\u{00d7}\u{009d}\n\u{00d7}\u{00a2}\u{00d7}\u{2022}\u{00d7}\u{0153}\u{00d7}\u{009d}".to_string(),
    ));

    let fixer = CommonIssuesFixer::new();
    let (processed, _) = fixer.process(srt, Some("heb")).unwrap();

    assert_eq!(processed[0].content, "\u{202b}שלום\n\u{202b}עולם");
}

#[test]
fn test_mojibake_repair_handles_double_encoded_text() {
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
    let (processed, _) = fixer.process(srt, Some("English")).unwrap();

    assert_eq!(processed[0].content, expected);
}

#[test]
fn test_mojibake_repair_handles_triple_encoded_fullwidth_english_text() {
    let mut srt = SubRipFile::new(None);
    srt.push(captionrs::subripfile::Subtitle::new(
        1,
        TimeDelta::seconds(1),
        TimeDelta::seconds(2),
        "ÃƒÂ¯Ã‚Â¼Ã‚Â¡ÃƒÂ¯Ã‚Â¼Ã‚Â¢ÃƒÂ¯Ã‚Â¼Ã‚Â£".to_string(),
    ));

    let fixer = CommonIssuesFixer::new();
    let (processed, _) = fixer.process(srt, Some("English")).unwrap();

    assert_eq!(processed[0].content, "ABC");
}

#[test]
fn test_single_character_cleanup_keeps_plain_alphanumeric_lines() {
    let fixer = CommonIssuesFixer::new();
    let (srt, _) = fixer
        .from_string(SINGLE_CHARACTER_LITERAL_PATTERN_EXAMPLE, None)
        .unwrap();

    assert_eq!(srt.len(), 2);
    assert_eq!(srt[0].content, "A");
    assert_eq!(srt[1].content, "B");
}

#[test]
fn test_sdh_stripping_with_extra_regexes() {
    // Test with extra regexes constructor parameter
    let stripper = SDHStripper::with_extra_regexes(vec![r"\bTEST\b"]).unwrap();

    let test_srt = SDH_EXTRA_REGEX_TEST;

    let (stripped_srt, _) = stripper.from_string(test_srt, None).unwrap();

    // The extra regex should remove "TEST"
    assert_eq!(stripped_srt.len(), 2);
    assert_eq!(stripped_srt[0].content, "This is a  line.");
    assert_eq!(stripped_srt[1].content, "Normal line.");
}

#[test]
fn test_extra_regexes_comprehensive() {
    // Test with multiple regexes
    let regexes = vec![
        r"\bAD\b",    // Remove "AD" (advertisement)
        r"\[MUSIC\]", // Remove [MUSIC] tags
        r"www\.\S+",  // Remove website URLs
    ];

    let stripper = SDHStripper::with_extra_regexes(regexes).unwrap();

    let test_srt = SDH_COMPREHENSIVE_TEST;

    let (result, _) = stripper.from_string(test_srt, None).unwrap();

    assert_eq!(result.len(), 3);
    // First subtitle should have AD, URL, and [MUSIC] removed
    assert_eq!(result[0].content, "This is an  for ");
    // Second subtitle unchanged
    assert_eq!(result[1].content, "Normal subtitle content");
    // Third subtitle should have AD and [MUSIC] removed
    assert_eq!(result[2].content, "Another  with  playing");
}

#[test]
fn test_extra_regexes_error_cases() {
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

#[test]
fn test_extra_regexes_edge_cases() {
    // Test with complex regex patterns
    let complex_regexes = vec![
        r"(?i)\b(advertisement|promo)\b", // Case insensitive
        r"\d{1,2}:\d{2}",                 // Time patterns
        r"\s+",                           // Multiple whitespace (normalize to single space)
    ];

    let stripper = SDHStripper::with_extra_regexes(complex_regexes).unwrap();

    let test_srt = SDH_EDGE_CASE_TEST;

    let (result, _) = stripper.from_string(test_srt, None).unwrap();

    assert_eq!(result.len(), 2);
    // Should remove "Advertisement", "12:34" and normalize whitespace
    assert_eq!(result[0].content, "Thisisanat");
    // Should remove "PROMO" and normalize whitespace
    assert_eq!(result[1].content, "Normalsubtitlewith");
}

#[test]
fn test_extra_regexes_empty_content() {
    // Test regex that removes entire content
    let aggressive_regex = vec![r".*"]; // Remove everything
    let stripper = SDHStripper::with_extra_regexes(aggressive_regex).unwrap();

    let test_srt = SDH_EMPTY_CONTENT_TEST;

    let (result, _) = stripper.from_string(test_srt, None).unwrap();

    // Should filter out subtitles with empty content after regex processing
    assert_eq!(result.len(), 0);
}
