//! Tests for CLI argument parsing functionality
//!
//! These tests verify that command line arguments are properly parsed,
//! including the new --days option for batch processing.

use amzn_career_pathway_activity_rust::types::config::{CliArgs, SimulationConfig};
use clap::Parser;

/// Test parsing of the days argument
#[test]
fn test_days_argument_parsing() {
    // Test default value
    let args = vec!["test"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.days, 1);
    
    // Test explicit value with --days
    let args = vec!["test", "--days", "5"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.days, 5);
    
    // Test different values
    let args = vec!["test", "--days", "10"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.days, 10);
    
    // Test large value
    let args = vec!["test", "--days", "365"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.days, 365);
}

/// Test days argument validation in configuration
#[test]
fn test_days_configuration_validation() {
    // Test valid days configuration
    let args = CliArgs {
        config: None,
        user_count: None,
        location_count: None,
        min_buildings_per_location: None,
        max_buildings_per_location: None,
        min_rooms_per_building: None,
        max_rooms_per_building: None,
        curious_user_percentage: None,
        cloned_badge_percentage: None,
        primary_building_affinity: None,
        same_location_travel: None,
        different_location_travel: None,
        output_format: None,
        seed: None,
        user_profiles_output: None,
        verbose: false,
        debug: false,
        dry_run: false,
        print_config: false,
        include_failure_reason: false,
        include_event_type: false,
        include_metadata: false,
        include_all_fields: false,
        days: 7,
    };
    
    let config = SimulationConfig::from_cli_args(args).unwrap();
    assert_eq!(config.days, 7);
    
    // Validation should pass for valid days
    config.validate().unwrap();
}

/// Test invalid days argument parsing
#[test]
fn test_invalid_days_argument() {
    // Test zero days (should be caught by validation, not parsing)
    let args = vec!["test", "--days", "0"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.days, 0);
    
    // Create config and test validation
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "Zero days should fail validation");
}

/// Test days argument with other CLI options
#[test]
fn test_days_with_other_options() {
    let args = vec![
        "test",
        "--days", "3",
        "--user-count", "100",
        "--location-count", "2",
        "--verbose"
    ];
    
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.days, 3);
    assert_eq!(cli_args.user_count, Some(100));
    assert_eq!(cli_args.location_count, Some(2));
    assert!(cli_args.verbose);
    
    // Test configuration creation
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert_eq!(config.days, 3);
    assert_eq!(config.user_count, 100);
    assert_eq!(config.location_count, 2);
}

/// Test user count argument parsing
#[test]
fn test_user_count_argument_parsing() {
    let args = vec!["test", "--user-count", "5000"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.user_count, Some(5000));
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert_eq!(config.user_count, 5000);
}

/// Test location count argument parsing
#[test]
fn test_location_count_argument_parsing() {
    let args = vec!["test", "--location-count", "3"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.location_count, Some(3));
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert_eq!(config.location_count, 3);
}

/// Test curious user percentage argument parsing
#[test]
fn test_curious_user_percentage_parsing() {
    let args = vec!["test", "--curious-user-percentage", "0.1"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.curious_user_percentage, Some(0.1));
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert_eq!(config.curious_user_percentage, 0.1);
}

/// Test cloned badge percentage argument parsing
#[test]
fn test_cloned_badge_percentage_parsing() {
    let args = vec!["test", "--cloned-badge-percentage", "0.002"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.cloned_badge_percentage, Some(0.002));
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert_eq!(config.cloned_badge_percentage, 0.002);
}

/// Test output format argument parsing
#[test]
fn test_output_format_parsing() {
    // Test JSON format
    let args = vec!["test", "--output-format", "json"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.output_format, Some("json".to_string()));
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert_eq!(config.output_format, "json");
    
    // Test CSV format
    let args = vec!["test", "--output-format", "csv"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.output_format, Some("csv".to_string()));
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert_eq!(config.output_format, "csv");
}

/// Test seed argument parsing
#[test]
fn test_seed_argument_parsing() {
    let args = vec!["test", "--seed", "12345"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.seed, Some(12345));
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert_eq!(config.seed, Some(12345));
}

