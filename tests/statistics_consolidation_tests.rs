//! Tests for statistics consolidation and accuracy
//!
//! These tests verify that the consolidated statistics system properly
//! tracks all event types in a single location without duplication.

use amzn_career_pathway_activity_rust::simulation::SimulationStatistics;

/// Test basic statistics initialization
#[test]
fn test_statistics_initialization() {
    let stats = SimulationStatistics::new(100, 5, 25, 500, 10, 2, 8);
    
    // Verify infrastructure statistics
    assert_eq!(stats.total_users, 100);
    assert_eq!(stats.total_locations, 5);
    assert_eq!(stats.total_buildings, 25);
    assert_eq!(stats.total_rooms, 500);
    assert_eq!(stats.curious_users, 10);
    assert_eq!(stats.cloned_badge_users, 2);
    assert_eq!(stats.night_shift_users, 8);
    
    // Verify event statistics start at zero
    assert_eq!(stats.total_events, 0);
    assert_eq!(stats.success_events, 0);
    assert_eq!(stats.failure_events, 0);
    assert_eq!(stats.curious_events, 0);
    assert_eq!(stats.impossible_traveler_events, 0);
    assert_eq!(stats.night_shift_events, 0);
}

/// Test success event counting
#[test]
fn test_success_event_counting() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 1);
    
    // Add success events
    stats.increment_success_events();
    stats.increment_success_events();
    stats.increment_success_events();
    
    assert_eq!(stats.success_events, 3);
    assert_eq!(stats.total_events, 3);
    assert_eq!(stats.failure_events, 0);
}

/// Test failure event counting
#[test]
fn test_failure_event_counting() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 1);
    
    // Add failure events
    stats.increment_failure_events();
    stats.increment_failure_events();
    
    assert_eq!(stats.failure_events, 2);
    assert_eq!(stats.total_events, 2);
    assert_eq!(stats.success_events, 0);
}

/// Test mixed event counting
#[test]
fn test_mixed_event_counting() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 1);
    
    // Add mixed events
    stats.increment_success_events();
    stats.increment_failure_events();
    stats.increment_success_events();
    stats.increment_invalid_badge_events(); // This also increments failure_events
    stats.increment_outside_hours_events(); // This also increments failure_events
    
    assert_eq!(stats.success_events, 2);
    assert_eq!(stats.failure_events, 3); // 1 direct + 2 from invalid_badge and outside_hours
    assert_eq!(stats.invalid_badge_events, 1);
    assert_eq!(stats.outside_hours_events, 1);
    assert_eq!(stats.total_events, 5);
}

/// Test curious event counting
#[test]
fn test_curious_event_counting() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 5, 0, 1);
    
    // Add curious events (these don't count toward total_events)
    stats.increment_curious_events();
    stats.increment_curious_events();
    
    assert_eq!(stats.curious_events, 2);
    assert_eq!(stats.total_events, 0); // Curious events don't count toward total
}

/// Test impossible traveler event counting
#[test]
fn test_impossible_traveler_event_counting() {
    let mut stats = SimulationStatistics::new(10, 2, 4, 40, 1, 2, 1);
    
    // Add impossible traveler events (these don't count toward total_events)
    stats.increment_impossible_traveler_events();
    stats.increment_impossible_traveler_events();
    stats.increment_impossible_traveler_events();
    
    assert_eq!(stats.impossible_traveler_events, 3);
    assert_eq!(stats.total_events, 0); // Impossible traveler events don't count toward total
}

/// Test night shift event counting
#[test]
fn test_night_shift_event_counting() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 3);
    
    // Add night shift events (these don't count toward total_events)
    stats.increment_night_shift_events();
    stats.increment_night_shift_events();
    
    assert_eq!(stats.night_shift_events, 2);
    assert_eq!(stats.total_events, 0); // Night shift events don't count toward total
}

/// Test suspicious event counting
#[test]
fn test_suspicious_event_counting() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 1);
    
    // Add suspicious events
    stats.increment_suspicious_events();
    stats.increment_suspicious_events();
    
    assert_eq!(stats.suspicious_events, 2);
    assert_eq!(stats.total_events, 2);
}

/// Test badge reader failure event counting
#[test]
fn test_badge_reader_failure_counting() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 1);
    
    // Add badge reader failure events
    stats.increment_badge_reader_failure_events();
    stats.increment_badge_reader_failure_events();
    
    assert_eq!(stats.badge_reader_failure_events, 2);
    assert_eq!(stats.total_events, 2);
}

