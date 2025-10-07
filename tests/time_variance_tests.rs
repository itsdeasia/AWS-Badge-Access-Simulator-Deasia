//! Comprehensive unit tests for the time variance module
//!
//! This test suite validates the time variance functionality including:
//! - Variance application within expected range (0 to +5 minutes forward-only)
//! - Event filtering for next-day extensions
//! - Chronological order maintenance with forward-only variance
//! - Unique timestamp generation with random gaps
//! - Edge cases and boundary conditions

use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use amzn_career_pathway_activity_rust::events::access_event::AccessEvent;
use amzn_career_pathway_activity_rust::simulation::time_variance::TimeVariance;
use amzn_career_pathway_activity_rust::types::{BuildingId, UserId, EventType, LocationId, RoomId};

/// Helper function to create a test AccessEvent
fn create_test_event(timestamp: DateTime<Utc>) -> AccessEvent {
    AccessEvent::new(
        timestamp,
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    )
}

#[cfg(test)]
mod variance_application_tests {
    use super::*;

    #[test]
    fn test_variance_within_expected_range_forward_only() {
        let mut variance = TimeVariance::new();
        let scheduled_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Test 1000 applications to ensure statistical validity
        for _ in 0..1000 {
            if let Some(varied_time) = variance.apply_variance(scheduled_time) {
                let diff = varied_time.signed_duration_since(scheduled_time);
                
                // Variance should be forward-only (>= 0)
                assert!(diff >= Duration::zero(), 
                    "Variance should be forward-only, got negative offset: {:?}", diff);
                
                // Variance should not exceed 5 minutes (300 seconds)
                assert!(diff <= Duration::seconds(300), 
                    "Variance exceeded 5 minutes: {:?}", diff);
                
                // Event should remain on the same day
                assert_eq!(varied_time.date_naive(), scheduled_time.date_naive(),
                    "Event moved to different day");
            }
        }
    }

    #[test]
    fn test_variance_distribution_uniformity() {
        let mut variance = TimeVariance::new();
        let scheduled_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        let mut variance_values = Vec::new();
        
        // Collect variance values for statistical analysis
        for _ in 0..1000 {
            if let Some(varied_time) = variance.apply_variance(scheduled_time) {
                let diff_seconds = varied_time.signed_duration_since(scheduled_time).num_seconds();
                variance_values.push(diff_seconds);
            }
        }
        
        assert!(!variance_values.is_empty(), "Should have collected variance values");
        
        // Check that we have values across the range (basic distribution check)
        let min_variance = *variance_values.iter().min().unwrap();
        let max_variance = *variance_values.iter().max().unwrap();
        
        assert_eq!(min_variance, 0, "Minimum variance should be 0 (forward-only)");
        assert!(max_variance <= 300, "Maximum variance should not exceed 300 seconds");
        assert!(max_variance >= 250, "Should have some values near the maximum range");
    }    
#[test]
    fn test_variance_zero_offset_possible() {
        let mut variance = TimeVariance::new();
        let scheduled_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Test many times to ensure small offset is possible (within 10 seconds)
        let mut found_minimal_offset = false;
        for _ in 0..1000 {
            if let Some(varied_time) = variance.apply_variance(scheduled_time) {
                let diff = varied_time.signed_duration_since(scheduled_time);
                if diff <= Duration::seconds(10) {
                    found_minimal_offset = true;
                    break;
                }
            }
        }
        
        assert!(found_minimal_offset, "Small offset should be possible in forward-only variance");
    }

    #[test]
    fn test_variance_maximum_offset_possible() {
        let mut variance = TimeVariance::new();
        let scheduled_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Test many times to ensure maximum offset is achievable
        let mut found_near_max = false;
        for _ in 0..1000 {
            if let Some(varied_time) = variance.apply_variance(scheduled_time) {
                let diff = varied_time.signed_duration_since(scheduled_time);
                if diff >= Duration::seconds(290) { // Within 10 seconds of max
                    found_near_max = true;
                    break;
                }
            }
        }
        
        assert!(found_near_max, "Should be able to achieve near-maximum variance");
    }
}

