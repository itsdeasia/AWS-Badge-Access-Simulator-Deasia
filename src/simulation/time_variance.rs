//! Time variance module for applying realistic timing variation to scheduled events
//!
//! This module provides functionality to add forward-only time variance to badge access events,
//! making the simulation data more realistic by eliminating artificial clustering at exact
//! scheduled times.

use chrono::{DateTime, Duration, Utc};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use crate::events::access_event::AccessEvent;

/// Default variance window in seconds (300.0 seconds = 5 minutes forward-only)
const DEFAULT_VARIANCE_WINDOW_SECONDS: f64 = 300.0;

/// Maximum timestamp gap in milliseconds for random timestamp separation (500ms)
const MAX_TIMESTAMP_GAP_MS: i64 = 500;

/// TimeVariance struct for applying realistic time variance to scheduled events
///
/// This struct provides methods to apply forward-only random time offsets to scheduled
/// event times, creating more realistic timing patterns while maintaining chronological
/// ordering and preventing events from extending into the next day.
#[derive(Debug)]
pub struct TimeVariance {
    variance_window_seconds: f64,
    rng: ThreadRng,
}

impl TimeVariance {
    /// Create a new TimeVariance instance with default settings
    pub fn new() -> Self {
        Self { variance_window_seconds: DEFAULT_VARIANCE_WINDOW_SECONDS, rng: thread_rng() }
    }

    /// Apply forward-only variance to a scheduled time
    /// Returns None if the varied time would extend into the next day
    pub fn apply_variance(&mut self, scheduled_time: DateTime<Utc>) -> Option<DateTime<Utc>> {
        // Generate random variance between 0 and variance_window_seconds (forward-only)
        let variance_seconds = self.rng.gen_range(0.0..=self.variance_window_seconds);
        let variance_duration = Duration::milliseconds((variance_seconds * 1000.0) as i64);
        let varied_time = scheduled_time + variance_duration;

        // Drop events that would extend into the next day
        if varied_time.date_naive() != scheduled_time.date_naive() {
            return None;
        }

        Some(varied_time)
    }

    /// Apply forward-only variance to a collection of events
    /// Events that would extend into the next day are filtered out
    pub fn apply_variance_to_events(&mut self, events: &mut Vec<AccessEvent>) {
        // Apply variance to each event and filter out events that extend to next day
        events.retain_mut(|event| {
            if let Some(varied_time) = self.apply_variance(event.timestamp) {
                event.timestamp = varied_time;
                true
            } else {
                false // Remove events that would extend into the next day
            }
        });
    }

    /// Ensure unique timestamps by adding random gaps when timestamps would be identical
    /// Uses random gaps between 1ms and 500ms to prevent clustering
    pub fn ensure_unique_timestamps(&mut self, events: &mut [AccessEvent]) {
        // Since variance is forward-only, events are naturally in chronological order
        // We only need to ensure uniqueness by checking adjacent timestamps
        for i in 1..events.len() {
            if events[i].timestamp <= events[i-1].timestamp {
                // Add a random gap between 1ms and 500ms to prevent clustering
                let gap_ms = self.rng.gen_range(1..=MAX_TIMESTAMP_GAP_MS);
                events[i].timestamp = events[i-1].timestamp + Duration::milliseconds(gap_ms);
            }
        }
    }
}

