// Badge Access Simulator - Main Entry Point (updated with Part B analysis)
//
// You can run it via Cargo:
//
// $ cargo build --release
// $ ./target/release/amzn-detection-engineering-challenge-rust
//
// Or with custom configuration:
//
// $ ./target/release/amzn-detection-engineering-challenge-rust --user-count 5000 --location-count 5 --verbose
//

mod analysis;
use analysis::{detect_cloned_badges, detect_curious_users, detect_night_shift_users, generate_report, simulate_s3_upload, UserAnomaly};

use amzn_career_pathway_activity_rust::user::UserGenerator;
use amzn_career_pathway_activity_rust::facility::FacilityGenerator;
use amzn_career_pathway_activity_rust::simulation::{
    BatchEventGenerator, LoggingConfig, SimulationOrchestrator, SimulationStatistics,
};
use amzn_career_pathway_activity_rust::types::config::CliArgs;
use amzn_career_pathway_activity_rust::types::SimulationConfig;
use clap::Parser;
use std::process;
use tracing::{error, info};

fn main() {
    // Parse CLI arguments first to check for special flags
    let args = CliArgs::parse();

    // Handle special CLI flags that don't require full initialization
    if args.print_config {
        let default_config = SimulationConfig::default();
        match default_config.print_json() {
            Ok(json) => {
                println!("{}", json);
                return;
            }
            Err(e) => {
                eprintln!("Failed to serialize default configuration: {}", e);
                process::exit(1);
            }
        }
    }

    // Initialize logging based on CLI flags
    let logging_result = if args.debug {
        LoggingConfig::init_debug()
    } else if args.verbose {
        LoggingConfig::init_verbose()
    } else {
        // Default: minimal logging for normal users
        LoggingConfig::new().with_level(tracing::Level::WARN).init()
    };

    if let Err(e) = logging_result {
        eprintln!("Failed to initialize logging: {}", e);
        process::exit(1);
    }

    info!("Starting Badge Access Simulator");

    // Load configuration from CLI arguments and optional config file
    let config = match SimulationConfig::from_cli_args(args.clone()) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Validate configuration
    if let Err(e) = config.validate() {
        error!("Configuration validation failed: {}", e);
        process::exit(1);
    }

    info!("Configuration loaded and validated successfully");

    // Handle dry run mode
    if args.dry_run {
        eprintln!("Configuration validation successful!");
        eprintln!("Dry run mode - simulation will not be executed.");
        print_configuration_summary(&config);
        return;
    }

    // Print startup banner and configuration
    print_startup_banner(&config);

    // Initialize the simulation system
    let (orchestrator, location_registry, users) =
        match initialize_simulation(config.clone()) {
            Ok(components) => components,
            Err(e) => {
                error!("Failed to initialize simulation: {}", e);
                process::exit(1);
            }
        };

    // Run the simulation
    info!("Starting simulation");
    // NOTE: pass clones so we keep ownership of `users` and `location_registry` for post-analysis
    if let Err(e) =
        run_simulation(config.clone(), location_registry.clone(), users.clone(), orchestrator)
    {
        error!("Simulation failed: {}", e);
        process::exit(1);
    }

    info!("Badge Access Simulator completed successfully");

    // --- Part B: Run custom analysis ---
   
info!("Starting anomaly analysis on generated users");

    // Path to JSON, same as what we just generated
let user_profiles_path = config
    .user_profiles_output
    .clone()
    .unwrap_or_else(|| "user_profiles.json".to_string());

// Check that the file exists before analyzing
if !std::path::Path::new(&user_profiles_path).exists() {
    error!("User profile JSON not found at {}. Generate it first.", user_profiles_path);
    process::exit(1);
}

    // Detect anomalies from JSON
    let mut all_anomalies: Vec<UserAnomaly> = Vec::new();

    // Run cloned-badge detection
    let cloned_badges = detect_cloned_badges(&user_profiles_path);
    info!("Detected {} cloned-badge anomalies", cloned_badges.len());
    all_anomalies.extend(cloned_badges);

    // Run curious/unauthorized-access detection
    let curious_users = detect_curious_users(&user_profiles_path);
    info!(
        "Detected {} curious/unauthorized-access anomalies",
        curious_users.len()
    );
    all_anomalies.extend(curious_users);
    let night_shift_users = detect_night_shift_users(&user_profiles_path);
    info!("Detected {} night-shift anomalies", night_shift_users.len());
    
    all_anomalies.extend(night_shift_users);
    // Generate a JSON report file (anomaly_report.json) in project root
    let report_path = "anomaly_report.json";
    generate_report(report_path, &all_anomalies);
    simulate_s3_upload("anomaly_report.json", "security-anomaly-logs-demo");
    info!(" Analysis complete! Report saved to {}", report_path);

    
}

