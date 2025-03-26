//! Audio processing tests

// tests/audio_tests.rs
#[test]
fn test_bpm_detection() {
    let detector = BpmDetector::new(44100);
    let test_signal = generate_test_signal(120.0, 44100);
    let bpm = detector.detect(&test_signal).unwrap();
    assert!((bpm - 120.0).abs() < 1.0);
}