impl Default for TimeVariance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_time_variance_creation() {
        let variance = TimeVariance::new();
        assert_eq!(variance.variance_window_seconds, DEFAULT_VARIANCE_WINDOW_SECONDS);
    }

    #[test]
    fn test_apply_variance_within_range() {
        let mut variance = TimeVariance::new();
        let scheduled_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();

        // Test multiple applications to verify variance is within expected range
        for _ in 0..100 {
            if let Some(varied_time) = variance.apply_variance(scheduled_time) {
                let diff = varied_time.signed_duration_since(scheduled_time);
                assert!(diff >= Duration::zero());
                assert!(diff <= Duration::seconds(300)); // 5 minutes
                assert_eq!(varied_time.date_naive(), scheduled_time.date_naive());
            }
        }
    }

    #[test]
    fn test_apply_variance_drops_next_day_events() {
        let mut variance = TimeVariance::new();
        // Set time very close to end of day
        let scheduled_time = Utc.with_ymd_and_hms(2024, 1, 15, 23, 58, 0).unwrap();

        // Some applications should return None when variance would extend to next day
        let mut none_count = 0;
        let mut some_count = 0;

        for _ in 0..100 {
            match variance.apply_variance(scheduled_time) {
                None => none_count += 1,
                Some(varied_time) => {
                    some_count += 1;
                    assert_eq!(varied_time.date_naive(), scheduled_time.date_naive());
                }
            }
        }

        // Should have some events dropped due to next-day extension
        assert!(none_count > 0, "Expected some events to be dropped");
        assert!(some_count > 0, "Expected some events to be kept");
    }

    #[test]
    fn test_default_implementation() {
        let variance = TimeVariance::default();
        assert_eq!(variance.variance_window_seconds, DEFAULT_VARIANCE_WINDOW_SECONDS);
    }

    #[test]
    fn test_apply_variance_to_events() {
        use crate::types::{BuildingId, UserId, EventType, LocationId, RoomId};
        
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Create test events with same timestamp
        let mut events = vec![
            AccessEvent::new(
                base_time,
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
            AccessEvent::new(
                base_time + Duration::minutes(1),
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
        ];

        let original_count = events.len();
        variance.apply_variance_to_events(&mut events);

        // All events should still be present (not extending to next day)
        assert_eq!(events.len(), original_count);

        // All events should have variance applied (timestamps should be different from original)
        for (i, event) in events.iter().enumerate() {
            let original_time = if i == 0 { base_time } else { base_time + Duration::minutes(1) };
            assert!(event.timestamp >= original_time);
            assert!(event.timestamp <= original_time + Duration::seconds(300));
            assert_eq!(event.timestamp.date_naive(), original_time.date_naive());
        }
    }

    #[test]
    fn test_apply_variance_to_events_filters_next_day() {
        use crate::types::{BuildingId, UserId, EventType, LocationId, RoomId};
        
        let mut variance = TimeVariance::new();
        // Set time very close to end of day
        let late_time = Utc.with_ymd_and_hms(2024, 1, 15, 23, 58, 0).unwrap();
        
        // Create multiple events at the same late time, some should be filtered out when variance is applied
        let mut events = vec![];
        for _ in 0..50 {
            events.push(AccessEvent::new(
                late_time,
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ));
        }

        let original_count = events.len();
        variance.apply_variance_to_events(&mut events);

        // Some events should be filtered out due to next-day extension
        assert!(events.len() < original_count);

        // All remaining events should be on the same day as the original scheduled time
        for event in &events {
            assert_eq!(event.timestamp.date_naive(), late_time.date_naive());
        }
    }

    #[test]
    fn test_ensure_unique_timestamps() {
        use crate::types::{BuildingId, UserId, EventType, LocationId, RoomId};
        
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Create events with identical timestamps
        let mut events = vec![
            AccessEvent::new(
                base_time,
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
            AccessEvent::new(
                base_time, // Same timestamp
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
            AccessEvent::new(
                base_time, // Same timestamp
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
        ];

        variance.ensure_unique_timestamps(&mut events);

        // All timestamps should be unique
        for i in 1..events.len() {
            assert!(events[i].timestamp > events[i-1].timestamp);
            
            // Gap should be between 1ms and 500ms
            let gap = events[i].timestamp.signed_duration_since(events[i-1].timestamp);
            assert!(gap >= Duration::milliseconds(1));
            assert!(gap <= Duration::milliseconds(MAX_TIMESTAMP_GAP_MS));
        }
    }

    #[test]
    fn test_ensure_unique_timestamps_maintains_order() {
        use crate::types::{BuildingId, UserId, EventType, LocationId, RoomId};
        
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Create events with slightly different timestamps
        let mut events = vec![
            AccessEvent::new(
                base_time,
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
            AccessEvent::new(
                base_time + Duration::milliseconds(100),
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
            AccessEvent::new(
                base_time + Duration::milliseconds(200),
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
        ];

        variance.ensure_unique_timestamps(&mut events);

        // Events should maintain chronological order
        for i in 1..events.len() {
            assert!(events[i].timestamp > events[i-1].timestamp);
        }
    }

    #[test]
    fn test_ensure_unique_timestamps_with_overlapping_times() {
        use crate::types::{BuildingId, UserId, EventType, LocationId, RoomId};
        
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Create events where some have overlapping times (simulating variance results)
        let mut events = vec![
            AccessEvent::new(
                base_time,
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
            AccessEvent::new(
                base_time + Duration::milliseconds(50),
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
            AccessEvent::new(
                base_time + Duration::milliseconds(25), // Earlier than previous
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
        ];

        variance.ensure_unique_timestamps(&mut events);

        // All timestamps should be unique and in order
        for i in 1..events.len() {
            assert!(events[i].timestamp > events[i-1].timestamp);
        }
    }

    #[test]
    fn test_max_timestamp_gap_constant() {
        // Verify the constant is set correctly
        assert_eq!(MAX_TIMESTAMP_GAP_MS, 500);
    }
}
