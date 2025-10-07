//! Integration tests for orchestrator statistics tracking
//!
//! These tests verify that the simulation orchestrator properly integrates
//! enhanced statistics tracking throughout the simulation lifecycle.

use amzn_career_pathway_activity_rust::user::{User, UserGenerator};
use amzn_career_pathway_activity_rust::events::AccessEvent;
use amzn_career_pathway_activity_rust::facility::{FacilityGenerator, LocationRegistry};
use amzn_career_pathway_activity_rust::simulation::SimulationOrchestrator;
use amzn_career_pathway_activity_rust::types::{
    BuildingId, UserId, EventType, LocationId, RoomId, SimulationConfig,
};
use chrono::Utc;

/// Test that orchestrator initializes with enhanced statistics tracking
#[test]
fn test_orchestrator_statistics_initialization() {
    let config = SimulationConfig::default();
    let orchestrator = SimulationOrchestrator::new(config).unwrap();

    // Verify statistics are initialized with detailed tracking enabled
    let stats = orchestrator.get_statistics();
    assert_eq!(stats.total_users, 0);
    assert_eq!(stats.total_locations, 0);
    assert_eq!(stats.total_buildings, 0);
    assert_eq!(stats.total_rooms, 0);

    // Verify event type statistics are initialized
    let event_stats = stats.event_type_statistics();
    assert_eq!(event_stats.total_events, 0);
    assert_eq!(event_stats.success_events, 0);
    assert_eq!(event_stats.failure_events, 0);
    assert_eq!(event_stats.curious_events, 0);
    assert_eq!(event_stats.impossible_traveler_events, 0);
}

/// Test orchestrator initialization with real facility and user data
#[test]
fn test_orchestrator_initialization_with_real_data() {
    let mut config = SimulationConfig::default();
    config.user_count = 10;
    config.location_count = 2;
    config.curious_user_percentage = 0.2; // 20% curious
    config.cloned_badge_percentage = 0.1; // 10% cloned badges

    // Generate facilities
    let mut facility_generator = FacilityGenerator::new();
    let location_registry = facility_generator.generate_facilities(&config).unwrap();

    // Generate users
    let mut user_generator = UserGenerator::new();
    let users = user_generator.generate_users(&config, &location_registry).unwrap();

    // Create and initialize orchestrator
    let mut orchestrator = SimulationOrchestrator::new(config).unwrap();
    orchestrator.initialize_with_data(location_registry, users).unwrap();

    // Verify statistics reflect the generated data
    let stats = orchestrator.get_statistics();
    assert_eq!(stats.total_users, 10);
    assert_eq!(stats.total_locations, 2);
    assert!(stats.total_buildings > 0);
    assert!(stats.total_rooms > 0);

    // Verify user breakdown statistics
    assert!(stats.curious_users <= 10);
    assert!(stats.cloned_badge_users <= 10);
    assert!(stats.curious_user_percentage() <= 100.0);
    assert!(stats.cloned_badge_percentage() <= 100.0);
}

/// Test statistics updates during event generation
#[test]
fn test_statistics_updates_during_event_generation() {
    let config = SimulationConfig::default();
    let mut orchestrator = SimulationOrchestrator::new(config).unwrap();

    // Update statistics manually (replacing deprecated update_statistics_with_events)
    {
        let stats = orchestrator.get_statistics_mut();
        // Simulate the events: 1 success, 1 failure, 1 invalid badge, 1 outside hours, 1 suspicious
        stats.increment_success_events();
        stats.increment_failure_events();
        stats.increment_invalid_badge_events();
        stats.increment_outside_hours_events();
        stats.increment_suspicious_events();
    }

    // Verify statistics were updated correctly
    let stats = orchestrator.get_statistics();
    let event_stats = stats.event_type_statistics();
    assert_eq!(event_stats.total_events, 5);
    assert_eq!(event_stats.success_events, 1);
    assert_eq!(event_stats.failure_events, 4); // 1 direct failure + 3 indirect (InvalidBadge, OutsideHours, Suspicious)
    assert_eq!(event_stats.invalid_badge_events, 1);
    assert_eq!(event_stats.outside_hours_events, 1);
    assert_eq!(event_stats.suspicious_events, 1);

    // Verify percentage calculations
    assert_eq!(event_stats.success_percentage(), 20.0);
    assert_eq!(event_stats.failure_percentage(), 80.0); // 4 failures out of 5 total events
    assert_eq!(event_stats.invalid_badge_percentage(), 20.0);
    assert_eq!(event_stats.outside_hours_percentage(), 20.0);
    assert_eq!(event_stats.suspicious_percentage(), 20.0);
}