/// Initialize the complete simulation system
fn initialize_simulation(
    config: SimulationConfig,
) -> Result<
    (
        SimulationOrchestrator,
        amzn_career_pathway_activity_rust::facility::LocationRegistry,
        Vec<amzn_career_pathway_activity_rust::user::User>,
    ),
    String,
> {
    info!("Initializing simulation components...");

    // Generate facilities (locations, buildings, rooms)
    eprintln!("Generating facilities...");
    let mut facility_generator = FacilityGenerator::new();
    let location_registry = facility_generator
        .generate_facilities(&config)
        .map_err(|e| format!("Failed to generate facilities: {}", e))?;

    info!(
        "Generated {} locations with {} total buildings and {} total rooms",
        location_registry.location_count(),
        location_registry.total_building_count(),
        location_registry.total_room_count()
    );

    // Generate users with permissions
    eprintln!("Generating users...");
    let mut user_generator = UserGenerator::new();
    let users = user_generator
        .generate_users(&config, &location_registry)
        .map_err(|e| format!("Failed to generate users: {}", e))?;

    info!(
        "Generated {} users ({} curious, {} with cloned badges)",
        users.len(),
        users.iter().filter(|e| e.is_curious).count(),
        users.iter().filter(|e| e.has_cloned_badge).count()
    );

    // Create simulation orchestrator
    eprintln!("Initializing simulation orchestrator...");
    let mut orchestrator = SimulationOrchestrator::new(config.clone())
        .map_err(|e| format!("Failed to create orchestrator: {}", e))?;

    // Initialize orchestrator with generated data for enhanced statistics tracking
    orchestrator
        .initialize_with_data(location_registry.clone(), users.clone())
        .map_err(|e| format!("Failed to initialize orchestrator with data: {}", e))?;

    // Print initial statistics
    let stats = orchestrator.get_statistics();
    info!(
        "Orchestrator initialized with enhanced statistics: {} users, {} locations, {} buildings, {} rooms",
        stats.total_users, stats.total_locations, stats.total_buildings, stats.total_rooms
    );
    info!(
        "User breakdown: {} curious ({}%), {} with cloned badges ({}%), {} night-shift ({}%)",
        stats.curious_users,
        stats.curious_user_percentage(),
        stats.cloned_badge_users,
        stats.cloned_badge_percentage(),
        stats.night_shift_users,
        stats.night_shift_user_percentage()
    );

    // Always generate user profiles JSON, defaulting to "user_profiles.json" if not set
let user_profiles_path = config
    .user_profiles_output
    .clone()
    .unwrap_or_else(|| "user_profiles.json".to_string());

eprintln!("Generating user profiles output at {}", user_profiles_path);
if let Err(e) = generate_user_profiles_output(&config, &users, &user_profiles_path) {
    error!("Failed to generate user profiles output: {}", e);
    return Err(format!(
        "Failed to generate user profiles output: {}",
        e
    ));
}
info!("User profiles successfully written to: {}", user_profiles_path);
eprintln!("User profiles successfully written to: {}", user_profiles_path);

    // Print actual configuration summary with real statistics
    eprintln!("\nActual Generation Results:");
    print_configuration_summary_with_stats(&config, Some(&stats));

    info!("Simulation initialization completed successfully");
    Ok((orchestrator, location_registry, users))
}

