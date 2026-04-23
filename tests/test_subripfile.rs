use std::fs;

use captionrs::{SubRipFile, subripfile::Subtitle};
use chrono::TimeDelta;
use encoding_rs::UTF_16LE;
use tempfile::tempdir;

#[test]
fn export_applies_requested_line_endings_inside_multiline_content() {
    let srt =
        SubRipFile::from_string("1\n00:00:01,000 --> 00:00:02,000\nFirst line\nSecond line\n")
            .expect("srt should parse");

    let exported = srt.export(Some("\r\n"));

    assert!(exported.contains("First line\r\nSecond line"));
    assert!(!exported.contains("First line\nSecond line"));
}

#[test]
fn export_sorts_and_reindexes_by_default() {
    let srt = SubRipFile::from_string(
        "3\n00:00:04.000 --> 00:00:05.000\nThird cue.\n\n1\n00:00:01.000 --> 00:00:02.000\nFirst cue.\n\n2\n00:00:02.500 --> 00:00:03.500\nSecond cue.\n",
    )
    .expect("srt should parse");

    let exported = srt.export(None);

    assert_eq!(
        exported,
        "1\n00:00:01,000 --> 00:00:02,000\nFirst cue.\n\n2\n00:00:02,500 --> 00:00:03,500\nSecond cue.\n\n3\n00:00:04,000 --> 00:00:05,000\nThird cue.\n\n"
    );
}

#[test]
fn from_string_attaches_orphan_text_after_timed_cues() {
    let srt = SubRipFile::from_string(
        "orphan text before the first cue\n\n1\n00:00:01,000 --> 00:00:02,000\nFirst cue.\n\norphan text after the first cue\n\n2\n00:00:03,000 --> 00:00:04,000\nSecond cue.\n\ntrailing orphan text\n",
    )
    .expect("srt should parse");

    assert_eq!(srt.len(), 2);
    assert_eq!(
        srt[0].content,
        "First cue.\norphan text after the first cue"
    );
    assert_eq!(srt[1].content, "Second cue.\ntrailing orphan text");
}

#[test]
fn from_string_assigns_the_next_index_when_a_cue_number_is_missing() {
    let srt = SubRipFile::from_string(
        "1\n00:00:01,000 --> 00:00:02,000\nFirst cue.\n\n00:00:03,000 --> 00:00:04,000\nSecond cue.\n",
    )
    .expect("srt should parse");

    assert_eq!(srt.len(), 2);
    assert_eq!(srt[0].index, 1);
    assert_eq!(srt[1].index, 2);
}

#[test]
fn clean_indexes_sorts_and_reindexes_in_place() {
    let mut srt = SubRipFile::from_string(
        "3\n00:00:04,000 --> 00:00:05,000\nThird cue.\n\n1\n00:00:01,000 --> 00:00:02,000\nFirst cue.\n\n2\n00:00:02,500 --> 00:00:03,500\nSecond cue.\n",
    )
    .expect("srt should parse");

    srt.clean_indexes();

    assert_eq!(srt[0].index, 1);
    assert_eq!(srt[0].content, "First cue.");
    assert_eq!(srt[1].index, 2);
    assert_eq!(srt[2].index, 3);
}

#[test]
fn offset_shifts_all_timestamps() {
    let mut srt = SubRipFile::from_string("1\n00:00:01,000 --> 00:00:02,000\nShift me.\n")
        .expect("srt should parse");

    srt.offset(TimeDelta::milliseconds(1500));

    assert_eq!(srt[0].start, TimeDelta::milliseconds(2500));
    assert_eq!(srt[0].end, TimeDelta::milliseconds(3500));
}

#[test]
fn save_respects_encoding_and_line_endings() {
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("encoded_utf16.srt");
    let srt =
        SubRipFile::from_string("1\n00:00:01,000 --> 00:00:02,000\nFirst line\nSecond line\n")
            .expect("srt should parse");

    srt.save(&output_path, Some("utf-16"), Some("\r\n"))
        .expect("srt should save");

    let written = fs::read(&output_path).expect("saved file should be readable");
    let (decoded, had_errors) = UTF_16LE.decode_without_bom_handling(&written[2..]);

    assert!(written.starts_with(&[0xFF, 0xFE]));
    assert!(!had_errors);
    assert!(decoded.contains("First line\r\nSecond line"));
}

#[test]
fn export_skips_zero_duration_cues_while_preserving_valid_entries() {
    let mut srt = SubRipFile::new(None);
    srt.push(Subtitle::new(
        1,
        TimeDelta::milliseconds(1500),
        TimeDelta::milliseconds(1500),
        "{\\an8}Single fragment top cue".to_string(),
    ));
    srt.push(Subtitle::new(
        2,
        TimeDelta::milliseconds(1500),
        TimeDelta::milliseconds(1900),
        "Single fragment plain cue".to_string(),
    ));

    let exported = srt.export(None);

    assert_eq!(srt.len(), 2);
    assert_eq!(
        exported,
        "1\n00:00:01,500 --> 00:00:01,900\nSingle fragment plain cue\n\n"
    );
}
