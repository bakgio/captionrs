use captionrs::SubRipFile;
use captionrs::processors::BaseProcessor;
use captionrs::processors::rtl::RTLFixer;
use captionrs::subripfile::Subtitle;
use captionrs::utils::time::{ms_from_timestamp, timestamp_from_ms, timestamp_from_seconds};
use chrono::TimeDelta;

#[test]
fn direct_rtl_processor_matches_shared_changed_status_behavior() {
    let mut srt = SubRipFile::new(None);
    srt.push(Subtitle::new(
        1,
        TimeDelta::seconds(1),
        TimeDelta::seconds(2),
        "\u{200f}\u{00d7}\u{00a9}\u{00d7}\u{0153}\u{00d7}\u{2022}\u{00d7}\u{009d}\n\u{00d7}\u{00a2}\u{00d7}\u{2022}\u{00d7}\u{0153}\u{00d7}\u{009d}".to_string(),
    ));

    let fixer = RTLFixer::new();
    let (processed, changed) = fixer.process(srt, Some("heb")).unwrap();

    assert_eq!(
        processed[0].content,
        "\u{202b}\u{00d7}\u{00a9}\u{00d7}\u{0153}\u{00d7}\u{2022}\u{00d7}\u{009d}\n\u{202b}\u{00d7}\u{00a2}\u{00d7}\u{2022}\u{00d7}\u{0153}\u{00d7}\u{009d}"
    );
    assert!(!changed);
}

#[test]
fn ms_from_timestamp_accepts_t_prefix() {
    assert_eq!(ms_from_timestamp("T:01:02:03.004").unwrap(), 3_723_004);
}

#[test]
fn timestamp_from_ms_truncates_fractional_milliseconds() {
    assert_eq!(timestamp_from_ms(1_000.9), "00:00:01.000");
}

#[test]
fn timestamp_from_seconds_matches_shared_float_precision_behavior() {
    assert_eq!(timestamp_from_seconds(1.001), "00:00:01.000");
}