/// Run the simulation using batch event generation
fn run_simulation(
    config: SimulationConfig,
    location_registry: amzn_career_pathway_activity_rust::facility::LocationRegistry,
    users: Vec<amzn_career_pathway_activity_rust::user::User>,
    _orchestrator: SimulationOrchestrator,
) -> Result<(), String> {
    use std::time::Instant;

    // Record start time for statistics
    let start_time = Instant::now();

    info!("Running batch simulation for {} days", config.days);
    // Create batch event generator
    eprintln!("Initializing batch event generator...");
    let mut batch_generator = BatchEventGenerator::new(config.clone(), location_registry, users);
    // Generate events for the specified number of days
    eprintln!("Generating events for {} days...", config.days);
    batch_generator
        .generate_events_for_days(config.days)
        .map_err(|e| format!("Batch event generation failed: {}", e))?;
    eprintln!("Batch event generation completed!");

    // Get final statistics from the batch generator
    let mut final_statistics = batch_generator.get_statistics().clone();
    // Update simulation duration in statistics
    final_statistics.set_simulation_duration(start_time.elapsed());
    // Print simplified final statistics
    print_simplified_final_statistics(&final_statistics);

    Ok(())
}

/// Print startup banner and configuration summary
fn print_startup_banner(config: &SimulationConfig) {
    eprintln!("Badge Access Simulator");
    eprintln!("======================");
    eprintln!("A realistic user badge access event simulation system");
    eprintln!();

    print_configuration_summary(config);
}

/// Print configuration summary
fn print_configuration_summary(config: &SimulationConfig) {
    print_configuration_summary_with_stats(config, None);
}

/// Print configuration summary with optional actual statistics
fn print_configuration_summary_with_stats(
    config: &SimulationConfig,
    stats: Option<&SimulationStatistics>,
) {
    eprintln!("Configuration:");
    eprintln!(" User Count: {}", config.user_count);
    eprintln!(" Location Count: {}", config.location_count);
    eprintln!(
        " Buildings per Location: {} - {}",
        config.min_buildings_per_location, config.max_buildings_per_location
    );
    eprintln!(
        " Rooms per Building: {} - {}",
        config.min_rooms_per_building, config.max_rooms_per_building
    );
    eprintln!(" Curious User %: {:.1}%", config.curious_user_percentage * 100.0);
    eprintln!(" Badge Replication %: {:.2}%", config.cloned_badge_percentage * 100.0);
    eprintln!(
        " Primary Building Affinity: {:.1}%",
        config.primary_building_affinity * 100.0
    );
    eprintln!(" Same Location Travel: {:.1}%", config.same_location_travel * 100.0);
    eprintln!(
        " Cross Location Travel: {:.1}%",
        config.different_location_travel * 100.0
    );
    eprintln!(" Output Format: {}", config.output_format);
    if let Some(seed) = config.seed {
        eprintln!(" Random Seed: {}", seed);
    }

    if let Some(stats) = stats {
        eprintln!("\nActual Scale:");
        eprintln!(" Total Buildings: {}", stats.total_buildings);
        eprintln!(" Total Rooms: {}", stats.total_rooms);
        eprintln!(" Curious Users: {}", stats.curious_users);
        eprintln!(" Cloned Badge Users: {}", stats.cloned_badge_users);
        eprintln!(" Night-Shift Users: {}", stats.night_shift_users);
    } else {
        eprintln!("\nEstimated Scale:");
        let avg_buildings =
            (config.min_buildings_per_location + config.max_buildings_per_location) / 2;
        let avg_rooms = (config.min_rooms_per_building + config.max_rooms_per_building) / 2;
        eprintln!(" Total Buildings: ~{}", config.location_count * avg_buildings);
        eprintln!(" Total Rooms: ~{}", config.location_count * avg_buildings * avg_rooms);
        eprintln!(
            " Curious Users: ~{}",
            (config.user_count as f64 * config.curious_user_percentage) as usize
        );
        eprintln!(
            " Cloned Badge Users: ~{}",
            (config.user_count as f64 * config.cloned_badge_percentage) as usize
        );
        eprintln!(
            " Night-Shift Users: ~{}",
            config.calculate_night_shift_users()
        );
    }
    eprintln!();
}

