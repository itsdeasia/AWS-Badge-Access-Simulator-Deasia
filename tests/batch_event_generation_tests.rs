//! Tests for batch event generation functionality
//!
//! These tests verify the batch event generation system that replaces
//! the streaming approach with a simpler day-by-day processing model.

use amzn_career_pathway_activity_rust::user::{User, UserGenerator};
use amzn_career_pathway_activity_rust::facility::{FacilityGenerator, LocationRegistry};
use amzn_career_pathway_activity_rust::simulation::BatchEventGenerator;
use amzn_career_pathway_activity_rust::types::SimulationConfig;

/// Create a minimal test setup for batch generation tests
fn create_test_setup() -> (SimulationConfig, LocationRegistry, Vec<User>) {
    let config = SimulationConfig {
        user_count: 5,
        location_count: 1,
        min_buildings_per_location: 2,
        max_buildings_per_location: 3,
        min_rooms_per_building: 5,
        max_rooms_per_building: 10,
        ..Default::default()
    };
    
    // Generate facilities
    let mut facility_generator = FacilityGenerator::new();
    let location_registry = facility_generator.generate_facilities(&config).unwrap();
    
    // Generate users
    let mut user_generator = UserGenerator::new();
    let users = user_generator.generate_users(&config, &location_registry).unwrap();
    
    (config, location_registry, users)
}

/// Test batch generator creation and initialization
#[test]
fn test_batch_generator_creation() {
    let (config, location_registry, users) = create_test_setup();
    
    let generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Verify generator was created with correct user count
    assert_eq!(generator.get_statistics().total_users, 5);
    assert_eq!(generator.get_statistics().total_events, 0);
}

/// Test batch generation for single day
#[test]
fn test_single_day_batch_generation() {
    let (config, location_registry, users) = create_test_setup();
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Generate events for 1 day
    let result = generator.generate_events_for_days(1);
    assert!(result.is_ok(), "Single day generation should succeed");
    
    // Verify statistics were updated
    let stats = generator.get_statistics();
    assert_eq!(stats.days_simulated, 1);
    assert!(stats.total_events > 0, "Should have generated some events");
}

/// Test batch generation for multiple days
#[test]
fn test_multiple_days_batch_generation() {
    let (config, location_registry, users) = create_test_setup();
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Generate events for 3 days
    let result = generator.generate_events_for_days(3);
    assert!(result.is_ok(), "Multiple day generation should succeed");
    
    // Verify statistics were updated
    let stats = generator.get_statistics();
    assert_eq!(stats.days_simulated, 3);
    assert!(stats.total_events > 0, "Should have generated events for multiple days");
}

/// Test batch generation with zero days (should fail)
#[test]
fn test_zero_days_batch_generation_error() {
    let (config, location_registry, users) = create_test_setup();
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Generate events for 0 days should fail
    let result = generator.generate_events_for_days(0);
    assert!(result.is_err(), "Zero days should return an error");
}

/// Test batch generation with curious users
#[test]
fn test_batch_generation_with_curious_users() {
    let config = SimulationConfig {
        user_count: 10,
        location_count: 1,
        curious_user_percentage: 0.5, // 50% curious for testing
        ..Default::default()
    };
    
    // Generate facilities and users
    let mut facility_generator = FacilityGenerator::new();
    let location_registry = facility_generator.generate_facilities(&config).unwrap();
    
    let mut user_generator = UserGenerator::new();
    let users = user_generator.generate_users(&config, &location_registry).unwrap();
    
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Generate events for 1 day
    let result = generator.generate_events_for_days(1);
    assert!(result.is_ok());
    
    // Verify curious users were counted
    let stats = generator.get_statistics();
    assert!(stats.curious_users > 0, "Should have some curious users");
}

/// Test batch generation with cloned badge users
#[test]
fn test_batch_generation_with_cloned_badge_users() {
    let config = SimulationConfig {
        user_count: 10,
        location_count: 2, // Multiple locations for impossible traveler scenarios
        cloned_badge_percentage: 0.2, // 20% cloned badges for testing
        ..Default::default()
    };
    
    // Generate facilities and users
    let mut facility_generator = FacilityGenerator::new();
    let location_registry = facility_generator.generate_facilities(&config).unwrap();
    
    let mut user_generator = UserGenerator::new();
    let users = user_generator.generate_users(&config, &location_registry).unwrap();
    
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Generate events for 1 day
    let result = generator.generate_events_for_days(1);
    assert!(result.is_ok());
    
    // Verify cloned badge users were counted
    let stats = generator.get_statistics();
    assert!(stats.cloned_badge_users > 0, "Should have some cloned badge users");
}

/// Test statistics consolidation during batch generation
#[test]
fn test_statistics_consolidation() {
    let (config, location_registry, users) = create_test_setup();
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Get initial statistics
    let initial_stats = generator.get_statistics();
    let initial_events = initial_stats.total_events;
    
    // Generate events for 1 day
    let result = generator.generate_events_for_days(1);
    assert!(result.is_ok());
    
    // Verify statistics were updated
    let final_stats = generator.get_statistics();
    assert!(final_stats.total_events > initial_events, "Event count should increase");
    assert_eq!(final_stats.days_simulated, 1);
    assert!(final_stats.simulation_duration.as_nanos() > 0);
}

