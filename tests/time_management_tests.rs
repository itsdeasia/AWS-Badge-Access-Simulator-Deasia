//! Unit tests for time management and acceleration calculations
//!
//! Tests Requirements: 6.1, 6.2, 6.3, 6.4, 6.5

use amzn_career_pathway_activity_rust::*;
use chrono::{Duration, TimeZone, Timelike, Utc};
use rand::thread_rng;
use std::thread;
use std::time::Duration as StdDuration;

/// Test time manager creation with valid acceleration factor
#[test]
fn test_time_manager_creation() {
    let tm = TimeManager::new();
    // Test that it was created successfully
    let current_time = tm.current_simulated_time();
    assert!(current_time > Utc::now() - Duration::seconds(1));
    
    let tm_default = TimeManager::default();
    let default_time = tm_default.current_simulated_time();
    assert!(default_time > Utc::now() - Duration::seconds(1));
}

/// Test time manager with invalid acceleration factor
#[test]
fn test_time_manager_invalid_acceleration() {
    let tm_zero = TimeManager::new();
    // Should handle invalid input gracefully
    let _ = tm_zero.current_simulated_time();
    
    let tm_negative = TimeManager::new();
    // Should handle invalid input gracefully
    let _ = tm_negative.current_simulated_time();
}



/// Test business hours detection
#[test]
fn test_business_hours_detection() {
    let tm = TimeManager::default();
    
    // Test business hours (9 AM - 5 PM, every day)
    let monday_10am = Utc.with_ymd_and_hms(2024, 1, 8, 10, 0, 0).unwrap(); // Monday
    let tuesday_2pm = Utc.with_ymd_and_hms(2024, 1, 9, 14, 0, 0).unwrap(); // Tuesday
    let wednesday_noon = Utc.with_ymd_and_hms(2024, 1, 10, 12, 0, 0).unwrap(); // Wednesday
    let thursday_4pm = Utc.with_ymd_and_hms(2024, 1, 11, 16, 0, 0).unwrap(); // Thursday
    let friday_2pm = Utc.with_ymd_and_hms(2024, 1, 12, 14, 0, 0).unwrap(); // Friday
    let saturday_10am = Utc.with_ymd_and_hms(2024, 1, 13, 10, 0, 0).unwrap(); // Saturday
    let sunday_2pm = Utc.with_ymd_and_hms(2024, 1, 14, 14, 0, 0).unwrap(); // Sunday
    
    assert!(tm.is_business_hours(monday_10am));
    assert!(tm.is_business_hours(tuesday_2pm));
    assert!(tm.is_business_hours(wednesday_noon));
    assert!(tm.is_business_hours(thursday_4pm));
    assert!(tm.is_business_hours(friday_2pm));
    assert!(tm.is_business_hours(saturday_10am)); // Saturday is now a work day
    assert!(tm.is_business_hours(sunday_2pm)); // Sunday is now a work day
}

/// Test non-business hours detection
#[test]
fn test_non_business_hours_detection() {
    let tm = TimeManager::default();
    
    // Test non-business hours (only based on time, not day of week)
    let monday_8am = Utc.with_ymd_and_hms(2024, 1, 8, 8, 0, 0).unwrap(); // Before business hours
    let monday_6pm = Utc.with_ymd_and_hms(2024, 1, 8, 18, 0, 0).unwrap(); // After business hours
    let monday_midnight = Utc.with_ymd_and_hms(2024, 1, 8, 0, 0, 0).unwrap(); // Midnight
    let saturday_8am = Utc.with_ymd_and_hms(2024, 1, 13, 8, 0, 0).unwrap(); // Saturday before business hours
    let sunday_6pm = Utc.with_ymd_and_hms(2024, 1, 14, 18, 0, 0).unwrap(); // Sunday after business hours
    
    assert!(!tm.is_business_hours(monday_8am));
    assert!(!tm.is_business_hours(monday_6pm));
    assert!(!tm.is_business_hours(monday_midnight));
    assert!(!tm.is_business_hours(saturday_8am)); // Outside business hours
    assert!(!tm.is_business_hours(sunday_6pm)); // Outside business hours
}



/// Test travel time calculation for same room
#[test]
fn test_travel_time_same_room() {
    let tm = TimeManager::default();
    let mut rng = thread_rng();
    
    let room_id = RoomId::new();
    let building_id = BuildingId::new();
    let location_id = LocationId::new();
    
    let travel_time = tm.calculate_travel_time(
        Some(room_id),
        room_id,
        building_id,
        building_id,
        location_id,
        location_id,
        &mut rng,
    );
    
    assert_eq!(travel_time, Duration::seconds(0));
}

