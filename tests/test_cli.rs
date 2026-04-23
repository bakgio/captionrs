use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use encoding_rs::{UTF_16LE, WINDOWS_1252};
use tempfile::tempdir;

fn captionrs_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_captionrs"))
}

fn assert_success(output: std::process::Output) -> std::process::Output {
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn write_file(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
}

fn stdout_text(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn version_command_prints_project_version_and_repo() {
    let output = assert_success(
        Command::new(captionrs_bin())
            .arg("version")
            .output()
            .unwrap(),
    );
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("CaptionRS version"));
    assert!(stdout.contains("https://github.com/bakgio/captionrs"));
}

#[test]
fn convert_respects_requested_output_encoding() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input.vtt");
    let utf8_output = temp_dir.path().join("output_utf8.srt");
    let utf8_sig_output = temp_dir.path().join("output_utf8_sig.srt");

    write_file(
        &input_path,
        "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nHello,world\n",
    );

    assert_success(
        Command::new(captionrs_bin())
            .arg("convert")
            .arg(&input_path)
            .arg("-o")
            .arg(&utf8_output)
            .arg("-e")
            .arg("utf-8")
            .output()
            .unwrap(),
    );
    assert_success(
        Command::new(captionrs_bin())
            .arg("convert")
            .arg(&input_path)
            .arg("-o")
            .arg(&utf8_sig_output)
            .arg("-e")
            .arg("utf-8-sig")
            .output()
            .unwrap(),
    );

    let utf8_bytes = fs::read(&utf8_output).unwrap();
    let utf8_sig_bytes = fs::read(&utf8_sig_output).unwrap();

    assert!(!utf8_bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
    assert!(utf8_sig_bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
}

#[test]
fn convert_default_output_uses_lf_line_endings() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input_default_eol.vtt");
    let output_path = temp_dir.path().join("output_default_eol.srt");

    write_file(
        &input_path,
        "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nHello\n",
    );

    assert_success(
        Command::new(captionrs_bin())
            .arg("convert")
            .arg(&input_path)
            .arg("-o")
            .arg(&output_path)
            .arg("-n")
            .output()
            .unwrap(),
    );

    let written = fs::read(&output_path).unwrap();
    assert!(!written.windows(2).any(|window| window == [b'\r', b'\n']));
    assert!(written.contains(&b'\n'));
}

#[test]
fn convert_reports_status_and_supports_windows_1252_output() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("input_cp1252.vtt");
    let output_path = temp_dir.path().join("output_cp1252.srt");

    write_file(
        &input_path,
        "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nCafe é\n",
    );

    let output = assert_success(
        Command::new(captionrs_bin())
            .arg("convert")
            .arg(&input_path)
            .arg("-o")
            .arg(&output_path)
            .arg("-e")
            .arg("windows-1252")
            .output()
            .unwrap(),
    );
    let stdout = stdout_text(&output);
    let written = fs::read(&output_path).unwrap();
    let (decoded, _, had_errors) = WINDOWS_1252.decode(&written);

    assert!(!had_errors);
    assert!(decoded.contains("Cafe é"));
    assert!(stdout.contains("Subtitle format: WebVTT"));
    assert!(stdout.contains("Converted subtitle to SubRip (SRT)"));
    assert!(stdout.contains("Processed subtitle but no issues were found..."));
    assert!(stdout.contains("Saved to:"));
}

#[test]
fn process_mend_does_not_save_when_nothing_changed() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("clean.srt");
    let expected_output = temp_dir.path().join("clean_mend.srt");

    write_file(
        &input_path,
        "1\n00:00:00,000 --> 00:00:01,000\nHello there.\n\n",
    );

    assert_success(
        Command::new(captionrs_bin())
            .arg("process")
            .arg(&input_path)
            .arg("mend")
            .output()
            .unwrap(),
    );

    assert!(!expected_output.exists());
}

#[test]
fn convert_unrecognized_input_does_not_create_output() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("unknown.bin");
    let output_path = temp_dir.path().join("unknown.srt");

    write_file(&input_path, "this is not a supported subtitle format");

    let output = assert_success(
        Command::new(captionrs_bin())
            .arg("convert")
            .arg(&input_path)
            .arg("-o")
            .arg(&output_path)
            .output()
            .unwrap(),
    );
    let stdout = stdout_text(&output);

    assert!(stdout.contains("Subtitle format was unrecognized..."));
    assert!(!output_path.exists());
}

