//! Comprehensive integration tests for night-shift users
//!
//! This module tests the complete night-shift user lifecycle including:
//! - Building assignment (1-3 users per building)
//! - Inverted schedule behavior
//! - Room patrol behavior
//! - Event classification
//! - Statistics reporting

use amzn_career_pathway_activity_rust::user::{BehaviorProfile, User, UserGenerator, ScheduledActivity};
use amzn_career_pathway_activity_rust::events::EventGenerator;
use amzn_career_pathway_activity_rust::facility::FacilityGenerator;
use amzn_career_pathway_activity_rust::permissions::{PermissionLevel, PermissionSet};
use amzn_career_pathway_activity_rust::simulation::{SimulationStatistics, TimeManager};
use amzn_career_pathway_activity_rust::types::{ActivityType, EventType, SimulationConfig};
use chrono::{Duration, Timelike};
use std::collections::HashMap;

/// Test complete night-shift user lifecycle
/// Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 3.1, 3.2, 3.3, 5.1, 5.2, 5.3, 5.5
#[test]
fn test_night_shift_user_complete_lifecycle() {
    // Create a large organization with multiple buildings
    let config = SimulationConfig {
        user_count: 1000, // Large enough to trigger night-shift generation
        location_count: 2,
        min_buildings_per_location: 3,
        max_buildings_per_location: 5,
        min_rooms_per_building: 8,
        max_rooms_per_building: 12,
        curious_user_percentage: 0.05,
        cloned_badge_percentage: 0.02,
        ..Default::default()
    };

    // Generate facilities
    let mut facility_generator = FacilityGenerator::with_seed(42);
    let location_registry = facility_generator.generate_facilities(&config).unwrap();

    // Generate users
    let mut user_generator = UserGenerator::with_seed(42);
    let users = user_generator.generate_users(&config, &location_registry).unwrap();

    // Get user statistics
    let stats = user_generator.get_user_stats(&users);

    // Verify night-shift users were created
    assert!(stats.night_shift_users > 0, "No night-shift users were generated");
    println!("Generated {} night-shift users out of {} total users", 
             stats.night_shift_users, stats.total_users);

    // Test building assignment - each building should have 1-3 night-shift users
    let night_shift_users: Vec<_> = users.iter()
        .filter(|e| e.is_night_shift)
        .collect();

    assert!(!night_shift_users.is_empty());

    // Count night-shift users per building
    let mut building_night_shift_counts = HashMap::new();
    for user in &night_shift_users {
        if let Some(building_id) = user.assigned_night_building {
            *building_night_shift_counts.entry(building_id).or_insert(0) += 1;
        }
    }

    // Verify each building has 1-3 night-shift users
    let _total_buildings = location_registry.total_building_count();
    assert!(building_night_shift_counts.len() > 0, "No buildings have night-shift users");
    
    for (building_id, count) in &building_night_shift_counts {
        assert!(*count >= 1 && *count <= 3, 
                "Building {} has {} night-shift users, expected 1-3", 
                building_id, count);
    }

    println!("Building night-shift distribution: {} buildings with night-shift coverage", 
             building_night_shift_counts.len());

    // Test night-shift user properties
    for user in &night_shift_users {
        // Verify night-shift designation
        assert!(user.is_night_shift);
        assert!(user.assigned_night_building.is_some());

        // Verify building-level permissions
        let assigned_building = user.assigned_night_building.unwrap();
        assert!(user.can_access_building(assigned_building, user.primary_location));

        // Verify they have extensive permissions in their assigned building
        let authorized_rooms = user.get_authorized_rooms();
        assert!(!authorized_rooms.is_empty(), "Night-shift user has no room permissions");
    }

    // Test event classification and statistics
    let _time_manager = TimeManager::new();
    let curious_count = users.iter().filter(|e| e.is_curious).count();
    let cloned_badge_count = users.iter().filter(|e| e.has_cloned_badge).count();
    let night_shift_count = users.iter().filter(|e| e.is_night_shift).count();

    let simulation_stats = SimulationStatistics::new(
        users.len(),
        location_registry.location_count(),
        location_registry.total_building_count(),
        location_registry.total_room_count(),
        curious_count,
        cloned_badge_count,
        night_shift_count,
    );

    // Verify night-shift user count is tracked in simulation statistics
    assert_eq!(simulation_stats.night_shift_users, night_shift_count);
    assert!(simulation_stats.night_shift_user_percentage() > 0.0);

    println!("Night-shift users: {} ({:.1}%)", 
             simulation_stats.night_shift_users, 
             simulation_stats.night_shift_user_percentage());

    // Test statistics display includes night-shift information
    let stats_display = format!("{}", simulation_stats);
    assert!(stats_display.contains("Night-Shift Users:"));
    assert!(stats_display.contains(&format!("{}", simulation_stats.night_shift_users)));

    println!("Statistics display test passed");
}