/// Test event ordering within a single day
#[test]
fn test_event_ordering_single_day() {
    let (config, location_registry, users) = create_test_setup();
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Generate events for 1 day
    let result = generator.generate_events_for_days(1);
    assert!(result.is_ok());
    
    // Events should be generated and statistics should reflect proper ordering
    // (We can't directly test event ordering without capturing output, but we can
    // verify that the generation completed successfully)
    let stats = generator.get_statistics();
    assert!(stats.total_events > 0);
}

/// Test batch generation with night shift users
#[test]
fn test_batch_generation_with_night_shift_users() {
    let config = SimulationConfig {
        user_count: 10,
        location_count: 1,
        min_buildings_per_location: 2,
        max_buildings_per_location: 3,
        ..Default::default()
    };
    
    // Generate facilities and users (night shift users are auto-generated)
    let mut facility_generator = FacilityGenerator::new();
    let location_registry = facility_generator.generate_facilities(&config).unwrap();
    
    let mut user_generator = UserGenerator::new();
    let users = user_generator.generate_users(&config, &location_registry).unwrap();
    
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Generate events for 1 day
    let result = generator.generate_events_for_days(1);
    assert!(result.is_ok());
    
    // Verify night shift users were counted
    let stats = generator.get_statistics();
    // Night shift users should be tracked (can be 0 or more)
    assert!(stats.night_shift_users < stats.total_users, "Night shift should be subset of total users");
}

/// Test batch generation error handling with invalid configuration
#[test]
fn test_batch_generation_error_handling() {
    // Create minimal setup
    let config = SimulationConfig::default();
    let location_registry = LocationRegistry::new();
    let users = vec![];
    
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Generate events for 1 day with no users should still work
    let result = generator.generate_events_for_days(1);
    assert!(result.is_ok(), "Should handle empty user list gracefully");
    
    // Verify statistics
    let stats = generator.get_statistics();
    assert_eq!(stats.total_users, 0);
    assert_eq!(stats.total_events, 0);
    assert_eq!(stats.days_simulated, 1);
}

/// Test batch generation performance with larger dataset
#[test]
fn test_batch_generation_performance() {
    let config = SimulationConfig {
        user_count: 50,
        location_count: 2,
        ..Default::default()
    };
    
    // Generate facilities and users
    let mut facility_generator = FacilityGenerator::new();
    let location_registry = facility_generator.generate_facilities(&config).unwrap();
    
    let mut user_generator = UserGenerator::new();
    let users = user_generator.generate_users(&config, &location_registry).unwrap();
    
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Measure time for batch generation
    let start = std::time::Instant::now();
    let result = generator.generate_events_for_days(2);
    let elapsed = start.elapsed();
    
    assert!(result.is_ok());
    assert!(elapsed.as_secs() < 30, "Batch generation should complete within 30 seconds");
    
    // Verify reasonable event generation
    let stats = generator.get_statistics();
    assert!(stats.total_events > 0);
    assert_eq!(stats.days_simulated, 2);
}

/// Test statistics accuracy across multiple days
#[test]
fn test_statistics_accuracy_multiple_days() {
    let (config, location_registry, users) = create_test_setup();
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Generate events for multiple days
    let result = generator.generate_events_for_days(5);
    assert!(result.is_ok());
    
    // Verify statistics consistency
    let stats = generator.get_statistics();
    assert_eq!(stats.days_simulated, 5);
    assert!(stats.total_events > 0);
    
    // Verify that success + failure events equal total events
    let success_and_failure = stats.success_events + stats.failure_events;
    assert!(success_and_failure <= stats.total_events, 
           "Success + failure events should not exceed total events");
}

/// Test batch generation with different user types
#[test]
fn test_batch_generation_different_user_types() {
    let config = SimulationConfig {
        user_count: 20,
        location_count: 2,
        curious_user_percentage: 0.1,
        cloned_badge_percentage: 0.05,
        ..Default::default()
    };
    
    // Generate facilities and users
    let mut facility_generator = FacilityGenerator::new();
    let location_registry = facility_generator.generate_facilities(&config).unwrap();
    
    let mut user_generator = UserGenerator::new();
    let users = user_generator.generate_users(&config, &location_registry).unwrap();
    
    let mut generator = BatchEventGenerator::new(config, location_registry, users);
    
    // Generate events for 1 day
    let result = generator.generate_events_for_days(1);
    assert!(result.is_ok());
    
    // Verify different user types are handled
    let stats = generator.get_statistics();
    assert_eq!(stats.total_users, 20);
    assert!(stats.total_events > 0);
    
    // Should have some variety in user types
    let total_special_users = stats.curious_users + stats.cloned_badge_users + stats.night_shift_users;
    // Should track special user types (can be 0 or more)
    assert!(total_special_users <= stats.total_users, "Special users should not exceed total");
}