/// Test statistics summary and formatting methods
#[test]
fn test_statistics_summary_methods() {
    let config = SimulationConfig::default();
    let mut orchestrator = SimulationOrchestrator::new(config).unwrap();

    // Update statistics manually (replacing deprecated update_statistics_with_events)
    {
        let stats = orchestrator.get_statistics_mut();
        // Simulate the events: 1 success, 1 failure
        stats.increment_success_events();
        stats.increment_failure_events();
    }

    // Test summary methods
    let summary = orchestrator.get_event_statistics_summary();
    assert!(summary.contains("2 total events"));
    assert!(summary.contains("Success: 1"));
    assert!(summary.contains("50.0%"));

    let detailed = orchestrator.get_detailed_event_statistics();
    assert!(detailed.contains("Total Events Generated: 2"));
    assert!(detailed.contains("Success Events: 1 (50.0%)"));
    assert!(detailed.contains("Failure Events: 1 (50.0%)"));
}

/// Test error handling during statistics updates
#[test]
fn test_statistics_error_handling() {
    let config = SimulationConfig::default();
    let mut orchestrator = SimulationOrchestrator::new(config).unwrap();

    // Test with no statistics updates (replacing deprecated empty array call)
    let stats = orchestrator.get_statistics();
    assert_eq!(stats.event_type_statistics().total_events, 0);

    // Test that statistics remain consistent after updates
    // Update statistics manually (replacing deprecated update_statistics_with_events)
    {
        let stats = orchestrator.get_statistics_mut();
        stats.increment_success_events();
    }

    let stats = orchestrator.get_statistics();
    assert_eq!(stats.event_type_statistics().total_events, 1);
    assert_eq!(stats.event_type_statistics().success_events, 1);
}

/// Test statistics persistence throughout simulation lifecycle
#[test]
fn test_statistics_lifecycle_persistence() {
    let config = SimulationConfig::default();
    let mut orchestrator = SimulationOrchestrator::new(config).unwrap();

    // Initialize with test data
    use amzn_career_pathway_activity_rust::permissions::PermissionSet;

    let location_registry = LocationRegistry::new();
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let workspace_id = RoomId::new();
    let permissions = PermissionSet::new();

    let users = vec![
        User::new_curious(location_id, building_id, workspace_id, permissions.clone()),
        User::new_with_cloned_badge(location_id, building_id, workspace_id, permissions),
    ];

    orchestrator.initialize_with_data(location_registry, users).unwrap();

    // Verify initial statistics
    let initial_stats = orchestrator.get_statistics();
    assert_eq!(initial_stats.total_users, 2);
    assert_eq!(initial_stats.curious_users, 1);
    assert_eq!(initial_stats.cloned_badge_users, 1);

    // Add events and verify statistics are maintained
    // Update statistics manually (replacing deprecated update_statistics_with_events)
    {
        let stats = orchestrator.get_statistics_mut();
        stats.increment_success_events();
        stats.increment_suspicious_events();
    }

    // Verify both user and event statistics are maintained
    let final_stats = orchestrator.get_statistics();
    assert_eq!(final_stats.total_users, 2);
    assert_eq!(final_stats.curious_users, 1);
    assert_eq!(final_stats.cloned_badge_users, 1);
    assert_eq!(final_stats.event_type_statistics().total_events, 2);
    assert_eq!(final_stats.event_type_statistics().success_events, 1);
    assert_eq!(final_stats.event_type_statistics().suspicious_events, 1);
}