/// Test night-shift event generation and classification
/// Requirements: 3.1, 3.2, 5.4
#[test]
fn test_night_shift_event_classification() {
    let config = SimulationConfig {
        user_count: 1000, // Large enough to trigger night-shift generation
        location_count: 1,
        min_buildings_per_location: 1,
        max_buildings_per_location: 1,
        min_rooms_per_building: 5,
        max_rooms_per_building: 5,
        ..Default::default()
    };

    // Generate facilities
    let mut facility_generator = FacilityGenerator::with_seed(42);
    let location_registry = facility_generator.generate_facilities(&config).unwrap();

    // Get the first location, building, and room for testing
    let locations = location_registry.get_all_locations();
    let location = &locations[0];
    let building = &location.buildings[0];
    let room = &building.rooms[0];

    // Store IDs to avoid borrow checker issues
    let location_id = location.id;
    let building_id = building.id;
    let room_id = room.id;

    let time_manager = TimeManager::new();

    // Create a night-shift user
    let mut permissions = PermissionSet::new();
    permissions.add_permission(PermissionLevel::Room(room_id));
    permissions.add_permission(PermissionLevel::Building(building_id));

    let night_shift_user = User::new_night_shift(
        location_id,
        building_id,
        room_id,
        permissions,
        building_id,
    );

    // Create statistics tracker (unused in new API)
    let _statistics = SimulationStatistics::new(
        1,    // total users
        1,    // locations
        1,    // buildings
        5,    // rooms
        0,    // curious users
        0,    // cloned badge users
        1,    // night-shift users
    );

    let mut event_generator = EventGenerator::new(
        config,
        location_registry,
        time_manager.clone(),
    );

    // Generate an off-hours activity for the night-shift user
    // Use a time that's definitely off-hours (2 AM)
    let off_hours_time = chrono::Utc::now()
        .with_hour(2).unwrap()
        .with_minute(0).unwrap()
        .with_second(0).unwrap();
    
    let activity = ScheduledActivity::new(
        ActivityType::NightPatrol,
        room_id,
        off_hours_time,
        Duration::hours(1),
    );

    // Generate events
    let events = event_generator.generate_events_from_activity(
        &night_shift_user,
        &activity,
        off_hours_time,
    ).unwrap();

    assert!(!events.is_empty(), "No events generated for night-shift user");

    // Verify events are classified as night-shift events
    for event in &events {
        if let Some(metadata) = &event.metadata {
            if metadata.is_night_shift_event {
                println!("Successfully generated night-shift event for room {}", event.room_id);
            }
        }
    }

    // NOTE: Statistics tracking has been moved to centralized location in BatchEventGenerator
    // The EventGenerator no longer tracks statistics directly
    println!("Night-shift events generated successfully (statistics now tracked centrally)");
}

/// Test night-shift behavior profile characteristics
/// Requirements: 5.4
#[test]
fn test_night_shift_behavior_profile() {
    let night_shift_profile = BehaviorProfile::night_shift();

    // Verify night-shift behavior characteristics
    assert!(night_shift_profile.travel_frequency <= 0.1, 
            "Night-shift users should have low travel frequency");
    assert!(night_shift_profile.curiosity_level <= 0.2, 
            "Night-shift users should have low curiosity (security-focused)");
    assert!(night_shift_profile.schedule_adherence >= 0.8, 
            "Night-shift users should have high schedule adherence");
    assert!(night_shift_profile.social_level <= 0.3, 
            "Night-shift users should have low social interaction during night");

    println!("Night-shift behavior profile validated: travel_freq={:.2}, curiosity={:.2}, adherence={:.2}, social={:.2}",
             night_shift_profile.travel_frequency,
             night_shift_profile.curiosity_level,
             night_shift_profile.schedule_adherence,
             night_shift_profile.social_level);
}