/// Test percentage calculations
#[test]
fn test_percentage_calculations() {
    let mut stats = SimulationStatistics::new(100, 2, 10, 200, 10, 5, 15);
    
    // Add events to test percentages
    for _ in 0..60 {
        stats.increment_success_events();
    }
    for _ in 0..40 {
        stats.increment_failure_events();
    }
    for _ in 0..5 {
        stats.increment_curious_events();
    }
    for _ in 0..3 {
        stats.increment_impossible_traveler_events();
    }
    
    // Test user percentages
    assert_eq!(stats.curious_user_percentage(), 10.0);
    assert_eq!(stats.cloned_badge_percentage(), 5.0);
    assert_eq!(stats.night_shift_user_percentage(), 15.0);
    
    // Test event percentages (total events = 60 + 40 = 100, curious and impossible don't count toward total)
    let success_pct = (60.0 / 100.0) * 100.0;
    let failure_pct = (40.0 / 100.0) * 100.0;
    let curious_pct = (5.0 / 100.0) * 100.0; // Curious percentage is relative to total_events
    let impossible_pct = (3.0 / 100.0) * 100.0; // Impossible traveler percentage is relative to total_events
    
    assert!((stats.success_percentage() - success_pct).abs() < 0.1);
    assert!((stats.failure_percentage() - failure_pct).abs() < 0.1);
    assert!((stats.curious_event_percentage() - curious_pct).abs() < 0.1);
    assert!((stats.impossible_traveler_percentage() - impossible_pct).abs() < 0.1);
}

/// Test percentage calculations with zero events
#[test]
fn test_percentage_calculations_zero_events() {
    let stats = SimulationStatistics::new(10, 1, 2, 20, 2, 1, 1);
    
    // With zero events, percentages should be 0.0
    assert_eq!(stats.success_percentage(), 0.0);
    assert_eq!(stats.failure_percentage(), 0.0);
    assert_eq!(stats.curious_event_percentage(), 0.0);
    assert_eq!(stats.impossible_traveler_percentage(), 0.0);
    
    // User percentages should still work
    assert_eq!(stats.curious_user_percentage(), 20.0);
    assert_eq!(stats.cloned_badge_percentage(), 10.0);
    assert_eq!(stats.night_shift_user_percentage(), 10.0);
}

/// Test average calculations
#[test]
fn test_average_calculations() {
    let stats = SimulationStatistics::new(100, 5, 25, 500, 10, 2, 8);
    
    // Test building and room averages
    assert_eq!(stats.average_buildings_per_location(), 5.0); // 25 buildings / 5 locations
    assert_eq!(stats.average_rooms_per_building(), 20.0); // 500 rooms / 25 buildings
}

/// Test average calculations with zero values
#[test]
fn test_average_calculations_zero_values() {
    let stats = SimulationStatistics::new(10, 0, 0, 0, 1, 0, 1);
    
    // With zero locations/buildings, averages should be 0.0
    assert_eq!(stats.average_buildings_per_location(), 0.0);
    assert_eq!(stats.average_rooms_per_building(), 0.0);
}

/// Test simulation metadata tracking
#[test]
fn test_simulation_metadata() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 1);
    
    // Set simulation metadata
    stats.set_days_simulated(7);
    stats.set_simulation_duration(std::time::Duration::from_secs(120));
    
    assert_eq!(stats.days_simulated, 7);
    assert_eq!(stats.simulation_duration.as_secs(), 120);
}

/// Test daily average calculations
#[test]
fn test_daily_average_calculations() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 1);
    
    // Add events
    for _ in 0..100 {
        stats.increment_success_events();
    }
    for _ in 0..50 {
        stats.increment_failure_events();
    }
    for _ in 0..10 {
        stats.increment_curious_events();
    }
    
    // Set 5 days simulated
    stats.set_days_simulated(5);
    
    // Test daily averages (total events = 100 + 50 = 150, curious don't count toward total)
    assert_eq!(stats.average_events_per_day(), 30.0); // 150 total events / 5 days
    assert_eq!(stats.average_curious_events_per_day(), 2.0); // 10 curious / 5 days
}

