//! Configuration structures for the badge access simulator
//!
//! This module contains the simulation configuration structure and validation logic
//! used to control the behavior and parameters of the simulation system.

use super::OutputFormat;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Night shift configuration constants
pub mod night_shift {
    /// Night shift start hour (5 PM in 24-hour format)
    pub const START_HOUR: u8 = 17;
    
    /// Night shift end hour (8 AM in 24-hour format)
    pub const END_HOUR: u8 = 8;
    
    /// Minimum night shift users per building
    pub const MIN_USERS_PER_BUILDING: usize = 1;

    /// Maximum night shift users per building
    pub const MAX_USERS_PER_BUILDING: usize = 3;
}

/// Configuration for which fields to include in event output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputFieldConfig {
    /// Include failure_reason field in output (default: false)
    pub include_failure_reason: bool,
    /// Include event_type field in output (default: false)
    pub include_event_type: bool,
    /// Include metadata field in output (default: false)
    pub include_metadata: bool,
    /// Include all fields (overrides individual settings, default: false)
    pub include_all: bool,
}

impl Default for OutputFieldConfig {
    fn default() -> Self {
        Self {
            include_failure_reason: false,
            include_event_type: false,
            include_metadata: false,
            include_all: false,
        }
    }
}

/// Command line arguments structure
#[derive(Debug, Clone, Parser)]
#[command(
    name = "badge-access-simulator",
    version = "1.0.0",
    about = "Badge Access Simulator - Generates realistic user badge access events",
    long_about = "Generates badge access events that mimic real-world security patterns across multiple geographical locations with realistic behavioral patterns, authorization violations, and security anomalies.

EXAMPLES:
    # Run with default settings
    badge-access-simulator

    # Use a configuration file
    badge-access-simulator --config config.json

    # Override specific settings
    badge-access-simulator --user-count 5000 --location-count 3

    # Generate configuration template
    badge-access-simulator --print-config > my-config.json

    # Validate configuration without running
    badge-access-simulator --config my-config.json --dry-run

    # Enable verbose logging
    badge-access-simulator --verbose