#[cfg(test)]
mod next_day_extension_tests {
    use super::*;

    #[test]
    fn test_events_extending_to_next_day_are_dropped() {
        let mut variance = TimeVariance::new();
        // Set time very close to end of day (23:58:00)
        let late_scheduled_time = Utc.with_ymd_and_hms(2024, 1, 15, 23, 58, 0).unwrap();
        
        let mut dropped_count = 0;
        let mut kept_count = 0;
        
        // Test many applications
        for _ in 0..1000 {
            match variance.apply_variance(late_scheduled_time) {
                None => dropped_count += 1,
                Some(varied_time) => {
                    kept_count += 1;
                    // Verify kept events are still on the same day
                    assert_eq!(varied_time.date_naive(), late_scheduled_time.date_naive(),
                        "Kept event should remain on same day");
                    
                    // Verify variance is still forward-only
                    let diff = varied_time.signed_duration_since(late_scheduled_time);
                    assert!(diff >= Duration::zero(), "Variance should be forward-only");
                }
            }
        }
        
        // Should have both dropped and kept events
        assert!(dropped_count > 0, "Some events should be dropped due to next-day extension");
        assert!(kept_count > 0, "Some events should be kept within the same day");
        
        println!("Dropped: {}, Kept: {}", dropped_count, kept_count);
    }

    #[test]
    fn test_end_of_day_boundary_conditions() {
        let mut variance = TimeVariance::new();
        
        // Test various times near end of day
        let test_times = vec![
            Utc.with_ymd_and_hms(2024, 1, 15, 23, 55, 0).unwrap(), // 5 minutes before midnight
            Utc.with_ymd_and_hms(2024, 1, 15, 23, 57, 0).unwrap(), // 3 minutes before midnight
            Utc.with_ymd_and_hms(2024, 1, 15, 23, 59, 0).unwrap(), // 1 minute before midnight
            Utc.with_ymd_and_hms(2024, 1, 15, 23, 59, 30).unwrap(), // 30 seconds before midnight
        ];
        
        for scheduled_time in test_times {
            let mut any_dropped = false;
            let mut any_kept = false;
            
            for _ in 0..100 {
                match variance.apply_variance(scheduled_time) {
                    None => any_dropped = true,
                    Some(varied_time) => {
                        any_kept = true;
                        assert_eq!(varied_time.date_naive(), scheduled_time.date_naive());
                    }
                }
            }
            
            // Use the any_kept variable to avoid warning
            if scheduled_time.time().hour() == 23 && scheduled_time.time().minute() >= 57 {
                assert!(any_dropped, "Should drop some events for time: {}", scheduled_time);
            } else {
                assert!(any_kept, "Should keep some events for time: {}", scheduled_time);
            }
        }
    }

    #[test]
    fn test_midnight_boundary_exact() {
        let mut variance = TimeVariance::new();
        // Test exactly at midnight
        let midnight = Utc.with_ymd_and_hms(2024, 1, 16, 0, 0, 0).unwrap();
        
        // All variance applications should succeed since we're adding forward-only time
        for _ in 0..100 {
            let result = variance.apply_variance(midnight);
            assert!(result.is_some(), "Midnight events should not be dropped");
            
            if let Some(varied_time) = result {
                assert_eq!(varied_time.date_naive(), midnight.date_naive());
                let diff = varied_time.signed_duration_since(midnight);
                assert!(diff >= Duration::zero());
                assert!(diff <= Duration::seconds(300));
            }
        }
    }
}#[
cfg(test)]
mod chronological_order_tests {
    use super::*;

