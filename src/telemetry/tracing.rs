//! Execution tracing for detailed performance analysis.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use parking_lot::RwLock;

/// Unique identifier for a span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanId(u64);

impl SpanId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// A span representing a period of execution
#[derive(Debug, Clone)]
pub struct Span {
    pub id: SpanId,
    pub parent: Option<SpanId>,
    pub name: String,
    pub start: Instant,
    pub end: Option<Instant>,
    pub metadata: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
}

impl Span {
    /// Get the duration of this span
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.end.map(|end| end.duration_since(self.start))
    }
}

/// An event that occurred within a span
#[derive(Debug, Clone)]
pub struct SpanEvent {
    pub timestamp: Instant,
    pub message: String,
    pub level: EventLevel,
}

/// Event severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Tracing system for collecting execution traces
pub struct TracingSystem {
    spans: RwLock<HashMap<SpanId, Span>>,
    enabled: bool,
}

impl TracingSystem {
    pub fn new(enabled: bool) -> Self {
        Self {
            spans: RwLock::new(HashMap::new()),
            enabled,
        }
    }
    
    /// Enter a new span
    pub fn enter_span(&self, name: &str) -> SpanGuard<'_> {
        if !self.enabled {
            return SpanGuard {
                span_id: None,
                system: self,
            };
        }
        
        let span_id = SpanId::new();
        let span = Span {
            id: span_id,
            parent: None, // TODO: track parent spans via thread-local
            name: name.to_string(),
            start: Instant::now(),
            end: None,
            metadata: HashMap::new(),
            events: Vec::new(),
        };
        
        self.spans.write().insert(span_id, span);
        
        SpanGuard {
            span_id: Some(span_id),
            system: self,
        }
    }
    
    /// Record an event within the current span
    pub fn record_event(&self, span_id: SpanId, message: String, level: EventLevel) {
        if !self.enabled {
            return;
        }
        
        if let Some(span) = self.spans.write().get_mut(&span_id) {
            span.events.push(SpanEvent {
                timestamp: Instant::now(),
                message,
                level,
            });
        }
    }
    
    /// Add metadata to a span
    pub fn add_metadata(&self, span_id: SpanId, key: String, value: String) {
        if !self.enabled {
            return;
        }
        
        if let Some(span) = self.spans.write().get_mut(&span_id) {
            span.metadata.insert(key, value);
        }
    }
    
    /// Get a snapshot of all spans
    pub fn spans_snapshot(&self) -> Vec<Span> {
        self.spans.read().values().cloned().collect()
    }
    
    /// Clear all recorded spans
    pub fn clear(&self) {
        self.spans.write().clear();
    }
}

impl Default for TracingSystem {
    fn default() -> Self {
        Self::new(true)
    }
}

/// Guard that automatically closes a span when dropped
pub struct SpanGuard<'a> {
    span_id: Option<SpanId>,
    system: &'a TracingSystem,
}

impl<'a> SpanGuard<'a> {
    /// Get the span ID
    pub fn span_id(&self) -> Option<SpanId> {
        self.span_id
    }
    
    /// Record an event in this span
    pub fn event(&self, message: impl Into<String>, level: EventLevel) {
        if let Some(span_id) = self.span_id {
            self.system.record_event(span_id, message.into(), level);
        }
    }
    
    /// Add metadata to this span
    pub fn metadata(&self, key: impl Into<String>, value: impl Into<String>) {
        if let Some(span_id) = self.span_id {
            self.system.add_metadata(span_id, key.into(), value.into());
        }
    }
}

impl<'a> Drop for SpanGuard<'a> {
    fn drop(&mut self) {
        if let Some(span_id) = self.span_id {
            if let Some(span) = self.system.spans.write().get_mut(&span_id) {
                span.end = Some(Instant::now());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tracing_basic() {
        let tracing = TracingSystem::new(true);
        
        {
            let _guard = tracing.enter_span("test");
            // Span is active
        }
        // Span is closed
        
        let spans = tracing.spans_snapshot();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "test");
        assert!(spans[0].end.is_some());
    }
    
    #[test]
    fn test_span_events() {
        let tracing = TracingSystem::new(true);
        
        let guard = tracing.enter_span("test");
        guard.event("something happened", EventLevel::Info);
        guard.event("error occurred", EventLevel::Error);
        
        let span_id = guard.span_id().unwrap();
        drop(guard);
        
        let spans = tracing.spans_snapshot();
        let span = spans.iter().find(|s| s.id == span_id).unwrap();
        assert_eq!(span.events.len(), 2);
    }
    
    #[test]
    fn test_span_metadata() {
        let tracing = TracingSystem::new(true);
        
        let guard = tracing.enter_span("test");
        guard.metadata("key", "value");
        guard.metadata("worker_id", "42");
        
        let span_id = guard.span_id().unwrap();
        drop(guard);
        
        let spans = tracing.spans_snapshot();
        let span = spans.iter().find(|s| s.id == span_id).unwrap();
        assert_eq!(span.metadata.len(), 2);
        assert_eq!(span.metadata.get("key"), Some(&"value".to_string()));
    }
}