CONFIGURATION:
    Configuration can be provided via:
    1. Command line arguments (highest priority)
    2. Configuration file (--config flag)
    3. Default values (lowest priority)

    Supported configuration file formats: JSON (.json)
    
    Use --print-config to generate a template configuration file."
)]
pub struct CliArgs {
    /// Configuration file path (JSON format)
    #[arg(
        short,
        long,
        help = "Configuration file path (JSON format)",
        long_help = "Path to a JSON configuration file. CLI arguments will override file settings."
    )]
    pub config: Option<String>,

    /// Number of users to simulate
    #[arg(
        long,
        help = "Number of users to simulate",
        long_help = "Total number of users in the simulation. Must be greater than 0. Default: 10000"
    )]
    pub user_count: Option<usize>,

    /// Number of geographical locations to create
    #[arg(
        long,
        help = "Number of geographical locations",
        long_help = "Number of geographical locations to create. Each location contains multiple buildings. Must be greater than 0. Default: 5"
    )]
    pub location_count: Option<usize>,

    /// Minimum number of buildings per location
    #[arg(long, help = "Minimum buildings per location")]
    pub min_buildings_per_location: Option<usize>,

    /// Maximum number of buildings per location
    #[arg(long, help = "Maximum buildings per location")]
    pub max_buildings_per_location: Option<usize>,

    /// Minimum number of rooms per building
    #[arg(long, help = "Minimum rooms per building")]
    pub min_rooms_per_building: Option<usize>,

    /// Maximum number of rooms per building
    #[arg(long, help = "Maximum rooms per building")]
    pub max_rooms_per_building: Option<usize>,

    /// Percentage of users with curious behavior (0.0-1.0)
    #[arg(
        long,
        help = "Percentage of curious users (0.0-1.0)",
        long_help = "Percentage of users who will occasionally attempt unauthorized access. Range: 0.0-1.0. Default: 0.05 (5%)"
    )]
    pub curious_user_percentage: Option<f64>,

    /// Percentage of users with cloned badges (0.0-1.0)
    #[arg(
        long,
        help = "Percentage of users with cloned badges (0.0-1.0)",
        long_help = "Percentage of users whose badges are cloned, creating impossible traveler scenarios. Range: 0.0-1.0. Default: 0.001 (0.1%)"
    )]
    pub cloned_badge_percentage: Option<f64>,

    /// Affinity for users to stay in their primary building (0.0-1.0)
    #[arg(long, help = "Primary building affinity (0.0-1.0)")]
    pub primary_building_affinity: Option<f64>,

    /// Probability of traveling within the same location (0.0-1.0)
    #[arg(long, help = "Same location travel probability (0.0-1.0)")]
    pub same_location_travel: Option<f64>,

    /// Probability of traveling to different locations (0.0-1.0)
    #[arg(long, help = "Different location travel probability (0.0-1.0)")]
    pub different_location_travel: Option<f64>,



    /// Output format for generated events
    #[arg(
        long,
        help = "Output format (json or csv)",
        long_help = "Output format for generated events. Supported formats: json, csv. Default: json"
    )]
    pub output_format: Option<String>,

    /// Random seed for reproducible results
    #[arg(long, help = "Random seed for reproducible results")]
    pub seed: Option<u64>,

    /// Output path for user profiles (answer key)
    #[arg(long, help = "Output path for user profiles JSONL file")]
    pub user_profiles_output: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, help = "Enable verbose logging")]
    pub verbose: bool,

    /// Enable debug logging
    #[arg(short, long, help = "Enable debug logging")]
    pub debug: bool,

    /// Dry run mode - validate configuration without running simulation
    #[arg(long, help = "Validate configuration without running simulation")]
    pub dry_run: bool,

    /// Print default configuration and exit
    #[arg(long, help = "Print default configuration in JSON format and exit")]
    pub print_config: bool,

    /// Include failure_reason field in output
    #[arg(
        long,
        help = "Include failure_reason field in output",
        long_help = "Include the failure_reason field in event output. This provides details about why access attempts failed."
    )]
    pub include_failure_reason: bool,

    /// Include event_type field in output
    #[arg(
        long,
        help = "Include event_type field in output",
        long_help = "Include the event_type field in event output. This provides information about the type of access event."
    )]
    pub include_event_type: bool,

    /// Include metadata field in output
    #[arg(
        long,
        help = "Include metadata field in output",
        long_help = "Include the metadata field in event output. This provides additional contextual information about the event."
    )]
    pub include_metadata: bool,

    /// Include all available fields in output
    #[arg(
        long,
        help = "Include all available fields in output",
        long_help = "Include all available fields in event output. This overrides individual field settings and provides maximum detail."
    )]
    pub include_all_fields: bool,

    /// Number of days to simulate
    #[arg(
        long,
        default_value = "1",
        help = "Number of days to simulate",
        long_help = "Number of days to simulate. Must be greater than 0. Default: 1"
    )]
    pub days: usize,
}

/// Configuration file structure (allows partial configuration)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    /// Number of users to simulate
    pub user_count: Option<usize>,

    /// Number of geographical locations to create
    pub location_count: Option<usize>,

    /// Minimum number of buildings per location
    pub min_buildings_per_location: Option<usize>,

    /// Maximum number of buildings per location
    pub max_buildings_per_location: Option<usize>,

    /// Minimum number of rooms per building
    pub min_rooms_per_building: Option<usize>,

    /// Maximum number of rooms per building
    pub max_rooms_per_building: Option<usize>,

    /// Percentage of users with curious behavior (0.0-1.0)
    pub curious_user_percentage: Option<f64>,

    /// Percentage of users with cloned badges (0.0-1.0)
    pub cloned_badge_percentage: Option<f64>,

    /// Affinity for users to stay in their primary building (0.0-1.0)
    pub primary_building_affinity: Option<f64>,

    /// Probability of traveling within the same location (0.0-1.0)
    pub same_location_travel: Option<f64>,

    /// Probability of traveling to different locations (0.0-1.0)
    pub different_location_travel: Option<f64>,



    /// Output format for generated events
    pub output_format: Option<String>,

    /// Random seed for reproducible results
    pub seed: Option<u64>,

    /// Output path for user profiles (answer key)
    pub user_profiles_output: Option<String>,

    /// Configuration for which fields to include in event output
    pub output_fields: Option<OutputFieldConfig>,

    /// Number of days to simulate
    pub days: Option<usize>,
}