/// Test verbose and debug flags
#[test]
fn test_logging_flags() {
    // Test verbose flag
    let args = vec!["test", "--verbose"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert!(cli_args.verbose);
    assert!(!cli_args.debug);
    
    // Test debug flag
    let args = vec!["test", "--debug"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert!(!cli_args.verbose);
    assert!(cli_args.debug);
    
    // Test both flags
    let args = vec!["test", "--verbose", "--debug"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert!(cli_args.verbose);
    assert!(cli_args.debug);
}

/// Test dry run flag
#[test]
fn test_dry_run_flag() {
    let args = vec!["test", "--dry-run"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert!(cli_args.dry_run);
}

/// Test print config flag
#[test]
fn test_print_config_flag() {
    let args = vec!["test", "--print-config"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert!(cli_args.print_config);
}

/// Test output field inclusion flags
#[test]
fn test_output_field_flags() {
    let args = vec![
        "test",
        "--include-failure-reason",
        "--include-event-type",
        "--include-metadata"
    ];
    
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert!(cli_args.include_failure_reason);
    assert!(cli_args.include_event_type);
    assert!(cli_args.include_metadata);
    assert!(!cli_args.include_all_fields);
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert!(config.output_fields.include_failure_reason);
    assert!(config.output_fields.include_event_type);
    assert!(config.output_fields.include_metadata);
    assert!(!config.output_fields.include_all);
}

/// Test include all fields flag
#[test]
fn test_include_all_fields_flag() {
    let args = vec!["test", "--include-all-fields"];
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert!(cli_args.include_all_fields);
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert!(config.output_fields.include_all);
    assert!(config.output_fields.include_failure_reason);
    assert!(config.output_fields.include_event_type);
    assert!(config.output_fields.include_metadata);
}

/// Test building and room range arguments
#[test]
fn test_building_and_room_range_arguments() {
    let args = vec![
        "test",
        "--min-buildings-per-location", "2",
        "--max-buildings-per-location", "8",
        "--min-rooms-per-building", "5",
        "--max-rooms-per-building", "25"
    ];
    
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.min_buildings_per_location, Some(2));
    assert_eq!(cli_args.max_buildings_per_location, Some(8));
    assert_eq!(cli_args.min_rooms_per_building, Some(5));
    assert_eq!(cli_args.max_rooms_per_building, Some(25));
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert_eq!(config.min_buildings_per_location, 2);
    assert_eq!(config.max_buildings_per_location, 8);
    assert_eq!(config.min_rooms_per_building, 5);
    assert_eq!(config.max_rooms_per_building, 25);
}

/// Test affinity and travel probability arguments
#[test]
fn test_affinity_and_travel_arguments() {
    let args = vec![
        "test",
        "--primary-building-affinity", "0.8",
        "--same-location-travel", "0.15",
        "--different-location-travel", "0.05"
    ];
    
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    assert_eq!(cli_args.primary_building_affinity, Some(0.8));
    assert_eq!(cli_args.same_location_travel, Some(0.15));
    assert_eq!(cli_args.different_location_travel, Some(0.05));
    
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    assert_eq!(config.primary_building_affinity, 0.8);
    assert_eq!(config.same_location_travel, 0.15);
    assert_eq!(config.different_location_travel, 0.05);
}

/// Test configuration validation with CLI arguments
#[test]
fn test_configuration_validation_with_cli() {
    // Test valid configuration
    let args = vec![
        "test",
        "--user-count", "100",
        "--location-count", "2",
        "--days", "5"
    ];
    
    let cli_args = CliArgs::try_parse_from(args).unwrap();
    let config = SimulationConfig::from_cli_args(cli_args).unwrap();
    
    // Should pass validation
    config.validate().unwrap();
    
    assert_eq!(config.user_count, 100);
    assert_eq!(config.location_count, 2);
    assert_eq!(config.days, 5);
}

/// Test help message generation (basic test)
#[test]
fn test_help_message() {
    let args = vec!["test", "--help"];
    let result = CliArgs::try_parse_from(args);
    
    // Should fail with help message (this is expected behavior)
    assert!(result.is_err());
    
    // The error should contain help information
    let error = result.unwrap_err();
    let error_string = error.to_string();
    assert!(error_string.contains("badge-access-simulator") || error_string.contains("Usage"));
}