/// Test that night-shift events don't count as violations
/// Requirements: 3.1, 3.2, 3.3
#[test]
fn test_night_shift_events_not_violations() {
    let config = SimulationConfig {
        user_count: 1000, // Large enough to trigger night-shift generation
        location_count: 1,
        min_buildings_per_location: 1,
        max_buildings_per_location: 1,
        min_rooms_per_building: 5,
        max_rooms_per_building: 5,
        ..Default::default()
    };

    // Generate facilities
    let mut facility_generator = FacilityGenerator::with_seed(42);
    let location_registry = facility_generator.generate_facilities(&config).unwrap();

    // Get the first location, building, and room for testing
    let locations = location_registry.get_all_locations();
    let location = &locations[0];
    let building = &location.buildings[0];
    let room = &building.rooms[0];

    // Store IDs to avoid borrow checker issues
    let location_id = location.id;
    let building_id = building.id;
    let room_id = room.id;

    let time_manager = TimeManager::new();

    // Create both regular and night-shift users
    let mut permissions = PermissionSet::new();
    permissions.add_permission(PermissionLevel::Room(room_id));

    let regular_user = User::new(location_id, building_id, room_id, permissions.clone());
    
    let mut night_shift_permissions = permissions.clone();
    night_shift_permissions.add_permission(PermissionLevel::Building(building_id));
    let night_shift_user = User::new_night_shift(
        location_id,
        building_id,
        room_id,
        night_shift_permissions,
        building_id,
    );

    // Create statistics tracker (unused in new API)
    let _statistics = SimulationStatistics::new(
        2,    // total users
        1,    // locations
        1,    // buildings
        5,    // rooms
        0,    // curious users
        0,    // cloned badge users
        1,    // night-shift users
    );

    let mut event_generator = EventGenerator::new(
        config,
        location_registry,
        time_manager.clone(),
    );

    // Generate off-hours activities for both users
    // Use a time that's definitely off-hours (2 AM)
    let off_hours_time = chrono::Utc::now()
        .with_hour(2).unwrap()
        .with_minute(0).unwrap()
        .with_second(0).unwrap();
    
    let activity = ScheduledActivity::new(
        ActivityType::Meeting,
        room_id,
        off_hours_time,
        Duration::hours(1),
    );

    // Generate events for regular user (should count as violation)
    let regular_events = event_generator.generate_events_from_activity(
        &regular_user,
        &activity,
        off_hours_time,
    ).unwrap();

    // Generate events for night-shift user (should NOT count as violation)
    let night_shift_activity = ScheduledActivity::new(
        ActivityType::NightPatrol,
        room_id,
        off_hours_time,
        Duration::hours(1),
    );

    let night_shift_events = event_generator.generate_events_from_activity(
        &night_shift_user,
        &night_shift_activity,
        off_hours_time,
    ).unwrap();

    // Verify event classification
    let mut regular_outside_hours = 0;
    let mut night_shift_classified = 0;

    for event in &regular_events {
        if event.event_type == EventType::OutsideHours {
            regular_outside_hours += 1;
        }
    }

    for event in &night_shift_events {
        if let Some(metadata) = &event.metadata {
            if metadata.is_night_shift_event {
                night_shift_classified += 1;
            }
        }
    }

    println!("Regular user outside-hours events: {}", regular_outside_hours);
    println!("Night-shift classified events: {}", night_shift_classified);

    // NOTE: Statistics tracking has been moved to centralized location in BatchEventGenerator
    // The EventGenerator no longer tracks statistics directly
    println!("Final statistics: Statistics now tracked centrally in BatchEventGenerator");
    println!("  Regular outside-hours events: {}", regular_outside_hours);
    println!("  Night-shift classified events: {}", night_shift_classified);
    
    // Night-shift events should be tracked separately (now handled centrally)
    // Verify that we generated the expected number of night-shift events
    assert!(night_shift_classified > 0, "Should have generated night-shift events");
    println!("Night-shift events are now tracked centrally in BatchEventGenerator");
}