/// Configuration for the badge access simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Number of users to simulate
    pub user_count: usize,

    /// Number of geographical locations to create
    pub location_count: usize,

    /// Minimum number of buildings per location
    pub min_buildings_per_location: usize,

    /// Maximum number of buildings per location
    pub max_buildings_per_location: usize,

    /// Minimum number of rooms per building
    pub min_rooms_per_building: usize,

    /// Maximum number of rooms per building
    pub max_rooms_per_building: usize,

    /// Percentage of users with curious behavior (0.0-1.0)
    pub curious_user_percentage: f64,

    /// Percentage of users with cloned badges (0.0-1.0)
    pub cloned_badge_percentage: f64,

    /// Affinity for users to stay in their primary building (0.0-1.0)
    pub primary_building_affinity: f64,

    /// Probability of traveling within the same location (0.0-1.0)
    pub same_location_travel: f64,

    /// Probability of traveling to different locations (0.0-1.0)
    pub different_location_travel: f64,



    /// Output format for generated events
    pub output_format: String,

    /// Random seed for reproducible results
    pub seed: Option<u64>,

    /// Output path for user profiles (answer key)
    pub user_profiles_output: Option<String>,

    /// Configuration for which fields to include in event output
    pub output_fields: OutputFieldConfig,

    /// Number of days to simulate
    pub days: usize,
}

/// Configuration loading and validation errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    /// Configuration file read error
    #[error("Failed to read configuration file: {0}")]
    ReadError(#[from] std::io::Error),

    /// JSON parsing error
    #[error("Failed to parse JSON configuration: {0}")]
    JsonError(#[from] serde_json::Error),

    /// TOML parsing error
    #[error("Failed to parse TOML configuration: {0}")]
    TomlError(String),

    /// Unsupported configuration file format
    #[error("Unsupported configuration file format: {0} (supported: .json, .toml)")]
    UnsupportedFormat(String),
}

/// Validation errors for simulation configuration
#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    /// User count is invalid
    #[error("User count must be greater than 0, got {0}")]
    InvalidUserCount(usize),

    /// Days count is invalid
    #[error("Days count must be greater than 0, got {0}")]
    InvalidDaysCount(usize),

    /// Location count is invalid
    #[error("Location count must be greater than 0, got {0}")]
    InvalidLocationCount(usize),

    /// Building range is invalid
    #[error("Invalid building range: min ({0}) must be <= max ({1})")]
    InvalidBuildingRange(usize, usize),

    /// Room range is invalid
    #[error("Invalid room range: min ({0}) must be <= max ({1})")]
    InvalidRoomRange(usize, usize),

    /// Percentage value is out of range
    #[error("Invalid percentage for {field}: {value} (must be between 0.0 and 1.0)")]
    InvalidPercentage {
        /// Name of the field with invalid percentage
        field: String,
        /// The invalid percentage value
        value: f64,
    },

    /// Affinity values don't sum to 1.0
    #[error("Affinity values must sum to 1.0, got {sum}")]
    InvalidAffinitySum {
        /// The actual sum of affinity values
        sum: f64,
    },


}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            user_count: 10_000,
            location_count: 5,
            min_buildings_per_location: 4,
            max_buildings_per_location: 6,
            min_rooms_per_building: 10,
            max_rooms_per_building: 50,
            curious_user_percentage: 0.05,
            cloned_badge_percentage: 0.001,
            primary_building_affinity: 0.7,
            same_location_travel: 0.29,
            different_location_travel: 0.01,
            output_format: "json".to_string(),
            seed: None,
            user_profiles_output: None,
            output_fields: OutputFieldConfig::default(),
            days: 1,
        }
    }
}

impl SimulationConfig {
    /// Create a new configuration from command line arguments and optional config file
    pub fn from_args() -> Result<Self, ConfigError> {
        let args = CliArgs::parse();
        Self::from_cli_args(args)
    }

    /// Create configuration from parsed CLI arguments
    pub fn from_cli_args(args: CliArgs) -> Result<Self, ConfigError> {
        // Start with default configuration
        let mut config = Self::default();

        // Load from config file if specified
        if let Some(config_path) = &args.config {
            config = Self::from_file(config_path)?;
        }

        // Override with command line arguments (CLI takes precedence)
        Self::apply_cli_overrides(&mut config, args);

        Ok(config)
    }

