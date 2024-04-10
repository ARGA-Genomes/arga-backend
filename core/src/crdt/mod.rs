pub mod hlc;
pub mod lww;

use bigdecimal::BigDecimal;
use chrono::Utc;

use self::hlc::HybridTimestamp;

#[derive(Debug, Copy, Clone)]
pub struct Version(HybridTimestamp);

impl Version {
    pub fn new() -> Version {
        let ts: HybridTimestamp = Utc::now().into();
        Version(ts)
    }

    /// Get the next frame version.
    ///
    /// This will generate a new hybrid logical clock and if it is greater
    /// than the current version it will return it. However, if the current
    /// clock is ahead we keep incrementing it.
    pub fn next(&self) -> Version {
        let ts: HybridTimestamp = Utc::now().into();
        if ts > self.0 {
            Version(ts)
        } else {
            Version(self.0.inc())
        }
    }

    pub fn inc(&self) -> Version {
        Version(self.0.inc())
    }
}

impl From<Version> for BigDecimal {
    fn from(value: Version) -> Self {
        value.0.as_u64().into()
    }
}

#[derive(Debug)]
pub struct Frame<Op> {
    pub current: Version,
    pub next: Version,
    pub operations: Vec<Op>,
}

impl<Op> Frame<Op> {
    pub fn new(last_version: Version) -> Frame<Op> {
        Frame {
            next: last_version.next(),
            current: last_version,
            operations: vec![],
        }
    }

    pub fn push(&mut self, operation: Op) -> Version {
        self.operations.push(operation);

        // increment the version for the next op if there is one. we could do
        // a next() instead but frames are transactional so having them incrementing
        // logically is more consistent
        self.current = self.next;
        self.next = self.next.inc();
        self.next
    }
}