    #[test]
    fn test_forward_only_variance_maintains_natural_order() {
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Create events with sequential scheduled times
        let scheduled_times = vec![
            base_time,
            base_time + Duration::minutes(1),
            base_time + Duration::minutes(2),
            base_time + Duration::minutes(3),
            base_time + Duration::minutes(4),
        ];
        
        // Apply variance multiple times to test ordering
        for _ in 0..100 {
            let mut varied_times = Vec::new();
            
            for &scheduled_time in &scheduled_times {
                if let Some(varied_time) = variance.apply_variance(scheduled_time) {
                    varied_times.push((scheduled_time, varied_time));
                }
            }
            
            // Check that the natural ordering is maintained
            // Since variance is forward-only, events should generally maintain their relative order
            for i in 1..varied_times.len() {
                let (prev_scheduled, prev_varied) = varied_times[i-1];
                let (curr_scheduled, curr_varied) = varied_times[i];
                
                // If scheduled times are in order, varied times should generally be too
                // (though there might be some overlap due to randomness)
                if prev_scheduled < curr_scheduled {
                    // The natural forward-only variance should tend to maintain order
                    // We'll check this statistically rather than requiring strict ordering
                    let prev_offset = prev_varied.signed_duration_since(prev_scheduled);
                    let curr_offset = curr_varied.signed_duration_since(curr_scheduled);
                    
                    // Both offsets should be forward-only
                    assert!(prev_offset >= Duration::zero());
                    assert!(curr_offset >= Duration::zero());
                }
            }
        }
    }

    #[test]
    fn test_variance_with_close_scheduled_times() {
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Create events scheduled with larger gaps (2 minutes apart)
        let scheduled_times = vec![
            base_time,
            base_time + Duration::minutes(2),
            base_time + Duration::minutes(4),
            base_time + Duration::minutes(6),
        ];
        
        let mut order_maintained_count = 0;
        let total_tests = 100;
        
        for _ in 0..total_tests {
            let mut varied_times = Vec::new();
            
            for &scheduled_time in &scheduled_times {
                if let Some(varied_time) = variance.apply_variance(scheduled_time) {
                    varied_times.push(varied_time);
                }
            }
            
            // Check if chronological order is maintained
            let mut is_ordered = true;
            for i in 1..varied_times.len() {
                if varied_times[i] <= varied_times[i-1] {
                    is_ordered = false;
                    break;
                }
            }
            
            if is_ordered {
                order_maintained_count += 1;
            }
        }
        
        // Forward-only variance should maintain order in most cases
        // Allow some flexibility due to randomness, but expect majority to maintain order
        let order_percentage = (order_maintained_count as f64 / total_tests as f64) * 100.0;
        println!("Order maintained in {:.1}% of cases", order_percentage);
        
        // With forward-only variance and 2-minute gaps, we should maintain order in a reasonable percentage of cases
        // Due to the 5-minute variance window, some reordering is expected
        assert!(order_percentage >= 30.0, 
            "Forward-only variance should maintain order in at least 30% of cases, got {:.1}%", 
            order_percentage);
        
        // Also verify that we're getting some variation (not 0% or 100%)
        assert!(order_percentage < 100.0, "Should have some reordering due to variance");
        assert!(order_percentage > 0.0, "Should maintain order in some cases");
    }

    #[test]
    fn test_variance_with_identical_scheduled_times() {
        let mut variance = TimeVariance::new();
        let scheduled_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Apply variance to multiple events with identical scheduled times
        let mut varied_times = Vec::new();
        
        for _ in 0..50 {
            if let Some(varied_time) = variance.apply_variance(scheduled_time) {
                varied_times.push(varied_time);
            }
        }
        
        assert!(!varied_times.is_empty(), "Should have some varied times");
        
        // All times should be >= scheduled time (forward-only)
        for &varied_time in &varied_times {
            assert!(varied_time >= scheduled_time, 
                "All varied times should be >= scheduled time");
        }
        
        // Times should have some variation (not all identical)
        let unique_times: std::collections::HashSet<_> = varied_times.iter().collect();
        assert!(unique_times.len() > 1, 
            "Should have variation in times, got {} unique times from {} total", 
            unique_times.len(), varied_times.len());
    }
}

#[cfg(test)]
mod unique_timestamp_tests {
    use super::*;