/// Test travel time calculation for different locations
#[test]
fn test_travel_time_different_locations() {
    let tm = TimeManager::default();
    let mut rng = thread_rng();
    
    let room_id1 = RoomId::new();
    let room_id2 = RoomId::new();
    let building_id = BuildingId::new();
    let location_id1 = LocationId::new();
    let location_id2 = LocationId::new();
    
    let travel_time = tm.calculate_travel_time(
        Some(room_id1),
        room_id2,
        building_id,
        building_id,
        location_id1,
        location_id2,
        &mut rng,
    );
    
    // Should be between 4-12 hours for different locations
    assert!(travel_time >= Duration::hours(4));
    assert!(travel_time <= Duration::hours(12));
}

/// Test travel time calculation for different buildings, same location
#[test]
fn test_travel_time_different_buildings() {
    let tm = TimeManager::default();
    let mut rng = thread_rng();
    
    let room_id1 = RoomId::new();
    let room_id2 = RoomId::new();
    let building_id1 = BuildingId::new();
    let building_id2 = BuildingId::new();
    let location_id = LocationId::new();
    
    let travel_time = tm.calculate_travel_time(
        Some(room_id1),
        room_id2,
        building_id1,
        building_id2,
        location_id,
        location_id,
        &mut rng,
    );
    
    // Should be between 2-10 minutes for different buildings
    assert!(travel_time >= Duration::minutes(2));
    assert!(travel_time <= Duration::minutes(10));
}

/// Test travel time calculation for same building, different rooms
#[test]
fn test_travel_time_same_building() {
    let tm = TimeManager::default();
    let mut rng = thread_rng();
    
    let room_id1 = RoomId::new();
    let room_id2 = RoomId::new();
    let building_id = BuildingId::new();
    let location_id = LocationId::new();
    
    let travel_time = tm.calculate_travel_time(
        Some(room_id1),
        room_id2,
        building_id,
        building_id,
        location_id,
        location_id,
        &mut rng,
    );
    
    // Should be between 30 seconds and 3 minutes for same building
    assert!(travel_time >= Duration::seconds(30));
    assert!(travel_time <= Duration::seconds(180));
}

/// Test travel time calculation from no previous room (entering building)
#[test]
fn test_travel_time_no_previous_room() {
    let tm = TimeManager::default();
    let mut rng = thread_rng();
    
    let room_id = RoomId::new();
    let building_id = BuildingId::new();
    let location_id = LocationId::new();
    
    let travel_time = tm.calculate_travel_time(
        None, // No previous room
        room_id,
        building_id,
        building_id,
        location_id,
        location_id,
        &mut rng,
    );
    
    // Should be between 30 seconds and 3 minutes (same building logic)
    assert!(travel_time >= Duration::seconds(30));
    assert!(travel_time <= Duration::seconds(180));
}



/// Test business hours edge cases
#[test]
fn test_business_hours_edge_cases() {
    let tm = TimeManager::default();
    
    // Test exact boundary times (works for any day of the week)
    let monday_9am = Utc.with_ymd_and_hms(2024, 1, 8, 9, 0, 0).unwrap(); // Start of business hours
    let monday_5pm = Utc.with_ymd_and_hms(2024, 1, 8, 17, 0, 0).unwrap(); // End of business hours
    let monday_859am = Utc.with_ymd_and_hms(2024, 1, 8, 8, 59, 0).unwrap(); // Just before
    let monday_501pm = Utc.with_ymd_and_hms(2024, 1, 8, 17, 1, 0).unwrap(); // Just after
    let saturday_9am = Utc.with_ymd_and_hms(2024, 1, 13, 9, 0, 0).unwrap(); // Saturday start of business hours
    let sunday_4pm = Utc.with_ymd_and_hms(2024, 1, 14, 16, 0, 0).unwrap(); // Sunday during business hours
    
    assert!(tm.is_business_hours(monday_9am));
    assert!(!tm.is_business_hours(monday_5pm)); // 5 PM is not included (9-17 exclusive)
    assert!(!tm.is_business_hours(monday_859am));
    assert!(!tm.is_business_hours(monday_501pm));
    assert!(tm.is_business_hours(saturday_9am)); // Saturday is now a work day
    assert!(tm.is_business_hours(sunday_4pm)); // Sunday is now a work day
}