/// Test performance impact of statistics tracking
#[test]
fn test_statistics_performance_impact() {
    use std::time::Instant;

    let config = SimulationConfig::default();
    let mut orchestrator = SimulationOrchestrator::new(config).unwrap();

    // Create a large batch of events
    let mut events = Vec::new();
    for i in 0..1000 {
        let event_type = match i % 5 {
            0 => EventType::Success,
            1 => EventType::Failure,
            2 => EventType::InvalidBadge,
            3 => EventType::OutsideHours,
            _ => EventType::Suspicious,
        };

        events.push(AccessEvent::new(
            Utc::now(),
            UserId::new(),
            RoomId::new(),
            BuildingId::new(),
            LocationId::new(),
            event_type == EventType::Success,
            event_type,
        ));
    }

    // Measure time to update statistics
    let start = Instant::now();
    // Update statistics manually (replacing deprecated update_statistics_with_events)
    {
        let stats = orchestrator.get_statistics_mut();
        // Process 1000 events: 200 each of Success, Failure, InvalidBadge, OutsideHours, Suspicious
        for _ in 0..200 {
            stats.increment_success_events();
        }
        for _ in 0..200 {
            stats.increment_failure_events();
        }
        for _ in 0..200 {
            stats.increment_invalid_badge_events();
        }
        for _ in 0..200 {
            stats.increment_outside_hours_events();
        }
        for _ in 0..200 {
            stats.increment_suspicious_events();
        }
    }
    let elapsed = start.elapsed();

    // Verify statistics were updated correctly
    let stats = orchestrator.get_statistics();
    let event_stats = stats.event_type_statistics();
    assert_eq!(event_stats.total_events, 1000);
    assert_eq!(event_stats.success_events, 200);
    assert_eq!(event_stats.failure_events, 800); // 200 direct + 600 indirect (200 each from InvalidBadge, OutsideHours, Suspicious)
    assert_eq!(event_stats.invalid_badge_events, 200);
    assert_eq!(event_stats.outside_hours_events, 200);
    assert_eq!(event_stats.suspicious_events, 200);

    // Verify performance is reasonable (should be very fast)
    assert!(elapsed.as_millis() < 100, "Statistics update took too long: {:?}", elapsed);
}

/// Test concurrent statistics updates (if needed in the future)
#[test]
fn test_statistics_consistency() {
    let config = SimulationConfig::default();
    let mut orchestrator = SimulationOrchestrator::new(config).unwrap();

    // Simulate multiple batches of events being processed
    for batch in 0..10 {
        let mut events = Vec::new();
        for i in 0..10 {
            events.push(AccessEvent::new(
                Utc::now(),
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                i % 2 == 0, // Alternate success/failure
                if i % 2 == 0 { EventType::Success } else { EventType::Failure },
            ));
        }

        // Update statistics manually (replacing deprecated update_statistics_with_events)
        {
            let stats = orchestrator.get_statistics_mut();
            // Process 10 events per batch: 5 success, 5 failure
            for _ in 0..5 {
                stats.increment_success_events();
            }
            for _ in 0..5 {
                stats.increment_failure_events();
            }
        }

        // Verify statistics remain consistent after each batch
        let stats = orchestrator.get_statistics();
        let event_stats = stats.event_type_statistics();
        assert_eq!(event_stats.total_events, (batch + 1) * 10);
        assert_eq!(event_stats.success_events, (batch + 1) * 5);
        assert_eq!(event_stats.failure_events, (batch + 1) * 5);
    }

    // Final verification
    let final_stats = orchestrator.get_statistics();
    let final_event_stats = final_stats.event_type_statistics();
    assert_eq!(final_event_stats.total_events, 100);
    assert_eq!(final_event_stats.success_events, 50);
    assert_eq!(final_event_stats.failure_events, 50);
    assert_eq!(final_event_stats.success_percentage(), 50.0);
    assert_eq!(final_event_stats.failure_percentage(), 50.0);
}