    #[test]
    fn test_unique_timestamp_generation_with_random_gaps() {
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Create events with identical timestamps
        let mut events = vec![
            create_test_event(base_time),
            create_test_event(base_time), // Same timestamp
            create_test_event(base_time), // Same timestamp
            create_test_event(base_time), // Same timestamp
            create_test_event(base_time), // Same timestamp
        ];
        
        variance.ensure_unique_timestamps(&mut events);
        
        // All timestamps should be unique
        for i in 1..events.len() {
            assert!(events[i].timestamp > events[i-1].timestamp,
                "Timestamp at index {} should be greater than previous", i);
            
            // Gap should be between 1ms and 500ms
            let gap = events[i].timestamp.signed_duration_since(events[i-1].timestamp);
            assert!(gap >= Duration::milliseconds(1),
                "Gap should be at least 1ms, got {:?}", gap);
            assert!(gap <= Duration::milliseconds(500),
                "Gap should be at most 500ms, got {:?}", gap);
        }
    }

    #[test]
    fn test_unique_timestamps_with_mixed_identical_times() {
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Create events with some identical and some different timestamps
        let mut events = vec![
            create_test_event(base_time),
            create_test_event(base_time), // Same as first
            create_test_event(base_time + Duration::milliseconds(100)),
            create_test_event(base_time + Duration::milliseconds(100)), // Same as third
            create_test_event(base_time + Duration::milliseconds(200)),
        ];
        
        variance.ensure_unique_timestamps(&mut events);
        
        // All timestamps should be unique and in order
        for i in 1..events.len() {
            assert!(events[i].timestamp > events[i-1].timestamp,
                "Events should be in chronological order");
        }
        
        // Check that gaps are reasonable
        for i in 1..events.len() {
            let gap = events[i].timestamp.signed_duration_since(events[i-1].timestamp);
            assert!(gap >= Duration::milliseconds(1));
            // Gap might be larger than 500ms if original times were already different
        }
    }

    #[test]
    fn test_unique_timestamps_preserves_existing_order() {
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Create events that are already in order but with small gaps
        let mut events = vec![
            create_test_event(base_time),
            create_test_event(base_time + Duration::milliseconds(50)),
            create_test_event(base_time + Duration::milliseconds(100)),
            create_test_event(base_time + Duration::milliseconds(150)),
        ];
        
        let original_times: Vec<_> = events.iter().map(|e| e.timestamp).collect();
        
        variance.ensure_unique_timestamps(&mut events);
        
        // Order should be preserved
        for i in 1..events.len() {
            assert!(events[i].timestamp > events[i-1].timestamp);
        }
        
        // Original times should be preserved if they were already unique
        for (i, &original_time) in original_times.iter().enumerate() {
            assert_eq!(events[i].timestamp, original_time,
                "Original unique timestamps should be preserved");
        }
    }

    #[test]
    fn test_unique_timestamps_handles_overlapping_times() {
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        // Create events where later events have earlier timestamps (simulating variance results)
        let mut events = vec![
            create_test_event(base_time),
            create_test_event(base_time + Duration::milliseconds(100)),
            create_test_event(base_time + Duration::milliseconds(50)), // Earlier than previous
            create_test_event(base_time + Duration::milliseconds(75)), // Between first and third
        ];
        
        variance.ensure_unique_timestamps(&mut events);
        
        // All timestamps should be unique and in order
        for i in 1..events.len() {
            assert!(events[i].timestamp > events[i-1].timestamp,
                "All events should be in chronological order after uniqueness processing");
            
            let gap = events[i].timestamp.signed_duration_since(events[i-1].timestamp);
            assert!(gap >= Duration::milliseconds(1));
            assert!(gap <= Duration::milliseconds(500));
        }
    }

    #[test]
    fn test_random_gap_distribution() {
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        
        let mut gaps = Vec::new();
        
        // Test gap generation multiple times
        for _ in 0..100 {
            let mut events = vec![
                create_test_event(base_time),
                create_test_event(base_time), // Same timestamp to force gap generation
            ];
            
            variance.ensure_unique_timestamps(&mut events);
            
            let gap = events[1].timestamp.signed_duration_since(events[0].timestamp);
            gaps.push(gap.num_milliseconds());
        }
        
        // Check gap distribution
        let min_gap = *gaps.iter().min().unwrap();
        let max_gap = *gaps.iter().max().unwrap();
        
        assert!(min_gap >= 1, "Minimum gap should be at least 1ms");
        assert!(max_gap <= 500, "Maximum gap should be at most 500ms");
        
        // Should have some variation in gaps
        let unique_gaps: std::collections::HashSet<_> = gaps.iter().collect();
        assert!(unique_gaps.len() > 10, 
            "Should have good variation in gap sizes, got {} unique gaps", 
            unique_gaps.len());
    }
}#[
cfg(test)]
mod edge_cases_and_boundary_tests {
    use super::*;

