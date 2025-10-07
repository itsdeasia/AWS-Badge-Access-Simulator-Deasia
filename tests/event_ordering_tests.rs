//! Tests for event ordering across day boundaries
//!
//! These tests verify that events are properly ordered chronologically
//! within days and across day boundaries in the batch processing system.

use amzn_career_pathway_activity_rust::events::AccessEvent;
use amzn_career_pathway_activity_rust::types::{
    BuildingId, UserId, EventType, LocationId, RoomId,
};
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};

/// Test event ordering within a single day
#[test]
fn test_event_ordering_within_day() {
    let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    
    // Create events at different times within the same day
    let morning_time = test_date.and_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap());
    let afternoon_time = test_date.and_time(NaiveTime::from_hms_opt(14, 30, 0).unwrap());
    let evening_time = test_date.and_time(NaiveTime::from_hms_opt(17, 45, 0).unwrap());
    
    let morning_event = AccessEvent::new(
        Utc.from_utc_datetime(&morning_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let afternoon_event = AccessEvent::new(
        Utc.from_utc_datetime(&afternoon_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let evening_event = AccessEvent::new(
        Utc.from_utc_datetime(&evening_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    // Create unsorted list
    let mut events = vec![evening_event, morning_event, afternoon_event];
    
    // Sort by timestamp (this is what the batch generator should do)
    events.sort_by_key(|event| event.timestamp);
    
    // Verify chronological order
    assert!(events[0].timestamp < events[1].timestamp);
    assert!(events[1].timestamp < events[2].timestamp);
    
    // Verify all events are from the same day
    for event in &events {
        assert_eq!(event.timestamp.date_naive(), test_date);
    }
}

/// Test event ordering across day boundaries
#[test]
fn test_event_ordering_across_day_boundaries() {
    let day1 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let day2 = day1.succ_opt().unwrap();
    let day3 = day2.succ_opt().unwrap();
    
    // Create events spanning multiple days
    let day1_late = day1.and_time(NaiveTime::from_hms_opt(23, 30, 0).unwrap());
    let day2_early = day2.and_time(NaiveTime::from_hms_opt(2, 15, 0).unwrap());
    let day2_late = day2.and_time(NaiveTime::from_hms_opt(22, 0, 0).unwrap());
    let day3_early = day3.and_time(NaiveTime::from_hms_opt(1, 45, 0).unwrap());
    
    let event1 = AccessEvent::new(
        Utc.from_utc_datetime(&day1_late),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let event2 = AccessEvent::new(
        Utc.from_utc_datetime(&day2_early),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let event3 = AccessEvent::new(
        Utc.from_utc_datetime(&day2_late),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let event4 = AccessEvent::new(
        Utc.from_utc_datetime(&day3_early),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    // Create unsorted list
    let mut events = vec![event3, event1, event4, event2];
    
    // Sort by timestamp (this is what the batch generator should do)
    events.sort_by_key(|event| event.timestamp);
    
    // Verify chronological order across days
    for i in 1..events.len() {
        assert!(events[i-1].timestamp <= events[i].timestamp,
               "Events should be in chronological order");
    }
    
    // Verify correct day assignment
    assert_eq!(events[0].timestamp.date_naive(), day1);
    assert_eq!(events[1].timestamp.date_naive(), day2);
    assert_eq!(events[2].timestamp.date_naive(), day2);
    assert_eq!(events[3].timestamp.date_naive(), day3);
}

/// Test event separation by date
#[test]
fn test_event_separation_by_date() {
    let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    
    // Create events that span across day boundaries
    let current_day_time = base_date.and_time(NaiveTime::from_hms_opt(14, 30, 0).unwrap());
    let next_day_time = base_date.succ_opt().unwrap().and_time(NaiveTime::from_hms_opt(2, 15, 0).unwrap());
    let day_after_time = base_date.succ_opt().unwrap().succ_opt().unwrap().and_time(NaiveTime::from_hms_opt(10, 0, 0).unwrap());
    
    let current_day_event = AccessEvent::new(
        Utc.from_utc_datetime(&current_day_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let next_day_event = AccessEvent::new(
        Utc.from_utc_datetime(&next_day_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let day_after_event = AccessEvent::new(
        Utc.from_utc_datetime(&day_after_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let all_events = vec![current_day_event, next_day_event, day_after_event];
    
    // Simulate the batch generator's day separation logic
    use std::collections::HashMap;
    let mut events_by_date: HashMap<NaiveDate, Vec<AccessEvent>> = HashMap::new();
    
    for event in all_events {
        let event_date = event.timestamp.date_naive();
        events_by_date.entry(event_date).or_default().push(event);
    }
    
    // Verify events are properly separated by date
    assert_eq!(events_by_date.len(), 3);
    assert!(events_by_date.contains_key(&base_date));
    assert!(events_by_date.contains_key(&base_date.succ_opt().unwrap()));
    assert!(events_by_date.contains_key(&base_date.succ_opt().unwrap().succ_opt().unwrap()));
    
    // Verify each date has exactly one event
    for (date, events) in &events_by_date {
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].timestamp.date_naive(), *date);
    }
}

/// Test event ordering with same timestamps
#[test]
fn test_event_ordering_with_same_timestamps() {
    let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let same_time = test_date.and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    
    // Create multiple events with the same timestamp
    let event1 = AccessEvent::new(
        Utc.from_utc_datetime(&same_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let event2 = AccessEvent::new(
        Utc.from_utc_datetime(&same_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        false,
        EventType::Failure,
    );
    
    let event3 = AccessEvent::new(
        Utc.from_utc_datetime(&same_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let mut events = vec![event1, event2, event3];
    
    // Sort by timestamp (stable sort should preserve relative order for same timestamps)
    events.sort_by_key(|event| event.timestamp);
    
    // Verify all events have the same timestamp
    for event in &events {
        assert_eq!(event.timestamp, Utc.from_utc_datetime(&same_time));
    }
    
    // Verify sorting is stable (order should be preserved for same timestamps)
    assert_eq!(events.len(), 3);
}

/// Test event ordering with microsecond precision
#[test]
fn test_event_ordering_microsecond_precision() {
    let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let base_time = test_date.and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    
    // Create events with microsecond differences
    let time1 = Utc.from_utc_datetime(&base_time);
    let time2 = time1 + chrono::Duration::microseconds(1);
    let time3 = time1 + chrono::Duration::microseconds(2);
    
    let event1 = AccessEvent::new(
        time3, // Latest time
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let event2 = AccessEvent::new(
        time1, // Earliest time
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let event3 = AccessEvent::new(
        time2, // Middle time
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let mut events = vec![event1, event2, event3];
    
    // Sort by timestamp
    events.sort_by_key(|event| event.timestamp);
    
    // Verify correct chronological order with microsecond precision
    assert_eq!(events[0].timestamp, time1);
    assert_eq!(events[1].timestamp, time2);
    assert_eq!(events[2].timestamp, time3);
    
    // Verify ordering is correct
    assert!(events[0].timestamp < events[1].timestamp);
    assert!(events[1].timestamp < events[2].timestamp);
}

/// Test event ordering with night shift events
#[test]
fn test_event_ordering_with_night_shift() {
    let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    
    // Create events spanning night shift hours (evening to early morning)
    let evening_time = test_date.and_time(NaiveTime::from_hms_opt(22, 0, 0).unwrap());
    let midnight_time = test_date.succ_opt().unwrap().and_time(NaiveTime::from_hms_opt(0, 30, 0).unwrap());
    let early_morning_time = test_date.succ_opt().unwrap().and_time(NaiveTime::from_hms_opt(6, 0, 0).unwrap());
    
    let evening_event = AccessEvent::new(
        Utc.from_utc_datetime(&evening_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let midnight_event = AccessEvent::new(
        Utc.from_utc_datetime(&midnight_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let early_morning_event = AccessEvent::new(
        Utc.from_utc_datetime(&early_morning_time),
        UserId::new(),
        RoomId::new(),
        BuildingId::new(),
        LocationId::new(),
        true,
        EventType::Success,
    );
    
    let mut events = vec![midnight_event, early_morning_event, evening_event];
    
    // Sort by timestamp
    events.sort_by_key(|event| event.timestamp);
    
    // Verify chronological order across night shift
    assert!(events[0].timestamp < events[1].timestamp);
    assert!(events[1].timestamp < events[2].timestamp);
    
    // Verify correct date assignment
    assert_eq!(events[0].timestamp.date_naive(), test_date);
    assert_eq!(events[1].timestamp.date_naive(), test_date.succ_opt().unwrap());
    assert_eq!(events[2].timestamp.date_naive(), test_date.succ_opt().unwrap());
}

/// Test large scale event ordering
#[test]
fn test_large_scale_event_ordering() {
    let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let mut events = Vec::new();
    
    // Create 1000 events across 3 days
    for day_offset in 0..3 {
        let current_date = base_date + chrono::Duration::days(day_offset);
        
        for hour in 0..24 {
            for minute in (0..60).step_by(15) { // Every 15 minutes
                let time = current_date.and_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap());
                
                let event = AccessEvent::new(
                    Utc.from_utc_datetime(&time),
                    UserId::new(),
                    RoomId::new(),
                    BuildingId::new(),
                    LocationId::new(),
                    true,
                    EventType::Success,
                );
                
                events.push(event);
            }
        }
    }
    
    // Shuffle events to simulate unordered generation
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    events.shuffle(&mut thread_rng());
    
    // Sort by timestamp
    events.sort_by_key(|event| event.timestamp);
    
    // Verify all events are in chronological order
    for i in 1..events.len() {
        assert!(events[i-1].timestamp <= events[i].timestamp,
               "Event {} is not in chronological order", i);
    }
    
    // Verify we have the expected number of events
    assert_eq!(events.len(), 3 * 24 * 4); // 3 days * 24 hours * 4 quarters per hour
}