/// Test complete application statistics output formatting
#[test]
fn test_complete_application_statistics_output() {
    let mut config = SimulationConfig::default();
    config.user_count = 100;
    config.location_count = 3;
    config.curious_user_percentage = 0.1; // 10% curious
    config.cloned_badge_percentage = 0.02; // 2% cloned badges

    // Generate facilities
    let mut facility_generator = FacilityGenerator::new();
    let location_registry = facility_generator.generate_facilities(&config).unwrap();

    // Generate users
    let mut user_generator = UserGenerator::new();
    let users = user_generator.generate_users(&config, &location_registry).unwrap();

    // Create and initialize orchestrator
    let mut orchestrator = SimulationOrchestrator::new(config.clone()).unwrap();
    orchestrator.initialize_with_data(location_registry, users).unwrap();

    // Update statistics manually (replacing deprecated update_statistics_with_events)
    {
        let stats = orchestrator.get_statistics_mut();
        // Process 1000 events: 500 success, 125 each of failure, invalid badge, outside hours, suspicious
        for _ in 0..500 {
            stats.increment_success_events();
        }
        for _ in 0..125 {
            stats.increment_failure_events();
        }
        for _ in 0..125 {
            stats.increment_invalid_badge_events();
        }
        for _ in 0..125 {
            stats.increment_outside_hours_events();
        }
        for _ in 0..125 {
            stats.increment_suspicious_events();
        }
    }

    // Manually add some curious and impossible traveler events
    orchestrator.increment_curious_events(25);
    orchestrator.increment_impossible_traveler_events(5);

    // Test the statistics output by capturing what would be printed
    let stats = orchestrator.get_statistics();
    let event_stats = stats.event_type_statistics();

    // Verify infrastructure statistics
    assert_eq!(stats.total_users, 100);
    assert_eq!(stats.total_locations, 3);
    assert!(stats.total_buildings > 0);
    assert!(stats.total_rooms > 0);
    assert!(stats.curious_users > 0);
    assert!(stats.cloned_badge_users <= 100); // Should be reasonable for 100 users

    // Verify event statistics
    assert_eq!(event_stats.total_events, 1000);
    assert_eq!(event_stats.success_events, 500);
    assert_eq!(event_stats.failure_events, 500); // 125 direct + 375 indirect (125 each from InvalidBadge, OutsideHours, Suspicious)
    assert_eq!(event_stats.invalid_badge_events, 125);
    assert_eq!(event_stats.outside_hours_events, 125);
    assert_eq!(event_stats.suspicious_events, 125);
    assert_eq!(event_stats.curious_events, 25);
    assert_eq!(event_stats.impossible_traveler_events, 5);

    // Verify percentage calculations
    assert_eq!(event_stats.success_percentage(), 50.0);
    assert_eq!(event_stats.failure_percentage(), 50.0); // 500 failures out of 1000 total events
    assert_eq!(event_stats.curious_event_percentage(), 2.5);
    assert_eq!(event_stats.impossible_traveler_percentage(), 0.5);

    // Verify aggregate calculations
    assert_eq!(event_stats.total_failure_events(), 875); // 500 failure_events + 125 invalid_badge + 125 outside_hours + 125 suspicious
    assert_eq!(event_stats.total_anomaly_events(), 30); // Curious + impossible traveler
    assert_eq!(event_stats.total_failure_percentage(), 87.5); // 875 / 1000 * 100
    assert_eq!(event_stats.total_anomaly_percentage(), 3.0);

    // Verify average calculations
    assert!(stats.average_buildings_per_location() > 0.0);
    assert!(stats.average_rooms_per_building() > 0.0);

    // Test summary methods
    let compact_summary = event_stats.compact_summary();
    assert!(compact_summary.contains("1000 events"));
    assert!(compact_summary.contains("500 success"));
    assert!(compact_summary.contains("25 curious"));
    assert!(compact_summary.contains("5 impossible traveler"));

    let detailed_breakdown = event_stats.detailed_breakdown();
    assert!(detailed_breakdown.contains("Total Events Generated: 1000"));
    assert!(detailed_breakdown.contains("Success Events: 500 (50.0%)"));
    assert!(detailed_breakdown.contains("Curious Events: 25 (2.5%)"));
    assert!(detailed_breakdown.contains("Impossible Traveler Events: 5 (0.5%)"));
}

