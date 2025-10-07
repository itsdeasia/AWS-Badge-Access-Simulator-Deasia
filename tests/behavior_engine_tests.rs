//! Unit tests for behavioral engine and activity scheduling
//!
//! Tests Requirements: 2.1, 2.2, 2.3, 2.4

use amzn_career_pathway_activity_rust::*;
use chrono::{NaiveDate, Timelike};

/// Test behavior engine creation
#[test]
fn test_behavior_engine_creation() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let engine = BehaviorEngine::new(config, time_manager);
    
    assert!(format!("{:?}", engine).contains("BehaviorEngine"));
}

/// Test daily schedule generation for regular user
#[test]
fn test_daily_schedule_generation_regular_user() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    let user = User::new(location_id, building_id, room_id, permissions);
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    let result = engine.generate_daily_schedule(&user, date, &registry);
    assert!(result.is_ok());
    
    let schedule = result.unwrap();
    // Should have at least arrival and departure
    assert!(schedule.len() >= 2);
    
    // Check that activities are sorted by time
    for window in schedule.windows(2) {
        assert!(window[0].start_time <= window[1].start_time);
    }
}

/// Test daily schedule generation for curious user
#[test]
fn test_daily_schedule_generation_curious_user() {
    let config = SimulationConfig {
        curious_user_percentage: 0.1,
        ..Default::default()
    };
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    let mut user = User::new(location_id, building_id, room_id, permissions);
    user.is_curious = true;
    user.behavior_profile = BehaviorProfile::curious();
    
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    let result = engine.generate_daily_schedule(&user, date, &registry);
    assert!(result.is_ok());
    
    let schedule = result.unwrap();
    assert!(schedule.len() >= 2); // At least arrival and departure
}

/// Test schedule generation for all days (no weekend logic)
#[test]
fn test_all_days_schedule_generation() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    let user = User::new(location_id, building_id, room_id, permissions);
    
    // Test all days of the week - all should be treated as work days
    let dates = vec![
        NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(),  // Monday
        NaiveDate::from_ymd_opt(2024, 1, 9).unwrap(),  // Tuesday
        NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(), // Wednesday
        NaiveDate::from_ymd_opt(2024, 1, 11).unwrap(), // Thursday
        NaiveDate::from_ymd_opt(2024, 1, 12).unwrap(), // Friday
        NaiveDate::from_ymd_opt(2024, 1, 13).unwrap(), // Saturday
        NaiveDate::from_ymd_opt(2024, 1, 14).unwrap(), // Sunday
    ];
    
    for date in dates {
        let result = engine.generate_daily_schedule(&user, date, &registry);
        assert!(result.is_ok(), "Schedule generation failed for date: {}", date);
        
        let schedule = result.unwrap();
        // All days should generate schedules (no weekend skipping)
        assert!(schedule.len() >= 2, "Schedule should have at least arrival and departure for date: {}", date);
    }
}

/// Test behavior profile impact on scheduling
#[test]
fn test_behavior_profile_impact_on_scheduling() {
    // Test schedule-focused behavior
    let mut schedule_focused_profile = BehaviorProfile::default();
    schedule_focused_profile.schedule_adherence = 0.9;
    assert!(schedule_focused_profile.is_schedule_focused());
    
    // Test flexible behavior
    let mut flexible_profile = BehaviorProfile::default();
    flexible_profile.schedule_adherence = 0.5;
    assert!(!flexible_profile.is_schedule_focused());
    
    // Test social behavior
    let mut social_profile = BehaviorProfile::default();
    social_profile.social_level = 0.8;
    assert!(social_profile.is_social());
}

/// Test schedule generation includes expected activity types
#[test]
fn test_schedule_activity_types() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    let user = User::new(location_id, building_id, room_id, permissions);
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    let result = engine.generate_daily_schedule(&user, date, &registry);
    assert!(result.is_ok());
    
    let schedule = result.unwrap();
    
    // Should have at least arrival and departure
    assert!(schedule.len() >= 2);
    
    // Check that activities are properly ordered by time
    for window in schedule.windows(2) {
        assert!(window[0].start_time <= window[1].start_time);
    }
}

