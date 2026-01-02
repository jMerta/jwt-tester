mod common;
use common::assert_exit;

#[test]
fn split_rejects_wrong_segment_count() {
    assert_exit(&["split", "abc.def"], 10);
}

#[test]
fn split_rejects_invalid_base64_segments() {
    assert_exit(&["split", "!!!!.!!!!.!!!!"], 10);
}