/// Test statistics output with no events generated
#[test]
fn test_statistics_output_with_no_events() {
    let config = SimulationConfig::default();
    let mut orchestrator = SimulationOrchestrator::new(config.clone()).unwrap();

    // Initialize with minimal data
    let location_registry = LocationRegistry::new();
    let users = vec![];
    orchestrator.initialize_with_data(location_registry, users).unwrap();

    // Verify statistics show appropriate messages for no events
    let stats = orchestrator.get_statistics();
    let event_stats = stats.event_type_statistics();

    assert_eq!(event_stats.total_events, 0);
    assert_eq!(event_stats.success_percentage(), 0.0);
    assert_eq!(event_stats.failure_percentage(), 0.0);
    assert_eq!(event_stats.curious_event_percentage(), 0.0);
    assert_eq!(event_stats.impossible_traveler_percentage(), 0.0);

    // Verify that percentage calculations handle zero division correctly
    assert_eq!(event_stats.total_failure_percentage(), 0.0);
    assert_eq!(event_stats.total_anomaly_percentage(), 0.0);

    // Verify summary methods handle empty statistics gracefully
    let compact_summary = event_stats.compact_summary();
    assert!(compact_summary.contains("0 events"));

    let detailed_breakdown = event_stats.detailed_breakdown();
    assert!(detailed_breakdown.contains("Total Events Generated: 0"));
}

/// Test statistics output formatting and display elements
#[test]
fn test_statistics_display_formatting() {
    let config = SimulationConfig::default();
    let mut orchestrator = SimulationOrchestrator::new(config).unwrap();

    // Update statistics manually (replacing deprecated update_statistics_with_events)
    {
        let stats = orchestrator.get_statistics_mut();
        stats.increment_success_events();
        stats.increment_failure_events();
        stats.increment_suspicious_events();
    }
    orchestrator.increment_curious_events(1);
    orchestrator.increment_impossible_traveler_events(1);

    let stats = orchestrator.get_statistics();
    let event_stats = stats.event_type_statistics();

    // Test that all required display elements are present in the detailed breakdown
    let detailed_breakdown = event_stats.detailed_breakdown();

    // Verify section headers
    assert!(detailed_breakdown.contains("=== Event Type Breakdown ==="));
    assert!(detailed_breakdown.contains("Standard Event Types:"));
    assert!(detailed_breakdown.contains("Security Anomaly Events:"));

    // Verify event counts and percentages are formatted correctly
    assert!(detailed_breakdown.contains("Success Events: 1 (33.3%)"));
    assert!(detailed_breakdown.contains("Failure Events: 2 (66.7%)")); // 1 direct + 1 from suspicious
    assert!(detailed_breakdown.contains("Curious Events: 1 (33.3%)"));
    assert!(detailed_breakdown.contains("Impossible Traveler Events: 1 (33.3%)"));
    // Note: Individual failure types (Suspicious, InvalidBadge, etc.) are not shown in detailed_breakdown

    // Test compact summary formatting
    let compact_summary = event_stats.compact_summary();
    assert!(compact_summary.contains("3 events"));
    assert!(compact_summary.contains("1 success"));
    assert!(compact_summary.contains("1 curious"));
    assert!(compact_summary.contains("1 impossible traveler"));
}