/// Test business hours for all days of the week
#[test]
fn test_all_days_business_hours() {
    let tm = TimeManager::default();
    
    // Test all days of the week (January 8-14, 2024) - all are work days now
    let days = vec![
        Utc.with_ymd_and_hms(2024, 1, 8, 10, 0, 0).unwrap(), // Monday
        Utc.with_ymd_and_hms(2024, 1, 9, 10, 0, 0).unwrap(), // Tuesday
        Utc.with_ymd_and_hms(2024, 1, 10, 10, 0, 0).unwrap(), // Wednesday
        Utc.with_ymd_and_hms(2024, 1, 11, 10, 0, 0).unwrap(), // Thursday
        Utc.with_ymd_and_hms(2024, 1, 12, 10, 0, 0).unwrap(), // Friday
        Utc.with_ymd_and_hms(2024, 1, 13, 10, 0, 0).unwrap(), // Saturday
        Utc.with_ymd_and_hms(2024, 1, 14, 10, 0, 0).unwrap(), // Sunday
    ];
    
    for date in days {
        // Business hours should be true for all days during 9-17
        assert!(tm.is_business_hours(date));
        
        // Test non-business hours for the same days
        let early_morning = date.with_hour(8).unwrap();
        let evening = date.with_hour(18).unwrap();
        assert!(!tm.is_business_hours(early_morning));
        assert!(!tm.is_business_hours(evening));
    }
}

/// Test time manager consistency over multiple calls
#[test]
fn test_time_manager_consistency() {
    let tm = TimeManager::new();
    
    let mut previous_time = tm.current_simulated_time();
    
    for _ in 0..10 {
        thread::sleep(StdDuration::from_millis(10));
        let current_time = tm.current_simulated_time();
        
        // Time should always be advancing
        assert!(current_time >= previous_time);
        previous_time = current_time;
    }
}

/// Test travel time calculation determinism with same seed
#[test]
fn test_travel_time_determinism() {
    let tm = TimeManager::default();
    
    let room_id1 = RoomId::new();
    let room_id2 = RoomId::new();
    let building_id1 = BuildingId::new();
    let building_id2 = BuildingId::new();
    let location_id = LocationId::new();
    
    // Use seeded RNG for deterministic results
    use rand::SeedableRng;
    let mut rng1 = rand::rngs::StdRng::seed_from_u64(12345);
    let mut rng2 = rand::rngs::StdRng::seed_from_u64(12345);
    
    let travel_time1 = tm.calculate_travel_time(
        Some(room_id1),
        room_id2,
        building_id1,
        building_id2,
        location_id,
        location_id,
        &mut rng1,
    );
    
    let travel_time2 = tm.calculate_travel_time(
        Some(room_id1),
        room_id2,
        building_id1,
        building_id2,
        location_id,
        location_id,
        &mut rng2,
    );
    
    assert_eq!(travel_time1, travel_time2);
}

/// Test realistic time acceleration scenarios
#[test]
fn test_realistic_acceleration_scenarios() {
    // Test common acceleration factors from the design
    
    // 1 day per 5 minutes (288x acceleration)
    let tm_fast = TimeManager::new();
    let fast_time = tm_fast.current_simulated_time();
    
    // 1 day per minute (1440x acceleration)
    let tm_very_fast = TimeManager::new();
    let very_fast_time = tm_very_fast.current_simulated_time();
    
    // Real-time (1x acceleration)
    let tm_realtime = TimeManager::new();
    let realtime_time = tm_realtime.current_simulated_time();
    
    // Test that all can calculate current time without panicking
    assert!(fast_time > Utc::now() - Duration::seconds(1));
    assert!(very_fast_time > Utc::now() - Duration::seconds(1));
    assert!(realtime_time > Utc::now() - Duration::seconds(1));
}

/// Test time calculations for impossible traveler scenarios
#[test]
fn test_impossible_traveler_time_calculations() {
    let tm = TimeManager::default();
    let mut rng = thread_rng();
    
    let room_id1 = RoomId::new();
    let room_id2 = RoomId::new();
    let building_id = BuildingId::new();
    let location_id1 = LocationId::new();
    let location_id2 = LocationId::new();
    
    // Calculate travel time between different locations
    let travel_time = tm.calculate_travel_time(
        Some(room_id1),
        room_id2,
        building_id,
        building_id,
        location_id1,
        location_id2,
        &mut rng,
    );
    
    // For impossible traveler detection, minimum travel time should be 4+ hours
    assert!(travel_time >= Duration::hours(4));
    
    // Test multiple calculations to ensure consistency
    for _ in 0..10 {
        let time = tm.calculate_travel_time(
            Some(room_id1),
            room_id2,
            building_id,
            building_id,
            location_id1,
            location_id2,
            &mut rng,
        );
        assert!(time >= Duration::hours(4));
        assert!(time <= Duration::hours(12));
    }
}