/// Print simulation statistics
#[allow(dead_code)]
fn print_simulation_statistics(
    stats: &amzn_career_pathway_activity_rust::simulation::SimulationStatistics,
) {
    eprintln!("Simulation Statistics:");
    eprintln!(" Total Users: {}", stats.total_users);
    eprintln!(" Total Locations: {}", stats.total_locations);
    eprintln!(" Total Buildings: {}", stats.total_buildings);
    eprintln!(" Total Rooms: {}", stats.total_rooms);
    eprintln!(" Curious Users: {}", stats.curious_users);
    eprintln!(" Cloned Badge Users: {}", stats.cloned_badge_users);
    eprintln!();
}

/// Print simplified final statistics using consolidated statistics
///
/// This function outputs the simplified statistics report as specified in task 11,
/// showing total events, daily averages, impossible traveler events, curious events,
/// night-shift events in a clear, readable manner without duplication.
fn print_simplified_final_statistics(statistics: &SimulationStatistics) {
    // Use the new simplified statistics output method
    eprintln!("{}", statistics.generate_simplified_statistics_output());
}

/// Generate user profiles output file in JSONL format
///
/// This creates the "answer key" file containing ground truth information
/// about each user's permissions, behavior, and characteristics.
fn generate_user_profiles_output(
    config: &SimulationConfig,
    users: &[amzn_career_pathway_activity_rust::user::User],
    output_path: &str,
) -> Result<(), String> {
    use amzn_career_pathway_activity_rust::user::UserProfile;
    use std::fs::File;
    use std::io::{BufWriter, Write};

    info!("Generating user profiles output to: {}", output_path);

    // Create output file
    let file = File::create(output_path).map_err(|e| {
        format!(
            "Failed to create user profiles output file '{}': {}",
            output_path, e
        )
    })?;
    let mut writer = BufWriter::new(file);

    // Generate user profile for each user and write as JSONL
    for user in users {
        let user_profile = UserProfile::from_user(user, config);
        // Serialize to JSON
        let json_line = serde_json::to_string(&user_profile).map_err(|e| {
            format!("Failed to serialize user profile for user {}: {}", user.id, e)
        })?;
        // Write JSON line
        writeln!(writer, "{}", json_line)
            .map_err(|e| format!("Failed to write user profile line: {}", e))?;
    }

    // Ensure all data is written
    writer
        .flush()
        .map_err(|e| format!("Failed to flush user profiles output: {}", e))?;

    info!("Successfully wrote {} user profiles to {}", users.len(), output_path);
    Ok(())
}