#[test]
fn process_mend_uses_suffix_output_and_keep_short_gaps() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("gaps.srt");
    let default_output = temp_dir.path().join("gaps_mend.srt");
    let keep_gap_input = temp_dir.path().join("gaps_keep.srt");
    let keep_gap_output = temp_dir.path().join("gaps_keep_mend.srt");

    let srt = concat!(
        "1\n",
        "00:00:00,000 --> 00:00:01,000\n",
        "Hello there.\n\n",
        "2\n",
        "00:00:01,050 --> 00:00:02,000\n",
        "General Kenobi.\n\n"
    );
    write_file(&input_path, srt);
    write_file(&keep_gap_input, srt);

    assert_success(
        Command::new(captionrs_bin())
            .arg("process")
            .arg(&input_path)
            .arg("mend")
            .output()
            .unwrap(),
    );
    assert!(default_output.exists());

    assert_success(
        Command::new(captionrs_bin())
            .arg("process")
            .arg(&keep_gap_input)
            .arg("--keep-short-gaps")
            .arg("mend")
            .output()
            .unwrap(),
    );
    assert!(!keep_gap_output.exists());
}

#[test]
fn process_strip_sdh_runs_post_processing_unless_disabled() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("dialogue.srt");
    let no_post_input_path = temp_dir.path().join("dialogue_no_post.srt");
    let default_output = temp_dir.path().join("dialogue_sdh_stripped.srt");
    let no_post_output = temp_dir.path().join("dialogue_no_post_sdh_stripped.srt");

    let srt = "1\n00:00:00,000 --> 00:00:02,000\n[MUSIC]\nHello,world\n\n";
    write_file(&input_path, srt);
    write_file(&no_post_input_path, srt);

    assert_success(
        Command::new(captionrs_bin())
            .arg("process")
            .arg(&input_path)
            .arg("strip-sdh")
            .output()
            .unwrap(),
    );
    assert_success(
        Command::new(captionrs_bin())
            .arg("process")
            .arg(&no_post_input_path)
            .arg("--no-post-processing")
            .arg("strip-sdh")
            .output()
            .unwrap(),
    );

    let default_contents = fs::read_to_string(default_output).unwrap();
    let no_post_contents = fs::read_to_string(no_post_output).unwrap();

    assert!(default_contents.contains("Hello, world"));
    assert!(no_post_contents.contains("Hello,world"));
}

#[test]
fn process_respects_requested_output_encoding() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("messy.srt");
    let utf8_output = temp_dir.path().join("messy_utf8.srt");
    let utf8_sig_output = temp_dir.path().join("messy_utf8_sig.srt");

    write_file(
        &input_path,
        "1\n00:00:00,000 --> 00:00:01,000\nHello,world\n\n",
    );

    assert_success(
        Command::new(captionrs_bin())
            .arg("process")
            .arg(&input_path)
            .arg("-o")
            .arg(&utf8_output)
            .arg("-e")
            .arg("utf-8")
            .arg("mend")
            .output()
            .unwrap(),
    );
    assert_success(
        Command::new(captionrs_bin())
            .arg("process")
            .arg(&input_path)
            .arg("-o")
            .arg(&utf8_sig_output)
            .arg("-e")
            .arg("utf-8-sig")
            .arg("mend")
            .output()
            .unwrap(),
    );

    let utf8_bytes = fs::read(&utf8_output).unwrap();
    let utf8_sig_bytes = fs::read(&utf8_sig_output).unwrap();

    assert!(!utf8_bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
    assert!(utf8_sig_bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
}

#[test]
fn process_supports_utf16_output() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("messy_utf16.srt");
    let output_path = temp_dir.path().join("messy_utf16_out.srt");

    write_file(
        &input_path,
        "1\n00:00:00,000 --> 00:00:01,000\nHello,world\n\n",
    );

    assert_success(
        Command::new(captionrs_bin())
            .arg("process")
            .arg(&input_path)
            .arg("-o")
            .arg(&output_path)
            .arg("-e")
            .arg("utf-16")
            .arg("mend")
            .output()
            .unwrap(),
    );

    let written = fs::read(&output_path).unwrap();
    let (decoded, had_errors) = UTF_16LE.decode_without_bom_handling(&written[2..]);

    assert!(written.starts_with(&[0xFF, 0xFE]));
    assert!(!had_errors);
    assert!(decoded.contains("Hello, world"));
}