/// Test daily averages with zero days
#[test]
fn test_daily_averages_zero_days() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 1);
    
    // Add events but don't set days simulated (defaults to 0)
    stats.increment_success_events();
    stats.increment_failure_events();
    
    // With zero days, averages should be 0.0
    assert_eq!(stats.average_events_per_day(), 0.0);
}

/// Test statistics consolidation (no duplication)
#[test]
fn test_statistics_consolidation_no_duplication() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 1);
    
    // Add the same type of event multiple times
    stats.increment_success_events();
    stats.increment_success_events();
    stats.increment_success_events();
    
    // Verify no duplication - each increment should add exactly 1
    assert_eq!(stats.success_events, 3);
    assert_eq!(stats.total_events, 3);
    
    // Add different types
    stats.increment_failure_events();
    stats.increment_curious_events(); // Doesn't count toward total
    
    // Verify proper consolidation
    assert_eq!(stats.success_events, 3);
    assert_eq!(stats.failure_events, 1);
    assert_eq!(stats.curious_events, 1);
    assert_eq!(stats.total_events, 4); // Only success + failure count toward total
}

/// Test statistics accuracy with large numbers
#[test]
fn test_statistics_accuracy_large_numbers() {
    let mut stats = SimulationStatistics::new(10000, 10, 100, 5000, 500, 50, 200);
    
    // Add large numbers of events
    for _ in 0..10000 {
        stats.increment_success_events();
    }
    for _ in 0..5000 {
        stats.increment_failure_events();
    }
    for _ in 0..1000 {
        stats.increment_curious_events();
    }
    for _ in 0..100 {
        stats.increment_impossible_traveler_events();
    }
    
    // Verify accuracy with large numbers (total events = success + failure only)
    assert_eq!(stats.success_events, 10000);
    assert_eq!(stats.failure_events, 5000);
    assert_eq!(stats.curious_events, 1000);
    assert_eq!(stats.impossible_traveler_events, 100);
    assert_eq!(stats.total_events, 15000); // Only success + failure count toward total
    
    // Test percentage accuracy
    assert!((stats.success_percentage() - 66.67).abs() < 0.1); // 10000/15000 * 100
    assert!((stats.failure_percentage() - 33.33).abs() < 0.1); // 5000/15000 * 100
}

/// Test statistics thread safety (basic test)
#[test]
fn test_statistics_consistency() {
    let mut stats = SimulationStatistics::new(10, 1, 2, 20, 1, 0, 1);
    
    // Simulate concurrent-like updates (sequential in test)
    for i in 0..100 {
        if i % 2 == 0 {
            stats.increment_success_events();
        } else {
            stats.increment_failure_events();
        }
    }
    
    // Verify consistency
    assert_eq!(stats.success_events, 50);
    assert_eq!(stats.failure_events, 50);
    assert_eq!(stats.total_events, 100);
    assert_eq!(stats.success_percentage(), 50.0);
    assert_eq!(stats.failure_percentage(), 50.0);
}

/// Test simplified statistics output format
#[test]
fn test_simplified_statistics_output() {
    let mut stats = SimulationStatistics::new(100, 3, 15, 300, 10, 2, 5);
    
    // Add some events
    for _ in 0..1000 {
        stats.increment_success_events();
    }
    for _ in 0..200 {
        stats.increment_failure_events();
    }
    for _ in 0..50 {
        stats.increment_curious_events();
    }
    for _ in 0..10 {
        stats.increment_impossible_traveler_events();
    }
    for _ in 0..30 {
        stats.increment_night_shift_events();
    }
    
    stats.set_days_simulated(5);
    stats.set_simulation_duration(std::time::Duration::from_secs(300));
    
    // Test simplified output generation
    let output = stats.generate_simplified_statistics_output();
    
    // Verify output contains expected information
    assert!(output.contains("Simulation Complete"));
    assert!(output.contains("Total Events Generated: 1200")); // 1000 success + 200 failure (curious, impossible, night-shift don't count toward total)
    assert!(output.contains("5 days") || output.contains("Days Simulated: 5"));
    assert!(output.contains("240.0 events/day")); // 1200/5
    assert!(output.contains("50 curious") || output.contains("Curious User Events: 50"));
    assert!(output.contains("10 impossible") || output.contains("Impossible Traveler Events: 10"));
    assert!(output.contains("30 night-shift") || output.contains("Night-Shift Events: 30"));
    assert!(output.contains("300") || output.contains("5.0")); // Duration in seconds or minutes
}