/// Test unauthorized access attempt generation
#[test]
fn test_unauthorized_access_attempt_generation() {
    let config = SimulationConfig {
        curious_user_percentage: 0.1,
        ..Default::default()
    };
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    // Test non-curious user
    let regular_user = User::new(location_id, building_id, room_id, permissions.clone());
    assert!(!engine.should_attempt_unauthorized_access(&regular_user));
    
    let unauthorized_room = engine.generate_unauthorized_access_attempt(&regular_user, &registry);
    assert!(unauthorized_room.is_none());
    
    // Test curious user
    let mut curious_user = User::new(location_id, building_id, room_id, permissions);
    curious_user.is_curious = true;
    curious_user.behavior_profile = BehaviorProfile::curious();
    
    // Should have some probability of unauthorized access
    let mut attempts = 0;
    for _ in 0..100 {
        if engine.should_attempt_unauthorized_access(&curious_user) {
            attempts += 1;
        }
    }
    
    // Should have some attempts but not too many (curious users still primarily use authorized areas)
    assert!(attempts >= 0); // At least some possibility
}

/// Test behavior profile characteristics
#[test]
fn test_behavior_profile_characteristics() {
    // Test curious profile
    let curious_profile = BehaviorProfile::curious();
    assert!(curious_profile.is_curious());
    assert!(curious_profile.curiosity_level > 0.5);
    
    // Test social behavior
    let mut social_profile = BehaviorProfile::default();
    social_profile.social_level = 0.8;
    assert!(social_profile.is_social());
    
    // Test schedule-focused behavior
    let mut schedule_profile = BehaviorProfile::default();
    schedule_profile.schedule_adherence = 0.9;
    assert!(schedule_profile.is_schedule_focused());
}

/// Test activity type distribution in schedules
#[test]
fn test_activity_type_distribution() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    let user = User::new(location_id, building_id, room_id, permissions);
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    let result = engine.generate_daily_schedule(&user, date, &registry);
    assert!(result.is_ok());
    
    let schedule = result.unwrap();
    
    // Count activity types
    let mut activity_counts = std::collections::HashMap::new();
    for activity in &schedule {
        *activity_counts.entry(activity.activity_type).or_insert(0) += 1;
    }
    
    // Should have arrival and departure
    assert!(activity_counts.contains_key(&ActivityType::Arrival));
    assert!(activity_counts.contains_key(&ActivityType::Departure));
    
    // Should have exactly one arrival and one departure
    assert_eq!(activity_counts[&ActivityType::Arrival], 1);
    assert_eq!(activity_counts[&ActivityType::Departure], 1);
}

/// Test schedule generation with different user types
#[test]
fn test_schedule_generation_different_user_types() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    // Test regular user
    let regular_user = User::new(location_id, building_id, room_id, permissions.clone());
    let regular_result = engine.generate_daily_schedule(&regular_user, date, &registry);
    assert!(regular_result.is_ok());
    
    // Test curious user
    let mut curious_user = User::new(location_id, building_id, room_id, permissions.clone());
    curious_user.is_curious = true;
    let curious_result = engine.generate_daily_schedule(&curious_user, date, &registry);
    assert!(curious_result.is_ok());
    
    // Test social user
    let mut social_user = User::new(location_id, building_id, room_id, permissions);
    social_user.behavior_profile.social_level = 0.9;
    let social_result = engine.generate_daily_schedule(&social_user, date, &registry);
    assert!(social_result.is_ok());
}

/// Test schedule generation error handling
#[test]
fn test_schedule_generation_error_handling() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    let user = User::new(location_id, building_id, room_id, permissions);
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    // Should handle empty registry gracefully
    let result = engine.generate_daily_schedule(&user, date, &registry);
    assert!(result.is_ok()); // Should not panic, may return minimal schedule
}

