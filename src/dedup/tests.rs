use super::DedupCache;

#[test]
fn repeated_id_is_duplicate() {
    let mut cache = DedupCache::new(4);
    assert!(!cache.is_duplicate("a"));
    assert!(cache.is_duplicate("a"));
    assert!(cache.is_duplicate("a"));
    assert!(!cache.is_duplicate("b"));
}

#[test]
fn empty_id_is_never_duplicate() {
    let mut cache = DedupCache::new(4);
    assert!(!cache.is_duplicate(""));
    assert!(!cache.is_duplicate(""));
}

#[test]
fn oldest_id_is_forgotten_when_full() {
    let mut cache = DedupCache::new(2);
    assert!(!cache.is_duplicate("a"));
    assert!(!cache.is_duplicate("b"));
    assert!(!cache.is_duplicate("c")); // evicts "a"
    assert!(cache.is_duplicate("b"));
    assert!(cache.is_duplicate("c"));
    assert!(!cache.is_duplicate("a")); // forgotten, evicts "b"
    assert!(!cache.is_duplicate("b"));
}

#[test]
fn cache_stays_bounded() {
    let mut cache = DedupCache::new(8);
    for i in 0..1000 {
        cache.is_duplicate(&i.to_string());
    }
    assert_eq!(cache.order.len(), 8);
    assert_eq!(cache.seen.len(), 8);
}

#[test]
fn zero_capacity_disables_dedup() {
    let mut cache = DedupCache::new(0);
    assert!(!cache.is_duplicate("a"));
    assert!(!cache.is_duplicate("a"));
}