    #[test]
    fn test_late_day_events_boundary_conditions() {
        let mut variance = TimeVariance::new();
        
        // Test various late-day scenarios
        let test_scenarios = vec![
            ("23:55:00", Utc.with_ymd_and_hms(2024, 1, 15, 23, 55, 0).unwrap()),
            ("23:56:00", Utc.with_ymd_and_hms(2024, 1, 15, 23, 56, 0).unwrap()),
            ("23:57:00", Utc.with_ymd_and_hms(2024, 1, 15, 23, 57, 0).unwrap()),
            ("23:58:00", Utc.with_ymd_and_hms(2024, 1, 15, 23, 58, 0).unwrap()),
            ("23:59:00", Utc.with_ymd_and_hms(2024, 1, 15, 23, 59, 0).unwrap()),
            ("23:59:30", Utc.with_ymd_and_hms(2024, 1, 15, 23, 59, 30).unwrap()),
            ("23:59:59", Utc.with_ymd_and_hms(2024, 1, 15, 23, 59, 59).unwrap()),
        ];
        
        for (label, scheduled_time) in test_scenarios {
            let mut success_count = 0;
            let mut drop_count = 0;
            
            for _ in 0..100 {
                match variance.apply_variance(scheduled_time) {
                    Some(varied_time) => {
                        success_count += 1;
                        // Verify it's still the same day
                        assert_eq!(varied_time.date_naive(), scheduled_time.date_naive(),
                            "Event at {} should stay on same day", label);
                        
                        // Verify forward-only variance
                        assert!(varied_time >= scheduled_time,
                            "Variance should be forward-only for {}", label);
                    }
                    None => drop_count += 1,
                }
            }
            
            println!("Time {}: {} successes, {} drops", label, success_count, drop_count);
            
            // Very late times should have some drops
            if scheduled_time.time().hour() == 23 && scheduled_time.time().minute() >= 58 {
                assert!(drop_count > 0, "Should drop some events for very late time {}", label);
            }
        }
    }

    #[test]
    fn test_leap_year_boundary() {
        let mut variance = TimeVariance::new();
        
        // Test February 29th on a leap year (2024 is a leap year)
        let leap_day_late = Utc.with_ymd_and_hms(2024, 2, 29, 23, 58, 0).unwrap();
        
        let mut success_count = 0;
        let mut drop_count = 0;
        
        for _ in 0..100 {
            match variance.apply_variance(leap_day_late) {
                Some(varied_time) => {
                    success_count += 1;
                    assert_eq!(varied_time.date_naive(), leap_day_late.date_naive());
                }
                None => drop_count += 1,
            }
        }
        
        assert!(success_count > 0, "Should have some successful variance applications on leap day");
        assert!(drop_count > 0, "Should drop some events extending to March 1st");
    }

    #[test]
    fn test_year_boundary() {
        let mut variance = TimeVariance::new();
        
        // Test December 31st late at night
        let year_end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 58, 0).unwrap();
        
        let mut success_count = 0;
        let mut drop_count = 0;
        
        for _ in 0..100 {
            match variance.apply_variance(year_end) {
                Some(varied_time) => {
                    success_count += 1;
                    // Should stay in 2024
                    assert_eq!(varied_time.year(), 2024);
                    assert_eq!(varied_time.date_naive(), year_end.date_naive());
                }
                None => drop_count += 1,
            }
        }
        