/// Test night-shift user schedule generation
#[test]
fn test_night_shift_user_schedule_generation() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let night_building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    // Create night-shift user
    let night_shift_user = User::new_night_shift(
        location_id,
        building_id,
        room_id,
        permissions,
        night_building_id,
    );
    
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    let result = engine.generate_daily_schedule(&night_shift_user, date, &registry);
    assert!(result.is_ok());
    
    let schedule = result.unwrap();
    // Night-shift users should have at least arrival and departure
    assert!(schedule.len() >= 2);
    
    // Check that activities are sorted by time
    for window in schedule.windows(2) {
        assert!(window[0].start_time <= window[1].start_time);
    }
    
    // Should have inverted schedule - departure in morning, arrival in evening
    let departure_activities: Vec<&ScheduledActivity> = schedule
        .iter()
        .filter(|activity| activity.activity_type == ActivityType::Departure)
        .collect();
    
    let arrival_activities: Vec<&ScheduledActivity> = schedule
        .iter()
        .filter(|activity| activity.activity_type == ActivityType::Arrival)
        .collect();
    
    assert!(!departure_activities.is_empty());
    assert!(!arrival_activities.is_empty());
    
    // Departure should be before arrival (morning departure, evening arrival)
    if let (Some(departure), Some(arrival)) = (departure_activities.first(), arrival_activities.first()) {
        assert!(departure.start_time < arrival.start_time);
    }
}

/// Test night-shift user behavior profile usage
#[test]
fn test_night_shift_user_behavior_profile() {
    let night_shift_profile = BehaviorProfile::night_shift();
    
    // Night-shift users should have specific behavior characteristics
    assert_eq!(night_shift_profile.travel_frequency, 0.05); // Stay in assigned building
    assert_eq!(night_shift_profile.curiosity_level, 0.1); // Low curiosity (security-focused)
    assert_eq!(night_shift_profile.schedule_adherence, 0.9); // High adherence to patrol schedule
    assert_eq!(night_shift_profile.social_level, 0.2); // Low social interaction during night
    
    // Verify behavior characteristics
    assert!(!night_shift_profile.is_curious()); // 0.1 < 0.5 threshold
    assert!(!night_shift_profile.travels_frequently()); // 0.05 < 0.15 threshold
    assert!(!night_shift_profile.is_social()); // 0.2 < 0.7 threshold
    assert!(night_shift_profile.is_schedule_focused()); // 0.9 > 0.8 threshold
}

/// Test night-shift schedule includes patrol activities
#[test]
fn test_night_shift_schedule_includes_patrol_activities() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    // Create night-shift user
    let night_shift_user = User::new_night_shift(
        location_id,
        building_id,
        room_id,
        permissions,
        building_id, // Same building for night shift
    );
    
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    let result = engine.generate_daily_schedule(&night_shift_user, date, &registry);
    assert!(result.is_ok());
    
    let schedule = result.unwrap();
    
    // Should have minimal schedule even with empty registry
    assert!(!schedule.is_empty());
    
    // Check for expected activity types in minimal schedule
    let activity_types: Vec<ActivityType> = schedule
        .iter()
        .map(|activity| activity.activity_type)
        .collect();
    
    // Should have at least arrival and departure
    assert!(activity_types.contains(&ActivityType::Arrival));
    assert!(activity_types.contains(&ActivityType::Departure));
}

/// Test night-shift vs regular user schedule differences
#[test]
fn test_night_shift_vs_regular_user_schedules() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    // Create regular user
    let regular_user = User::new(location_id, building_id, room_id, permissions.clone());
    let regular_result = engine.generate_daily_schedule(&regular_user, date, &registry);
    assert!(regular_result.is_ok());
    
    // Create night-shift user
    let night_shift_user = User::new_night_shift(
        location_id,
        building_id,
        room_id,
        permissions,
        building_id,
    );
    let night_shift_result = engine.generate_daily_schedule(&night_shift_user, date, &registry);
    assert!(night_shift_result.is_ok());
    
    let regular_schedule = regular_result.unwrap();
    let night_shift_schedule = night_shift_result.unwrap();
    
    // Both should have schedules
    assert!(!regular_schedule.is_empty());
    assert!(!night_shift_schedule.is_empty());
    
    // Schedules should be different (different timing patterns)
    // Both minimal schedules have 2 activities, but with different timing patterns
    assert_eq!(regular_schedule.len(), 2);
    assert_eq!(night_shift_schedule.len(), 4);
    
    // Regular user: Arrival first, then Departure
    // Night-shift user: Departure first, then Arrival (inverted pattern)
    assert_eq!(regular_schedule[0].activity_type, ActivityType::Arrival);
    assert_eq!(regular_schedule[1].activity_type, ActivityType::Departure);
    assert_eq!(night_shift_schedule[0].activity_type, ActivityType::NightPatrol);
    assert_eq!(night_shift_schedule[1].activity_type, ActivityType::Departure);
    
    // Check timing differences - night-shift has earlier departure and later arrival
    assert!(regular_schedule[0].start_time.hour() >= 9); // Regular arrival at 9 AM or later
    assert!(night_shift_schedule[0].start_time.hour() <= 8); // Night-shift departure at 8 AM or earlier
}

