use super::resolve_stream_id;

#[test]
fn resolve_stream_id_keeps_supplied_id() {
    assert_eq!(resolve_stream_id("mod", String::from("existing-id")), "existing-id");
}

#[test]
fn resolve_stream_id_generates_when_empty() {
    let generated = resolve_stream_id("mod", String::new());
    assert!(!generated.is_empty());
    assert!(generated.starts_with("mod-"));
}

#[test]
fn resolve_stream_id_generates_distinct_ids() {
    let first = resolve_stream_id("mod", String::new());
    let second = resolve_stream_id("mod", String::new());
    assert_ne!(first, second);
}