    /// Load configuration from a file (JSON or TOML)
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.display().to_string()));
        }

        let content = fs::read_to_string(path)?;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => {
                let config_file: ConfigFile = serde_json::from_str(&content)?;
                Ok(Self::from_config_file(config_file))
            }
            Some("toml") => {
                // Add TOML support if needed in the future
                Err(ConfigError::TomlError("TOML support not yet implemented".to_string()))
            }
            Some(ext) => Err(ConfigError::UnsupportedFormat(ext.to_string())),
            None => Err(ConfigError::UnsupportedFormat("no extension".to_string())),
        }
    }

    /// Create configuration from a config file, merging with defaults
    fn from_config_file(config_file: ConfigFile) -> Self {
        let defaults = Self::default();

        Self {
            user_count: config_file.user_count.unwrap_or(defaults.user_count),
            location_count: config_file.location_count.unwrap_or(defaults.location_count),
            min_buildings_per_location: config_file
                .min_buildings_per_location
                .unwrap_or(defaults.min_buildings_per_location),
            max_buildings_per_location: config_file
                .max_buildings_per_location
                .unwrap_or(defaults.max_buildings_per_location),
            min_rooms_per_building: config_file
                .min_rooms_per_building
                .unwrap_or(defaults.min_rooms_per_building),
            max_rooms_per_building: config_file
                .max_rooms_per_building
                .unwrap_or(defaults.max_rooms_per_building),
            curious_user_percentage: config_file
                .curious_user_percentage
                .unwrap_or(defaults.curious_user_percentage),
            cloned_badge_percentage: config_file
                .cloned_badge_percentage
                .unwrap_or(defaults.cloned_badge_percentage),
            primary_building_affinity: config_file
                .primary_building_affinity
                .unwrap_or(defaults.primary_building_affinity),
            same_location_travel: config_file
                .same_location_travel
                .unwrap_or(defaults.same_location_travel),
            different_location_travel: config_file
                .different_location_travel
                .unwrap_or(defaults.different_location_travel),
            output_format: config_file.output_format.unwrap_or(defaults.output_format),
            seed: config_file.seed.or(defaults.seed),
            user_profiles_output: config_file
                .user_profiles_output
                .or(defaults.user_profiles_output),
            output_fields: config_file.output_fields.unwrap_or(defaults.output_fields),
            days: config_file.days.unwrap_or(defaults.days),
        }
    }

    /// Apply CLI argument overrides to configuration
    fn apply_cli_overrides(config: &mut Self, args: CliArgs) {
        if let Some(value) = args.user_count {
            config.user_count = value;
        }
        if let Some(value) = args.location_count {
            config.location_count = value;
        }
        if let Some(value) = args.min_buildings_per_location {
            config.min_buildings_per_location = value;
        }
        if let Some(value) = args.max_buildings_per_location {
            config.max_buildings_per_location = value;
        }
        if let Some(value) = args.min_rooms_per_building {
            config.min_rooms_per_building = value;
        }
        if let Some(value) = args.max_rooms_per_building {
            config.max_rooms_per_building = value;
        }
        if let Some(value) = args.curious_user_percentage {
            config.curious_user_percentage = value;
        }
        if let Some(value) = args.cloned_badge_percentage {
            config.cloned_badge_percentage = value;
        }
        if let Some(value) = args.primary_building_affinity {
            config.primary_building_affinity = value;
        }
        if let Some(value) = args.same_location_travel {
            config.same_location_travel = value;
        }
        if let Some(value) = args.different_location_travel {
            config.different_location_travel = value;
        }
        if let Some(value) = args.output_format {
            config.output_format = value;
        }

        if let Some(value) = args.seed {
            config.seed = Some(value);
        }
        if let Some(value) = args.user_profiles_output {
            config.user_profiles_output = Some(value);
        }

        // Handle output field configuration from CLI arguments
        // CLI arguments override config file settings
        if args.include_failure_reason || args.include_event_type || args.include_metadata || args.include_all_fields {
            config.output_fields = OutputFieldConfig {
                include_failure_reason: args.include_failure_reason || args.include_all_fields,
                include_event_type: args.include_event_type || args.include_all_fields,
                include_metadata: args.include_metadata || args.include_all_fields,
                include_all: args.include_all_fields,
            };
        }

        // Apply days override (always applied since it has a default value)
        config.days = args.days;
    }

    /// Save configuration to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Print configuration as JSON
    pub fn print_json(&self) -> Result<String, ConfigError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Validate the configuration parameters
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        // Validate user count
        if self.user_count == 0 {
            return Err(ConfigValidationError::InvalidUserCount(self.user_count));
        }

        // Validate days count
        if self.days == 0 {
            return Err(ConfigValidationError::InvalidDaysCount(self.days));
        }

        // Validate location count
        if self.location_count == 0 {
            return Err(ConfigValidationError::InvalidLocationCount(self.location_count));
        }

        // Validate building range
        if self.min_buildings_per_location == 0
            || self.max_buildings_per_location == 0
            || self.min_buildings_per_location > self.max_buildings_per_location
        {
            return Err(ConfigValidationError::InvalidBuildingRange(
                self.min_buildings_per_location,
                self.max_buildings_per_location,
            ));
        }

        // Validate room range
        if self.min_rooms_per_building == 0
            || self.max_rooms_per_building == 0
            || self.min_rooms_per_building > self.max_rooms_per_building
        {
            return Err(ConfigValidationError::InvalidRoomRange(
                self.min_rooms_per_building,
                self.max_rooms_per_building,
            ));
        }

        // Validate percentages
        self.validate_percentage("curious_user_percentage", self.curious_user_percentage)?;
        self.validate_percentage("cloned_badge_percentage", self.cloned_badge_percentage)?;
        self.validate_percentage("primary_building_affinity", self.primary_building_affinity)?;
        self.validate_percentage("same_location_travel", self.same_location_travel)?;
        self.validate_percentage("different_location_travel", self.different_location_travel)?;

        // Validate affinity sum
        let affinity_sum = self.primary_building_affinity
            + self.same_location_travel
            + self.different_location_travel;

        if (affinity_sum - 1.0).abs() > 0.01 {
            return Err(ConfigValidationError::InvalidAffinitySum { sum: affinity_sum });
        }



        Ok(())
    }

    /// Helper method to validate percentage values
    fn validate_percentage(&self, field: &str, value: f64) -> Result<(), ConfigValidationError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(ConfigValidationError::InvalidPercentage {
                field: field.to_string(),
                value,
            });
        }
        Ok(())
    }

    /// Get the building count range as a tuple
    pub fn buildings_per_location(&self) -> (usize, usize) {
        (self.min_buildings_per_location, self.max_buildings_per_location)
    }

    /// Get the room count range as a tuple
    pub fn rooms_per_building(&self) -> (usize, usize) {
        (self.min_rooms_per_building, self.max_rooms_per_building)
    }

    /// Get the output format as an enum-like value
    pub fn get_output_format(&self) -> Result<OutputFormat, String> {
        match self.output_format.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(format!("Unknown output format: {}", self.output_format)),
        }
    }

    /// Calculate the number of night shift users based on building count
    /// Uses a range of 1-3 users per building for varied staffing
    pub fn calculate_night_shift_users(&self) -> usize {
        let total_buildings = self.location_count * 
            ((self.min_buildings_per_location + self.max_buildings_per_location) / 2);
        
        // Calculate average users per building (midpoint of 1-3 range = 2)
        let avg_users_per_building = (night_shift::MIN_USERS_PER_BUILDING + night_shift::MAX_USERS_PER_BUILDING) as f64 / 2.0;
        
        (total_buildings as f64 * avg_users_per_building).round() as usize
    }

    /// Get the night shift user range per building
    /// Returns (min, max) users per building
    pub fn get_night_shift_user_range(&self) -> (usize, usize) {
        (night_shift::MIN_USERS_PER_BUILDING, night_shift::MAX_USERS_PER_BUILDING)
    }

    /// Get night shift hours as a tuple (start_hour, end_hour)
    /// Returns (17, 8) representing 5 PM to 8 AM
    pub fn get_night_shift_hours(&self) -> (u8, u8) {
        (night_shift::START_HOUR, night_shift::END_HOUR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_config_default() {
        let config = SimulationConfig::default();

        assert_eq!(config.user_count, 10_000);
        assert_eq!(config.location_count, 5);
        assert_eq!(config.min_buildings_per_location, 4);
        assert_eq!(config.max_buildings_per_location, 6);
        assert_eq!(config.min_rooms_per_building, 10);
        assert_eq!(config.max_rooms_per_building, 50);
        assert_eq!(config.curious_user_percentage, 0.05);
        assert_eq!(config.cloned_badge_percentage, 0.001);
        assert_eq!(config.primary_building_affinity, 0.7);
        assert_eq!(config.same_location_travel, 0.29);
        assert_eq!(config.different_location_travel, 0.01);
        assert_eq!(config.output_format, "json");
        assert!(config.seed.is_none());
        assert_eq!(config.days, 1);
    }

    #[test]
    fn test_days_cli_parsing() {
        // Test parsing with --days flag
        let args = vec!["test", "--days", "5"];
        let cli_args = CliArgs::try_parse_from(args).unwrap();
        assert_eq!(cli_args.days, 5);
        
        // Test parsing with different values
        let args = vec!["test", "--days", "10"];
        let cli_args = CliArgs::try_parse_from(args).unwrap();
        assert_eq!(cli_args.days, 10);
        
        // Test default value
        let args = vec!["test"];
        let cli_args = CliArgs::try_parse_from(args).unwrap();
        assert_eq!(cli_args.days, 1);
    }

    #[test]
    fn test_days_validation() {
        // Test that days validation works for valid values
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
        
        // Test validation passes for valid days
        config.validate().unwrap();
    }

    #[test]
    fn test_config_file_loading() {
        use std::io::Write;
        use tempfile::Builder;

        // Create a temporary config file with .json extension
        let mut temp_file = Builder::new().suffix(".json").tempfile().unwrap();
        let config_json = r#"{
            "user_count": 5000,
            "location_count": 3,
            "min_buildings_per_location": 2,
            "max_buildings_per_location": 4,
            "min_rooms_per_building": 5,
            "max_rooms_per_building": 25,
            "curious_user_percentage": 0.02,
            "cloned_badge_percentage": 0.001,
            "primary_building_affinity": 0.8,
            "same_location_travel": 0.15,
            "different_location_travel": 0.05,
            "output_format": "csv",
            "seed": 12345,
            "user_profiles_output": "profiles.jsonl"
        }"#;

        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Load configuration from file
        let config = SimulationConfig::from_file(temp_file.path()).unwrap();

        assert_eq!(config.user_count, 5000);
        assert_eq!(config.location_count, 3);
        assert_eq!(config.min_buildings_per_location, 2);
        assert_eq!(config.max_buildings_per_location, 4);
        assert_eq!(config.curious_user_percentage, 0.02);
        assert_eq!(config.output_format, "csv");
        assert_eq!(config.seed, Some(12345));
    }

    #[test]
    fn test_cli_overrides() {
        let args = CliArgs {
            config: None,
            user_count: Some(8000),
            location_count: Some(7),
            min_buildings_per_location: None,
            max_buildings_per_location: None,
            min_rooms_per_building: None,
            max_rooms_per_building: None,
            curious_user_percentage: Some(0.08),
            cloned_badge_percentage: None,
            primary_building_affinity: None,
            same_location_travel: None,
            different_location_travel: None,
            output_format: Some("json".to_string()),
            seed: Some(54321),
            user_profiles_output: None,
            verbose: false,
            debug: false,
            dry_run: false,
            print_config: false,
            include_failure_reason: false,
            include_event_type: false,
            include_metadata: false,
            include_all_fields: false,
            days: 3,
        };

        let config = SimulationConfig::from_cli_args(args).unwrap();

        assert_eq!(config.user_count, 8000);
        assert_eq!(config.location_count, 7);
        assert_eq!(config.curious_user_percentage, 0.08);
        assert_eq!(config.seed, Some(54321));
        assert_eq!(config.days, 3);
        // Default values should remain for non-overridden fields
        assert_eq!(config.min_buildings_per_location, 4);
        assert_eq!(config.max_buildings_per_location, 6);
    }

    #[test]
    fn test_simulation_config_validation_success() {
        let config = SimulationConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_simulation_config_validation_user_count() {
        let mut config = SimulationConfig::default();
        config.user_count = 0;

        match config.validate() {
            Err(ConfigValidationError::InvalidUserCount(0)) => {}
            _ => panic!("Expected InvalidUserCount error"),
        }
    }

    #[test]
    fn test_simulation_config_validation_location_count() {
        let mut config = SimulationConfig::default();
        config.location_count = 0;

        match config.validate() {
            Err(ConfigValidationError::InvalidLocationCount(0)) => {}
            _ => panic!("Expected InvalidLocationCount error"),
        }
    }

    #[test]
    fn test_simulation_config_validation_building_range() {
        let mut config = SimulationConfig::default();
        config.min_buildings_per_location = 10;
        config.max_buildings_per_location = 5;

        match config.validate() {
            Err(ConfigValidationError::InvalidBuildingRange(10, 5)) => {}
            _ => panic!("Expected InvalidBuildingRange error"),
        }
    }

    #[test]
    fn test_simulation_config_validation_room_range() {
        let mut config = SimulationConfig::default();
        config.min_rooms_per_building = 20;
        config.max_rooms_per_building = 10;

        match config.validate() {
            Err(ConfigValidationError::InvalidRoomRange(20, 10)) => {}
            _ => panic!("Expected InvalidRoomRange error"),
        }
    }

    #[test]
    fn test_simulation_config_validation_percentage() {
        let mut config = SimulationConfig::default();
        config.curious_user_percentage = 1.5;

        match config.validate() {
            Err(ConfigValidationError::InvalidPercentage { field, value }) => {
                assert_eq!(field, "curious_user_percentage");
                assert_eq!(value, 1.5);
            }
            _ => panic!("Expected InvalidPercentage error"),
        }
    }

    #[test]
    fn test_simulation_config_validation_affinity_sum() {
        let mut config = SimulationConfig::default();
        config.primary_building_affinity = 0.5;
        config.same_location_travel = 0.3;
        config.different_location_travel = 0.3; // Sum = 1.1

        match config.validate() {
            Err(ConfigValidationError::InvalidAffinitySum { sum }) => {
                assert!((sum - 1.1).abs() < 0.01);
            }
            _ => panic!("Expected InvalidAffinitySum error"),
        }
    }

    #[test]
    fn test_simulation_config_helper_methods() {
        let config = SimulationConfig::default();

        assert_eq!(config.buildings_per_location(), (4, 6));
        assert_eq!(config.rooms_per_building(), (10, 50));
        assert!(config.get_output_format().is_ok());
    }

    #[test]
    fn test_cli_output_field_overrides() {
        // Test individual field flags
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
            include_failure_reason: true,
            include_event_type: false,
            include_metadata: true,
            include_all_fields: false,
            days: 1,
        };

        let config = SimulationConfig::from_cli_args(args).unwrap();

        assert!(config.output_fields.include_failure_reason);
        assert!(!config.output_fields.include_event_type);
        assert!(config.output_fields.include_metadata);
        assert!(!config.output_fields.include_all);
    }

    #[test]
    fn test_cli_include_all_fields_override() {
        // Test that include_all_fields overrides individual settings
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
            include_all_fields: true,
            days: 1,
        };

        let config = SimulationConfig::from_cli_args(args).unwrap();

        // All fields should be enabled when include_all_fields is true
        assert!(config.output_fields.include_failure_reason);
        assert!(config.output_fields.include_event_type);
        assert!(config.output_fields.include_metadata);
        assert!(config.output_fields.include_all);
    }

    #[test]
    fn test_cli_no_output_field_flags_uses_defaults() {
        // Test that when no output field flags are set, defaults are used
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
            days: 1,
        };

        let config = SimulationConfig::from_cli_args(args).unwrap();

        // Should use default values (all false)
        assert!(!config.output_fields.include_failure_reason);
        assert!(!config.output_fields.include_event_type);
        assert!(!config.output_fields.include_metadata);
        assert!(!config.output_fields.include_all);
    }

    #[test]
    fn test_simulation_config_serialization() {
        let config = SimulationConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SimulationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.user_count, deserialized.user_count);
        assert_eq!(config.location_count, deserialized.location_count);
        assert_eq!(config.output_format, deserialized.output_format);
    }

    #[test]
    fn test_config_file_with_output_fields() {
        use std::io::Write;
        use tempfile::Builder;

        // Create a temporary config file with output_fields configuration
        let mut temp_file = Builder::new().suffix(".json").tempfile().unwrap();
        let config_json = r#"{
            "user_count": 1000,
            "output_fields": {
                "include_failure_reason": true,
                "include_event_type": false,
                "include_metadata": true,
                "include_all": false
            }
        }"#;

        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Load configuration from file
        let config = SimulationConfig::from_file(temp_file.path()).unwrap();

        assert_eq!(config.user_count, 1000);
        assert!(config.output_fields.include_failure_reason);
        assert!(!config.output_fields.include_event_type);
        assert!(config.output_fields.include_metadata);
        assert!(!config.output_fields.include_all);
    }

    #[test]
    fn test_config_file_without_output_fields() {
        use std::io::Write;
        use tempfile::Builder;

        // Create a temporary config file without output_fields (backward compatibility)
        let mut temp_file = Builder::new().suffix(".json").tempfile().unwrap();
        let config_json = r#"{
            "user_count": 2000
        }"#;

        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Load configuration from file
        let config = SimulationConfig::from_file(temp_file.path()).unwrap();

        assert_eq!(config.user_count, 2000);
        // Should use default output field configuration
        assert!(!config.output_fields.include_failure_reason);
        assert!(!config.output_fields.include_event_type);
        assert!(!config.output_fields.include_metadata);
        assert!(!config.output_fields.include_all);
    }

    #[test]
    fn test_simulation_config_includes_output_fields() {
        let config = SimulationConfig::default();
        
        // Verify that output_fields is included and has default values
        assert!(!config.output_fields.include_failure_reason);
        assert!(!config.output_fields.include_event_type);
        assert!(!config.output_fields.include_metadata);
        assert!(!config.output_fields.include_all);
    }

    #[test]
    fn test_output_format_parsing() {
        let mut config = SimulationConfig::default();

        config.output_format = "json".to_string();
        assert!(matches!(config.get_output_format().unwrap(), OutputFormat::Json));

        config.output_format = "csv".to_string();
        assert!(matches!(config.get_output_format().unwrap(), OutputFormat::Csv));

        config.output_format = "invalid".to_string();
        assert!(config.get_output_format().is_err());
    }

    #[test]
    fn test_output_field_config_default() {
        let config = OutputFieldConfig::default();

        assert!(!config.include_failure_reason);
        assert!(!config.include_event_type);
        assert!(!config.include_metadata);
        assert!(!config.include_all);
    }

    #[test]
    fn test_output_field_config_serialization() {
        let config = OutputFieldConfig {
            include_failure_reason: true,
            include_event_type: false,
            include_metadata: true,
            include_all: false,
        };

        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"include_failure_reason\":true"));
        assert!(json.contains("\"include_event_type\":false"));
        assert!(json.contains("\"include_metadata\":true"));
        assert!(json.contains("\"include_all\":false"));

        // Test deserialization
        let deserialized: OutputFieldConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.include_failure_reason, deserialized.include_failure_reason);
        assert_eq!(config.include_event_type, deserialized.include_event_type);
        assert_eq!(config.include_metadata, deserialized.include_metadata);
        assert_eq!(config.include_all, deserialized.include_all);
    }

    #[test]
    fn test_output_field_config_default_serialization() {
        let config = OutputFieldConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        
        // Verify all fields are false by default
        assert!(json.contains("\"include_failure_reason\":false"));
        assert!(json.contains("\"include_event_type\":false"));
        assert!(json.contains("\"include_metadata\":false"));
        assert!(json.contains("\"include_all\":false"));

        // Test that we can deserialize back to the same values
        let deserialized: OutputFieldConfig = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.include_failure_reason);
        assert!(!deserialized.include_event_type);
        assert!(!deserialized.include_metadata);
        assert!(!deserialized.include_all);
    }

    #[test]
    fn test_night_shift_constants() {
        use super::night_shift;
        
        // Test hardcoded night shift hours
        assert_eq!(night_shift::START_HOUR, 17); // 5 PM
        assert_eq!(night_shift::END_HOUR, 8);    // 8 AM
        
        // Test user range per building
        assert_eq!(night_shift::MIN_USERS_PER_BUILDING, 1);
        assert_eq!(night_shift::MAX_USERS_PER_BUILDING, 3);
    }

    #[test]
    fn test_calculate_night_shift_users() {
        let config = SimulationConfig {
            location_count: 2,
            min_buildings_per_location: 4,
            max_buildings_per_location: 6,
            ..SimulationConfig::default()
        };

        // Expected calculation: 2 locations * ((4 + 6) / 2) buildings * 2.0 avg_users = 20
        let night_shift_count = config.calculate_night_shift_users();
        assert_eq!(night_shift_count, 20);
    }

    #[test]
    fn test_calculate_night_shift_users_single_location() {
        let config = SimulationConfig {
            location_count: 1,
            min_buildings_per_location: 3,
            max_buildings_per_location: 3,
            ..SimulationConfig::default()
        };

        // Expected calculation: 1 location * 3 buildings * 2.0 avg_users = 6
        let night_shift_count = config.calculate_night_shift_users();
        assert_eq!(night_shift_count, 6);
    }

    #[test]
    fn test_calculate_night_shift_users_rounding() {
        let config = SimulationConfig {
            location_count: 1,
            min_buildings_per_location: 1,
            max_buildings_per_location: 2,
            ..SimulationConfig::default()
        };

        // Expected calculation: 1 location * ((1 + 2) / 2) buildings * 2.0 avg_users
        // = 1 * (3 / 2) * 2.0 = 1 * 1 * 2.0 = 2.0 (integer division)
        let night_shift_count = config.calculate_night_shift_users();
        assert_eq!(night_shift_count, 2);
    }

    #[test]
    fn test_get_night_shift_hours() {
        let config = SimulationConfig::default();
        let (start, end) = config.get_night_shift_hours();
        
        assert_eq!(start, 17); // 5 PM
        assert_eq!(end, 8);    // 8 AM
    }

    #[test]
    fn test_get_night_shift_user_range() {
        let config = SimulationConfig::default();
        let (min, max) = config.get_night_shift_user_range();
        
        assert_eq!(min, 1); // Minimum 1 user per building
        assert_eq!(max, 3); // Maximum 3 users per building
    }
}
