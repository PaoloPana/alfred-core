use std::collections::{HashSet, VecDeque};

/// Bounded cache of recently seen message ids, used to drop the duplicates that MQTT `QoS` 1
/// can deliver. When full, the oldest id is forgotten.
#[derive(Debug)]
pub struct DedupCache {
    capacity: usize,
    order: VecDeque<String>,
    seen: HashSet<String>,
}

impl DedupCache {
    pub fn new(capacity: usize) -> Self {
        Self { capacity, order: VecDeque::with_capacity(capacity), seen: HashSet::with_capacity(capacity) }
    }

    /// Returns `true` if `id` was already seen (a duplicate), otherwise remembers it and returns
    /// `false`. Empty ids (messages without an id) are never considered duplicates.
    pub fn is_duplicate(&mut self, id: &str) -> bool {
        if id.is_empty() || self.capacity == 0 {
            return false;
        }
        if self.seen.contains(id) {
            return true;
        }
        if self.order.len() == self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.seen.remove(&oldest);
            }
        }
        self.order.push_back(id.to_string());
        self.seen.insert(id.to_string());
        false
    }
}

#[cfg(test)]
mod tests;