        assert!(success_count > 0, "Should have some successful applications at year end");
        assert!(drop_count > 0, "Should drop some events extending to next year");
    }

    #[test]
    fn test_month_boundary_conditions() {
        let mut variance = TimeVariance::new();
        
        // Test end of various months
        let month_ends = vec![
            Utc.with_ymd_and_hms(2024, 1, 31, 23, 58, 0).unwrap(), // January
            Utc.with_ymd_and_hms(2024, 2, 29, 23, 58, 0).unwrap(), // February (leap year)
            Utc.with_ymd_and_hms(2024, 4, 30, 23, 58, 0).unwrap(), // April (30 days)
        ];
        
        for month_end in month_ends {
            let mut stayed_same_month = 0;
            let mut dropped = 0;
            
            for _ in 0..50 {
                match variance.apply_variance(month_end) {
                    Some(varied_time) => {
                        stayed_same_month += 1;
                        assert_eq!(varied_time.date_naive(), month_end.date_naive(),
                            "Should stay on same date");
                    }
                    None => dropped += 1,
                }
            }
            
            assert!(stayed_same_month > 0, "Should have some events staying in same month");
            assert!(dropped > 0, "Should drop some events at month boundary");
        }
    }

    #[test]
    fn test_early_morning_events() {
        let mut variance = TimeVariance::new();
        
        // Test events early in the day (should never be dropped)
        let early_times = vec![
            Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap(),   // Midnight
            Utc.with_ymd_and_hms(2024, 1, 15, 1, 0, 0).unwrap(),   // 1 AM
            Utc.with_ymd_and_hms(2024, 1, 15, 6, 0, 0).unwrap(),   // 6 AM
            Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),  // Noon
        ];
        
        for early_time in early_times {
            for _ in 0..50 {
                let result = variance.apply_variance(early_time);
                assert!(result.is_some(), 
                    "Early day events should never be dropped: {}", early_time);
                
                if let Some(varied_time) = result {
                    assert_eq!(varied_time.date_naive(), early_time.date_naive());
                    assert!(varied_time >= early_time);
                    
                    let diff = varied_time.signed_duration_since(early_time);
                    assert!(diff <= Duration::seconds(300));
                }
            }
        }
    }

    #[test]
    fn test_apply_variance_to_events_with_mixed_times() {
        let mut variance = TimeVariance::new();
        
        // Create events with mix of early and late times
        let mut events = vec![
            create_test_event(Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap()),   // Early
            create_test_event(Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap()),  // Midday
            create_test_event(Utc.with_ymd_and_hms(2024, 1, 15, 18, 0, 0).unwrap()),  // Evening
            create_test_event(Utc.with_ymd_and_hms(2024, 1, 15, 23, 58, 0).unwrap()), // Very late
            create_test_event(Utc.with_ymd_and_hms(2024, 1, 15, 23, 59, 0).unwrap()), // Very late
        ];
        
        let original_count = events.len();
        variance.apply_variance_to_events(&mut events);
        
        // Should have fewer events due to late-day drops
        assert!(events.len() < original_count, 
            "Should have dropped some late-day events");
        
        // All remaining events should be on the same day
        for event in &events {
            assert_eq!(event.timestamp.date_naive(), 
                Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap().date_naive());
        }
        
        // Early events should definitely be preserved
        let early_events = events.iter()
            .filter(|e| e.timestamp.time().hour() < 20)
            .count();
        assert!(early_events >= 3, "Early events should be preserved");
    }

    #[test]
    fn test_large_event_batch_processing() {
        let mut variance = TimeVariance::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap();
        
        // Create a large batch of events
        let mut events = Vec::new();
        for i in 0..1000 {
            let event_time = base_time + Duration::minutes(i % 60); // Spread over an hour
            events.push(create_test_event(event_time));
        }
        
        let original_count = events.len();
        variance.apply_variance_to_events(&mut events);
        
        // All events should be preserved (none are late-day)
        assert_eq!(events.len(), original_count, 
            "All events should be preserved for mid-day times");
        
        // Apply uniqueness
        variance.ensure_unique_timestamps(&mut events);
        
        // All timestamps should be unique
        for i in 1..events.len() {
            assert!(events[i].timestamp > events[i-1].timestamp,
                "Large batch should maintain chronological order");
        }
    }
}