/// Print final statistics with enhanced event tracking from orchestrator (legacy)
///
/// This function is kept for backward compatibility but should be replaced
/// with print_simplified_final_statistics for the batch processing system.
#[allow(dead_code)]
fn print_final_statistics_with_orchestrator(
    _config: &SimulationConfig,
    elapsed: std::time::Duration,
    orchestrator: &SimulationOrchestrator,
) {
    eprintln!("\n Simulation Complete!");
    eprintln!("========================");
    eprintln!(
        " Runtime: {:.2} seconds ({:.1} minutes)",
        elapsed.as_secs_f64(),
        elapsed.as_secs_f64() / 60.0
    );
    eprintln!();

    // Print enhanced statistics from orchestrator
    let stats = orchestrator.get_statistics();
    eprintln!(" Infrastructure Statistics:");
    eprintln!("=============================");
    eprintln!(" Total Users: {}", stats.total_users);
    eprintln!(
        " Curious Users: {} ({:.1}%)",
        stats.curious_users,
        stats.curious_user_percentage()
    );
    eprintln!(
        " Cloned Badge Users: {} ({:.1}%)",
        stats.cloned_badge_users,
        stats.cloned_badge_percentage()
    );
    eprintln!(
        " Night-Shift Users: {} ({:.1}%)",
        stats.night_shift_users,
        stats.night_shift_user_percentage()
    );
    eprintln!(" Total Locations: {}", stats.total_locations);
    eprintln!(" Total Buildings: {}", stats.total_buildings);
    eprintln!(" Total Rooms: {}", stats.total_rooms);
    eprintln!(
        " Average Buildings per Location: {:.1}",
        stats.average_buildings_per_location()
    );
    eprintln!(
        " Average Rooms per Building: {:.1}",
        stats.average_rooms_per_building()
    );
    eprintln!();

    // Print detailed event statistics
    let event_stats = stats.event_type_statistics();
    if event_stats.total_events > 0 {
        eprintln!("Event Type Statistics:");
        eprintln!("=========================");
        eprintln!("Total Events Generated: {}", event_stats.total_events);
        eprintln!();

        eprintln!("Standard Access Events:");
        eprintln!(
            " Success Events: {} ({:.1}%)",
            event_stats.success_events,
            event_stats.success_percentage()
        );
        eprintln!(
            " Failure Events: {} ({:.1}%)",
            event_stats.failure_events,
            event_stats.failure_percentage()
        );
        eprintln!(
            " Invalid Badge Events: {} ({:.1}%)",
            event_stats.invalid_badge_events,
            event_stats.invalid_badge_percentage()
        );
        eprintln!(
            " Outside Hours Events: {} ({:.1}%)",
            event_stats.outside_hours_events,
            event_stats.outside_hours_percentage()
        );
        eprintln!(
            " Suspicious Events: {} ({:.1}%)",
            event_stats.suspicious_events,
            event_stats.suspicious_percentage()
        );
        eprintln!();

        eprintln!("Security Anomaly Events:");
        eprintln!(
            " Curious Events: {} ({:.1}%)",
            event_stats.curious_events,
            event_stats.curious_event_percentage()
        );
        eprintln!(
            " Impossible Traveler Events: {} ({:.1}%)",
            event_stats.impossible_traveler_events,
            event_stats.impossible_traveler_percentage()
        );
        eprintln!();

        eprintln!("Authorized Off-Hours Events:");
        eprintln!(
            " Night-Shift Events: {} ({:.1}%)",
            event_stats.night_shift_events,
            event_stats.night_shift_percentage()
        );
        eprintln!();

        // Print additional analysis
        let total_failures = event_stats.total_failure_events();
        let total_anomalies = event_stats.total_anomaly_events();

        eprintln!("Analysis Summary:");
        eprintln!(
            " Total Failure Events: {} ({:.1}%)",
            total_failures,
            event_stats.total_failure_percentage()
        );
        eprintln!(
            " Total Anomaly Events: {} ({:.1}%)",
            total_anomalies,
            event_stats.total_anomaly_percentage()
        );
        eprintln!(" Success Rate: {:.1}%", event_stats.success_percentage());
        eprintln!();

        // Print detailed breakdown using the statistics module's method
        eprintln!("Detailed Event Breakdown:");
        eprintln!("{}", event_stats.detailed_breakdown());

        // Print one-line summary
        eprintln!("Quick Summary: {}", event_stats.compact_summary());
    } else {
        eprintln!(" Event Statistics: No events tracked during this simulation run");
        eprintln!(" This may indicate the simulation ended before events were generated.");
    }
    eprintln!();

    // Print performance metrics
    if elapsed.as_secs_f64() > 0.0 {
        let actual_events_per_second = event_stats.total_events as f64 / elapsed.as_secs_f64();
        eprintln!("Performance Metrics:");
        eprintln!("======================");
        eprintln!(
            " Actual Events per Second: {:.2}",
            actual_events_per_second
        );
        eprintln!();
    }
}