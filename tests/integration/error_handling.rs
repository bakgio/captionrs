use captionrs::subripfile::SubtitleError;
use captionrs::utils::time::timestamp_from_seconds;

#[test]
fn test_invalid_format_error() {
    // Test that InvalidFormat error variant exists and can be created
    // (even if current parsing is permissive and rarely triggers it)

    let error = SubtitleError::InvalidFormat("Test error message".to_string());
    match error {
        SubtitleError::InvalidFormat(msg) => {
            assert_eq!(msg, "Test error message");
        }
        _ => panic!("Failed to create InvalidFormat error"),
    }

    // Test error display formatting
    let error2 = SubtitleError::InvalidFormat("Bad format".to_string());
    let error_string = format!("{}", error2);
    assert!(error_string.contains("Invalid format"));
    assert!(error_string.contains("Bad format"));

    // This ensures the InvalidFormat variant is part of the public API
    // and works correctly even if current parsing logic is permissive
}

#[test]
fn test_timestamp_from_seconds() {
    // Test basic conversion (current implementation uses dots, not commas)
    assert_eq!(timestamp_from_seconds(0.0), "00:00:00.000");
    assert_eq!(timestamp_from_seconds(1.0), "00:00:01.000");
    assert_eq!(timestamp_from_seconds(60.0), "00:01:00.000");
    assert_eq!(timestamp_from_seconds(3600.0), "01:00:00.000");

    // Test fractional seconds
    assert_eq!(timestamp_from_seconds(1.5), "00:00:01.500");
    assert_eq!(timestamp_from_seconds(1.123), "00:00:01.123");
    assert_eq!(timestamp_from_seconds(1.001), "00:00:01.000");

    // Test complex time
    assert_eq!(timestamp_from_seconds(3661.5), "01:01:01.500");
    assert_eq!(timestamp_from_seconds(7323.789), "02:02:03.789");

    // Test edge cases
    assert_eq!(timestamp_from_seconds(0.999), "00:00:00.999");
    assert_eq!(timestamp_from_seconds(59.999), "00:00:59.999");
}