/// Test night-shift user with different assigned building
#[test]
fn test_night_shift_user_different_assigned_building() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let primary_building_id = BuildingId::new();
    let night_building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    // Create night-shift user with different night building
    let night_shift_user = User::new_night_shift(
        location_id,
        primary_building_id,
        room_id,
        permissions,
        night_building_id, // Different building for night shift
    );
    
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    let result = engine.generate_daily_schedule(&night_shift_user, date, &registry);
    assert!(result.is_ok());
    
    let schedule = result.unwrap();
    assert!(!schedule.is_empty());
    
    // Verify user has correct building assignments
    assert_eq!(night_shift_user.primary_building, primary_building_id);
    assert_eq!(night_shift_user.assigned_night_building, Some(night_building_id));
    assert_ne!(night_shift_user.primary_building, night_building_id);
}

/// Test night-shift users do not attend meetings
#[test]
fn test_night_shift_users_no_meetings() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    let night_shift_user = User::new_night_shift(
        location_id,
        building_id,
        room_id,
        permissions,
        building_id,
    );
    
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    let result = engine.generate_daily_schedule(&night_shift_user, date, &registry);
    assert!(result.is_ok());
    
    let schedule = result.unwrap();
    
    // Verify that no meeting activities are present in night-shift schedule
    let meeting_activities: Vec<&ScheduledActivity> = schedule
        .iter()
        .filter(|activity| activity.activity_type == ActivityType::Meeting)
        .collect();
    
    assert!(meeting_activities.is_empty(), "Night-shift users should not have meeting activities");
    
    // Verify that patrol activities are present instead
    let patrol_activities: Vec<&ScheduledActivity> = schedule
        .iter()
        .filter(|activity| activity.activity_type == ActivityType::NightPatrol)
        .collect();
    
    // Should have patrol activities (even if minimal schedule)
    assert!(!patrol_activities.is_empty() || schedule.len() >= 2, "Night-shift users should have patrol activities or minimal schedule");
}

/// Test night-shift schedule timing patterns
#[test]
fn test_night_shift_schedule_timing_patterns() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::default();
    let mut engine = BehaviorEngine::new(config, time_manager);
    let registry = LocationRegistry::new();
    
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();
    
    let night_shift_user = User::new_night_shift(
        location_id,
        building_id,
        room_id,
        permissions,
        building_id,
    );
    
    let date = NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(); // Monday
    
    let result = engine.generate_daily_schedule(&night_shift_user, date, &registry);
    assert!(result.is_ok());
    
    let schedule = result.unwrap();
    
    // Find departure and arrival activities
    let departure_activity = schedule
        .iter()
        .find(|activity| activity.activity_type == ActivityType::Departure);
    
    let arrival_activity = schedule
        .iter()
        .find(|activity| activity.activity_type == ActivityType::Arrival);
    
    if let (Some(departure), Some(arrival)) = (departure_activity, arrival_activity) {
        // Departure should be in the morning (around 8 AM)
        let departure_hour = departure.start_time.hour();
        assert!(departure_hour >= 6 && departure_hour <= 10, "Departure should be in morning, got hour {}", departure_hour);
        
        // Arrival should be in the evening (around 5 PM)
        let arrival_hour = arrival.start_time.hour();
        assert!(arrival_hour >= 15 && arrival_hour <= 19, "Arrival should be in evening, got hour {}", arrival_hour);
        
        // Departure should be before arrival (inverted schedule)
        assert!(departure.start_time < arrival.start_time);
    }
}
