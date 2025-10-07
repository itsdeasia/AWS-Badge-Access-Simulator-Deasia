//! Statistics collection and reporting
//!
//! This module contains statistics collection and reporting functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// Consolidated statistics structure serving as single source of truth for all event counts
/// This replaces the complex multi-layered statistics tracking with a unified approach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedStatistics {
    // Infrastructure stats
    /// Total number of users in the simulation
    pub total_users: usize,
    /// Total number of locations in the simulation
    pub total_locations: usize,
    /// Total number of buildings in the simulation
    pub total_buildings: usize,
    /// Total number of rooms in the simulation
    pub total_rooms: usize,

    // User breakdown
    /// Number of curious users (those who attempt unauthorized access)
    pub curious_users: usize,
    /// Number of users with cloned badges
    pub cloned_badge_users: usize,
    /// Number of night-shift users
    pub night_shift_users: usize,

    // Event statistics (single source of truth)
    /// Total number of events generated across all days
    pub total_events: usize,
    /// Number of successful access events
    pub success_events: usize,
    /// Number of failed access events
    pub failure_events: usize,
    /// Number of curious events (unauthorized access attempts)
    pub curious_events: usize,
    /// Number of impossible traveler events detected
    pub impossible_traveler_events: usize,
    /// Number of night-shift events during off-hours
    pub night_shift_events: usize,
    /// Number of badge reader failure events (backward compatibility field)
    pub badge_reader_failure_events: usize,
    /// Number of invalid badge events (backward compatibility field)
    pub invalid_badge_events: usize,
    /// Number of outside hours events (backward compatibility field)
    pub outside_hours_events: usize,
    /// Number of suspicious events (backward compatibility field)
    pub suspicious_events: usize,

    // Simulation metadata
    /// Number of days simulated
    pub days_simulated: usize,
    /// Total duration of the simulation
    pub simulation_duration: Duration,
}

/// Type alias for backward compatibility during transition to consolidated statistics
/// This allows existing code to continue working while we migrate to the new system
pub type SimulationStatistics = ConsolidatedStatistics;

impl ConsolidatedStatistics {
    /// Create new consolidated statistics with infrastructure and user counts
    pub fn new(
        total_users: usize,
        total_locations: usize,
        total_buildings: usize,
        total_rooms: usize,
        curious_users: usize,
        cloned_badge_users: usize,
        night_shift_users: usize,
    ) -> Self {
        Self {
            total_users,
            total_locations,
            total_buildings,
            total_rooms,
            curious_users,
            cloned_badge_users,
            night_shift_users,
            total_events: 0,
            success_events: 0,
            failure_events: 0,
            curious_events: 0,
            impossible_traveler_events: 0,
            night_shift_events: 0,
            badge_reader_failure_events: 0,
            invalid_badge_events: 0,
            outside_hours_events: 0,
            suspicious_events: 0,
            days_simulated: 0,
            simulation_duration: Duration::from_secs(0),
        }
    }

    // Backward compatibility methods for existing SimulationStatistics interface

    /// Get the average rooms per building
    pub fn average_rooms_per_building(&self) -> f64 {
        if self.total_buildings == 0 {
            0.0
        } else {
            self.total_rooms as f64 / self.total_buildings as f64
        }
    }

    /// Get the average buildings per location
    pub fn average_buildings_per_location(&self) -> f64 {
        if self.total_locations == 0 {
            0.0
        } else {
            self.total_buildings as f64 / self.total_locations as f64
        }
    }

    /// Increment invalid badge event counter (maps to failure events in consolidated model)
    pub fn increment_invalid_badge_events(&mut self) {
        self.invalid_badge_events += 1;
        self.increment_failure_events();
    }

    /// Increment outside hours event counter (maps to failure events in consolidated model)
    pub fn increment_outside_hours_events(&mut self) {
        self.outside_hours_events += 1;
        self.increment_failure_events();
    }

    /// Increment suspicious event counter (maps to failure events in consolidated model)
    pub fn increment_suspicious_events(&mut self) {
        self.suspicious_events += 1;
        self.increment_failure_events();
    }

    /// Increment badge technical failure event counter (maps to failure events in consolidated model)
    pub fn increment_badge_reader_failure_events(&mut self) {
        self.badge_reader_failure_events += 1;
        self.increment_failure_events();
    }

    /// Update event type statistics with a batch of counters (backward compatibility)
    pub fn update_event_type_counters<F>(&mut self, update_fn: F)
    where
        F: FnOnce(&mut Self),
    {
        update_fn(self);
    }

    /// Get a reference to event type statistics (backward compatibility)
    /// Returns self since we are now the consolidated statistics
    pub fn event_type_statistics(&self) -> &Self {
        self
    }

    /// Get a mutable reference to event type statistics (backward compatibility)
    /// Returns self since we are now the consolidated statistics
    pub fn event_type_statistics_mut(&mut self) -> &mut Self {
        self
    }

    /// Generate a summary of event type breakdowns with counts and percentages (backward compatibility)
    pub fn summary(&self) -> String {
        format!(
            "Event Summary: {} total events | Success: {} ({:.1}%) | Failures: {} ({:.1}%) | Anomalies: {} curious ({:.1}%), {} impossible traveler ({:.1}%)",
            self.total_events,
            self.success_events, self.success_percentage(),
            self.failure_events, self.failure_percentage(),
            self.curious_events, self.curious_event_percentage(),
            self.impossible_traveler_events, self.impossible_traveler_percentage()
        )
    }

    /// Generate a detailed breakdown of all event types with counts and percentages (backward compatibility)
    pub fn detailed_breakdown(&self) -> String {
        let mut breakdown = String::new();
        breakdown.push_str(&format!("=== Event Type Breakdown ===\n"));
        breakdown.push_str(&format!("Total Events Generated: {}\n\n", self.total_events));

        breakdown.push_str("Standard Event Types:\n");
        breakdown.push_str(&format!(
            "  • Success Events: {} ({:.1}%)\n",
            self.success_events,
            self.success_percentage()
        ));
        breakdown.push_str(&format!(
            "  • Failure Events: {} ({:.1}%)\n",
            self.failure_events,
            self.failure_percentage()
        ));

        breakdown.push_str("\nSecurity Anomaly Events:\n");
        breakdown.push_str(&format!(
            "  • Curious Events: {} ({:.1}%)\n",
            self.curious_events,
            self.curious_event_percentage()
        ));
        breakdown.push_str(&format!(
            "  • Impossible Traveler Events: {} ({:.1}%)\n",
            self.impossible_traveler_events,
            self.impossible_traveler_percentage()
        ));

        breakdown.push_str("\nAuthorized Off-Hours Events:\n");
        breakdown.push_str(&format!(
            "  • Night-Shift Events: {} ({:.1}%)\n",
            self.night_shift_events,
            self.night_shift_percentage()
        ));

        breakdown
    }

    /// Get the percentage of invalid badge events (backward compatibility)
    pub fn invalid_badge_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.invalid_badge_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of outside hours events (backward compatibility)
    pub fn outside_hours_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.outside_hours_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of suspicious events (backward compatibility)
    pub fn suspicious_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.suspicious_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the total number of failure-type events (backward compatibility)
    pub fn total_failure_events(&self) -> usize {
        self.failure_events
            + self.invalid_badge_events
            + self.outside_hours_events
            + self.suspicious_events
    }

    /// Get the total number of anomaly events (backward compatibility)
    pub fn total_anomaly_events(&self) -> usize {
        self.curious_events + self.impossible_traveler_events + self.badge_reader_failure_events
    }

    /// Get the percentage of failure-type events (backward compatibility)
    pub fn total_failure_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.total_failure_events() as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of anomaly events relative to total events (backward compatibility)
    pub fn total_anomaly_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.total_anomaly_events() as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Generate a compact one-line summary suitable for logging (backward compatibility)
    pub fn compact_summary(&self) -> String {
        self.generate_compact_summary()
    }

    /// Increment success events (backward compatibility for EventTypeStatistics)
    pub fn increment_success(&mut self) {
        self.increment_success_events();
    }

    /// Increment failure events (backward compatibility for EventTypeStatistics)
    pub fn increment_failure(&mut self) {
        self.increment_failure_events();
    }

    /// Increment curious events (backward compatibility for EventTypeStatistics)
    pub fn increment_curious(&mut self) {
        self.increment_curious_events();
    }

    /// Increment impossible traveler events (backward compatibility for EventTypeStatistics)
    pub fn increment_impossible_traveler(&mut self) {
        self.increment_impossible_traveler_events();
    }

    /// Increment night shift events (backward compatibility for EventTypeStatistics)
    pub fn increment_night_shift(&mut self) {
        self.increment_night_shift_events();
    }

    /// Increment invalid badge events (backward compatibility for EventTypeStatistics)
    pub fn increment_invalid_badge(&mut self) {
        self.increment_invalid_badge_events();
    }

    /// Increment outside hours events (backward compatibility for EventTypeStatistics)
    pub fn increment_outside_hours(&mut self) {
        self.increment_outside_hours_events();
    }

    /// Increment suspicious events (backward compatibility for EventTypeStatistics)
    pub fn increment_suspicious(&mut self) {
        self.increment_suspicious_events();
    }

    /// Increment badge reader failure events (backward compatibility for EventTypeStatistics)
    pub fn increment_badge_reader_failure(&mut self) {
        self.increment_badge_reader_failure_events();
    }

    /// Increment the counter for successful access events
    pub fn increment_success_events(&mut self) {
        self.success_events += 1;
        self.total_events += 1;
    }

    /// Increment the counter for failed access events
    pub fn increment_failure_events(&mut self) {
        self.failure_events += 1;
        self.total_events += 1;
    }

    /// Increment the counter for curious events (unauthorized access attempts)
    pub fn increment_curious_events(&mut self) {
        self.curious_events += 1;
    }

    /// Increment the counter for impossible traveler events
    pub fn increment_impossible_traveler_events(&mut self) {
        self.impossible_traveler_events += 1;
    }

    /// Increment the counter for night-shift events
    pub fn increment_night_shift_events(&mut self) {
        self.night_shift_events += 1;
    }

    /// Set the number of days simulated
    pub fn set_days_simulated(&mut self, days: usize) {
        self.days_simulated = days;
    }

    /// Set the simulation duration
    pub fn set_simulation_duration(&mut self, duration: Duration) {
        self.simulation_duration = duration;
    }

    /// Get the percentage of successful events
    pub fn success_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.success_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of failed events
    pub fn failure_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.failure_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of curious events relative to total events
    pub fn curious_event_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.curious_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of impossible traveler events relative to total events
    pub fn impossible_traveler_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.impossible_traveler_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of night-shift events relative to total events
    pub fn night_shift_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.night_shift_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of curious users
    pub fn curious_user_percentage(&self) -> f64 {
        if self.total_users == 0 {
            0.0
        } else {
            (self.curious_users as f64 / self.total_users as f64) * 100.0
        }
    }

    /// Get the percentage of users with cloned badges
    pub fn cloned_badge_percentage(&self) -> f64 {
        if self.total_users == 0 {
            0.0
        } else {
            (self.cloned_badge_users as f64 / self.total_users as f64) * 100.0
        }
    }

    /// Get the percentage of night-shift users
    pub fn night_shift_user_percentage(&self) -> f64 {
        if self.total_users == 0 {
            0.0
        } else {
            (self.night_shift_users as f64 / self.total_users as f64) * 100.0
        }
    }

    /// Get the average events per day
    pub fn average_events_per_day(&self) -> f64 {
        if self.days_simulated == 0 {
            0.0
        } else {
            self.total_events as f64 / self.days_simulated as f64
        }
    }

    /// Get the average impossible traveler events per day
    pub fn average_impossible_traveler_per_day(&self) -> f64 {
        if self.days_simulated == 0 {
            0.0
        } else {
            self.impossible_traveler_events as f64 / self.days_simulated as f64
        }
    }

    /// Get the average curious events per day
    pub fn average_curious_events_per_day(&self) -> f64 {
        if self.days_simulated == 0 {
            0.0
        } else {
            self.curious_events as f64 / self.days_simulated as f64
        }
    }

    /// Get the average night-shift events per day
    pub fn average_night_shift_events_per_day(&self) -> f64 {
        if self.days_simulated == 0 {
            0.0
        } else {
            self.night_shift_events as f64 / self.days_simulated as f64
        }
    }

    /// Generate a comprehensive summary report
    pub fn generate_summary_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Simulation Summary Report ===\n\n");

        // Simulation metadata
        report.push_str(&format!(
            "Simulation Duration: {:.2} seconds\n",
            self.simulation_duration.as_secs_f64()
        ));
        report.push_str(&format!("Days Simulated: {}\n\n", self.days_simulated));

        // Infrastructure overview
        report.push_str("Infrastructure Overview:\n");
        report.push_str(&format!("  • Total Users: {}\n", self.total_users));
        report.push_str(&format!("  • Total Locations: {}\n", self.total_locations));
        report.push_str(&format!("  • Total Buildings: {}\n", self.total_buildings));
        report.push_str(&format!("  • Total Rooms: {}\n\n", self.total_rooms));

        // User breakdown
        report.push_str("User Breakdown:\n");
        report.push_str(&format!(
            "  • Curious Users: {} ({:.1}%)\n",
            self.curious_users,
            self.curious_user_percentage()
        ));
        report.push_str(&format!(
            "  • Cloned Badge Users: {} ({:.1}%)\n",
            self.cloned_badge_users,
            self.cloned_badge_percentage()
        ));
        report.push_str(&format!(
            "  • Night-Shift Users: {} ({:.1}%)\n\n",
            self.night_shift_users,
            self.night_shift_user_percentage()
        ));

        // Event statistics
        report.push_str("Event Statistics:\n");
        report.push_str(&format!(
            "  • Total Events: {} (avg {:.1}/day)\n",
            self.total_events,
            self.average_events_per_day()
        ));
        report.push_str(&format!(
            "  • Success Events: {} ({:.1}%)\n",
            self.success_events,
            self.success_percentage()
        ));
        report.push_str(&format!(
            "  • Failure Events: {} ({:.1}%)\n\n",
            self.failure_events,
            self.failure_percentage()
        ));

        // Anomaly detection
        report.push_str("Security Anomalies:\n");
        report.push_str(&format!(
            "  • Curious Events: {} ({:.1}%, avg {:.1}/day)\n",
            self.curious_events,
            self.curious_event_percentage(),
            self.average_curious_events_per_day()
        ));
        report.push_str(&format!(
            "  • Impossible Traveler Events: {} ({:.1}%, avg {:.1}/day)\n",
            self.impossible_traveler_events,
            self.impossible_traveler_percentage(),
            self.average_impossible_traveler_per_day()
        ));
        report.push_str(&format!(
            "  • Night-Shift Events: {} ({:.1}%, avg {:.1}/day)\n",
            self.night_shift_events,
            self.night_shift_percentage(),
            self.average_night_shift_events_per_day()
        ));

        report
    }

    /// Generate simplified statistics output for batch processing
    /// 
    /// This method creates a clear, readable statistics report without duplication,
    /// focusing on the key metrics required for the simplified batch system.
    /// Addresses requirements 3.1, 3.2, 3.3, 3.4, 3.5 for consolidated statistics.
    pub fn generate_simplified_statistics_output(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str("🎯 Badge Access Simulation Complete!\n");
        output.push_str("=====================================\n\n");

        // Simulation metadata (Requirements 3.4, 3.5)
        output.push_str("📊 Simulation Summary:\n");
        output.push_str(&format!("   Days Simulated: {}\n", self.days_simulated));
        output.push_str(&format!(
            "   Duration: {:.2} seconds\n",
            self.simulation_duration.as_secs_f64()
        ));
        output.push_str(&format!(
            "   Infrastructure: {} users across {} locations ({} buildings, {} rooms)\n\n",
            self.total_users, self.total_locations, self.total_buildings, self.total_rooms
        ));

        // Total events and daily averages (Requirements 3.1, 3.2)
        output.push_str("📈 Event Statistics:\n");
        output.push_str(&format!("   Total Events Generated: {}\n", self.total_events));
        if self.days_simulated > 0 {
            output.push_str(&format!(
                "   Daily Average: {:.1} events/day\n",
                self.average_events_per_day()
            ));
        }
        output.push_str(&format!(
            "   Success Rate: {:.1}% ({} successful, {} failed)\n\n",
            self.success_percentage(),
            self.success_events,
            self.failure_events
        ));

        // Security anomalies (Requirements 3.1, 3.3)
        output.push_str("🚨 Security Anomalies Detected:\n");
        output.push_str(&format!(
            "   Impossible Traveler Events: {} ({:.1}%",
            self.impossible_traveler_events,
            self.impossible_traveler_percentage()
        ));
        if self.days_simulated > 0 {
            output.push_str(&format!(", avg {:.1}/day", self.average_impossible_traveler_per_day()));
        }
        output.push_str(")\n");

        output.push_str(&format!(
            "   Curious User Events: {} ({:.1}%",
            self.curious_events,
            self.curious_event_percentage()
        ));
        if self.days_simulated > 0 {
            output.push_str(&format!(", avg {:.1}/day", self.average_curious_events_per_day()));
        }
        output.push_str(")\n");

        output.push_str(&format!(
            "   Night-Shift Events: {} ({:.1}%",
            self.night_shift_events,
            self.night_shift_percentage()
        ));
        if self.days_simulated > 0 {
            output.push_str(&format!(", avg {:.1}/day", self.average_night_shift_events_per_day()));
        }
        output.push_str(")\n\n");

        // User breakdown summary
        output.push_str("👥 User Profile:\n");
        output.push_str(&format!(
            "   {} curious users ({:.1}%), {} with cloned badges ({:.1}%), {} night-shift ({:.1}%)\n\n",
            self.curious_users,
            self.curious_user_percentage(),
            self.cloned_badge_users,
            self.cloned_badge_percentage(),
            self.night_shift_users,
            self.night_shift_user_percentage()
        ));

        // Performance summary
        if self.simulation_duration.as_secs_f64() > 0.0 {
            let events_per_second = self.total_events as f64 / self.simulation_duration.as_secs_f64();
            output.push_str("⚡ Performance:\n");
            output.push_str(&format!("   Generated {:.0} events/second\n\n", events_per_second));
        }

        // One-line summary for easy parsing (Requirement 3.3)
        output.push_str("💡 Summary: ");
        output.push_str(&self.generate_compact_summary());
        output.push_str("\n");

        output
    }

    /// Generate a compact one-line summary suitable for logging
    pub fn generate_compact_summary(&self) -> String {
        format!(
            "Simulation: {} days, {} events ({} success, {} failures), {} anomalies ({} curious, {} impossible traveler)",
            self.days_simulated,
            self.total_events,
            self.success_events,
            self.failure_events,
            self.curious_events + self.impossible_traveler_events,
            self.curious_events,
            self.impossible_traveler_events
        )
    }
}

impl Default for ConsolidatedStatistics {
    fn default() -> Self {
        Self {
            total_users: 0,
            total_locations: 0,
            total_buildings: 0,
            total_rooms: 0,
            curious_users: 0,
            cloned_badge_users: 0,
            night_shift_users: 0,
            total_events: 0,
            success_events: 0,
            failure_events: 0,
            curious_events: 0,
            impossible_traveler_events: 0,
            night_shift_events: 0,
            badge_reader_failure_events: 0,
            invalid_badge_events: 0,
            outside_hours_events: 0,
            suspicious_events: 0,
            days_simulated: 0,
            simulation_duration: Duration::from_secs(0),
        }
    }
}

impl fmt::Display for ConsolidatedStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.generate_summary_report())
    }
}

/// Detailed statistics about event types generated during simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeStatistics {
    /// Total number of events generated
    pub total_events: usize,
    /// Count of successful access events
    pub success_events: usize,
    /// Count of failed access events
    pub failure_events: usize,
    /// Count of invalid badge events
    pub invalid_badge_events: usize,
    /// Count of outside hours events
    pub outside_hours_events: usize,
    /// Count of suspicious events
    pub suspicious_events: usize,
    /// Count of curious user unauthorized access attempts
    pub curious_events: usize,
    /// Count of impossible traveler event pairs
    pub impossible_traveler_events: usize,
    /// Count of badge technical failures
    pub badge_reader_failure_events: usize,
    /// Count of night-shift user events during off-hours
    pub night_shift_events: usize,
}

impl EventTypeStatistics {
    /// Create new event type statistics with all counters initialized to zero
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the counter for successful access events
    pub fn increment_success(&mut self) {
        self.success_events += 1;
        self.total_events += 1;
    }

    /// Increment the counter for failed access events
    pub fn increment_failure(&mut self) {
        self.failure_events += 1;
        self.total_events += 1;
    }

    /// Increment the counter for invalid badge events
    pub fn increment_invalid_badge(&mut self) {
        self.invalid_badge_events += 1;
        self.total_events += 1;
    }

    /// Increment the counter for outside hours events
    pub fn increment_outside_hours(&mut self) {
        self.outside_hours_events += 1;
        self.total_events += 1;
    }

    /// Increment the counter for suspicious events
    pub fn increment_suspicious(&mut self) {
        self.suspicious_events += 1;
        self.total_events += 1;
    }

    /// Increment the counter for curious events (unauthorized access attempts)
    pub fn increment_curious(&mut self) {
        self.curious_events += 1;
    }

    /// Increment the counter for impossible traveler events
    pub fn increment_impossible_traveler(&mut self) {
        self.impossible_traveler_events += 1;
    }

    /// Increment the counter for badge technical failure events
    pub fn increment_badge_reader_failure(&mut self) {
        self.badge_reader_failure_events += 1;
    }

    /// Increment the counter for night-shift events
    pub fn increment_night_shift(&mut self) {
        self.night_shift_events += 1;
    }

    /// Get the percentage of successful events
    pub fn success_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.success_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of failed events
    pub fn failure_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.failure_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of invalid badge events
    pub fn invalid_badge_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.invalid_badge_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of outside hours events
    pub fn outside_hours_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.outside_hours_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of suspicious events
    pub fn suspicious_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.suspicious_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of curious events relative to total events
    pub fn curious_event_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.curious_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of impossible traveler events relative to total events
    pub fn impossible_traveler_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.impossible_traveler_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of badge technical failure events relative to total events
    pub fn badge_reader_failure_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.badge_reader_failure_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of night-shift events relative to total events
    pub fn night_shift_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.night_shift_events as f64 / self.total_events as f64) * 100.0
        }
    }
}

impl Default for EventTypeStatistics {
    fn default() -> Self {
        Self {
            total_events: 0,
            success_events: 0,
            failure_events: 0,
            invalid_badge_events: 0,
            outside_hours_events: 0,
            suspicious_events: 0,
            curious_events: 0,
            impossible_traveler_events: 0,
            badge_reader_failure_events: 0,
            night_shift_events: 0,
        }
    }
}

impl EventTypeStatistics {
    /// Generate a summary of event type breakdowns with counts and percentages
    pub fn summary(&self) -> String {
        format!(
            "Event Summary: {} total events | Success: {} ({:.1}%) | Failures: {} ({:.1}%) | Anomalies: {} curious ({:.1}%), {} impossible traveler ({:.1}%)",
            self.total_events,
            self.success_events, self.success_percentage(),
            self.failure_events + self.invalid_badge_events + self.outside_hours_events + self.suspicious_events,
            self.failure_percentage() + self.invalid_badge_percentage() + self.outside_hours_percentage() + self.suspicious_percentage(),
            self.curious_events, self.curious_event_percentage(),
            self.impossible_traveler_events, self.impossible_traveler_percentage()
        )
    }

    /// Generate a detailed breakdown of all event types with counts and percentages
    pub fn detailed_breakdown(&self) -> String {
        let mut breakdown = String::new();
        breakdown.push_str(&format!("=== Event Type Breakdown ===\n"));
        breakdown.push_str(&format!("Total Events Generated: {}\n\n", self.total_events));

        breakdown.push_str("Standard Event Types:\n");
        breakdown.push_str(&format!(
            "  • Success Events: {} ({:.1}%)\n",
            self.success_events,
            self.success_percentage()
        ));
        breakdown.push_str(&format!(
            "  • Failure Events: {} ({:.1}%)\n",
            self.failure_events,
            self.failure_percentage()
        ));
        breakdown.push_str(&format!(
            "  • Invalid Badge Events: {} ({:.1}%)\n",
            self.invalid_badge_events,
            self.invalid_badge_percentage()
        ));
        breakdown.push_str(&format!(
            "  • Outside Hours Events: {} ({:.1}%)\n",
            self.outside_hours_events,
            self.outside_hours_percentage()
        ));
        breakdown.push_str(&format!(
            "  • Suspicious Events: {} ({:.1}%)\n",
            self.suspicious_events,
            self.suspicious_percentage()
        ));

        breakdown.push_str("\nSecurity Anomaly Events:\n");
        breakdown.push_str(&format!(
            "  • Curious Events: {} ({:.1}%)\n",
            self.curious_events,
            self.curious_event_percentage()
        ));
        breakdown.push_str(&format!(
            "  • Impossible Traveler Events: {} ({:.1}%)\n",
            self.impossible_traveler_events,
            self.impossible_traveler_percentage()
        ));
        breakdown.push_str(&format!(
            "  • Badge Technical Failures: {} ({:.1}%)\n",
            self.badge_reader_failure_events,
            self.badge_reader_failure_percentage()
        ));

        breakdown.push_str("\nAuthorized Off-Hours Events:\n");
        breakdown.push_str(&format!(
            "  • Night-Shift Events: {} ({:.1}%)\n",
            self.night_shift_events,
            self.night_shift_percentage()
        ));

        breakdown
    }

    /// Generate a compact one-line summary suitable for logging
    pub fn compact_summary(&self) -> String {
        format!(
            "Events: {} total, {} success ({:.1}%), {} failures ({:.1}%), {} curious ({:.1}%), {} impossible traveler ({:.1}%)",
            self.total_events,
            self.success_events, self.success_percentage(),
            self.failure_events + self.invalid_badge_events + self.outside_hours_events + self.suspicious_events,
            self.failure_percentage() + self.invalid_badge_percentage() + self.outside_hours_percentage() + self.suspicious_percentage(),
            self.curious_events, self.curious_event_percentage(),
            self.impossible_traveler_events, self.impossible_traveler_percentage()
        )
    }

    /// Format event counts with appropriate labels for display
    pub fn format_event_counts(&self) -> Vec<(String, usize, f64)> {
        vec![
            ("Success Events".to_string(), self.success_events, self.success_percentage()),
            ("Failure Events".to_string(), self.failure_events, self.failure_percentage()),
            (
                "Invalid Badge Events".to_string(),
                self.invalid_badge_events,
                self.invalid_badge_percentage(),
            ),
            (
                "Outside Hours Events".to_string(),
                self.outside_hours_events,
                self.outside_hours_percentage(),
            ),
            ("Suspicious Events".to_string(), self.suspicious_events, self.suspicious_percentage()),
            ("Curious Events".to_string(), self.curious_events, self.curious_event_percentage()),
            (
                "Impossible Traveler Events".to_string(),
                self.impossible_traveler_events,
                self.impossible_traveler_percentage(),
            ),
            (
                "Badge Technical Failures".to_string(),
                self.badge_reader_failure_events,
                self.badge_reader_failure_percentage(),
            ),
            (
                "Night-Shift Events".to_string(),
                self.night_shift_events,
                self.night_shift_percentage(),
            ),
        ]
    }

    /// Get the total number of failure-type events (all non-success events)
    pub fn total_failure_events(&self) -> usize {
        self.failure_events
            + self.invalid_badge_events
            + self.outside_hours_events
            + self.suspicious_events
    }

    /// Get the total number of anomaly events (curious + impossible traveler + badge technical failures)
    pub fn total_anomaly_events(&self) -> usize {
        self.curious_events + self.impossible_traveler_events + self.badge_reader_failure_events
    }

    /// Get the percentage of failure-type events
    pub fn total_failure_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.total_failure_events() as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the percentage of anomaly events relative to total events
    pub fn total_anomaly_percentage(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.total_anomaly_events() as f64 / self.total_events as f64) * 100.0
        }
    }
}

impl fmt::Display for EventTypeStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Event Type Statistics:")?;
        writeln!(f, "  Total Events: {}", self.total_events)?;
        writeln!(
            f,
            "  Success Events: {} ({:.1}%)",
            self.success_events,
            self.success_percentage()
        )?;
        writeln!(
            f,
            "  Failure Events: {} ({:.1}%)",
            self.failure_events,
            self.failure_percentage()
        )?;
        writeln!(
            f,
            "  Invalid Badge Events: {} ({:.1}%)",
            self.invalid_badge_events,
            self.invalid_badge_percentage()
        )?;
        writeln!(
            f,
            "  Outside Hours Events: {} ({:.1}%)",
            self.outside_hours_events,
            self.outside_hours_percentage()
        )?;
        writeln!(
            f,
            "  Suspicious Events: {} ({:.1}%)",
            self.suspicious_events,
            self.suspicious_percentage()
        )?;
        writeln!(
            f,
            "  Curious Events: {} ({:.1}%)",
            self.curious_events,
            self.curious_event_percentage()
        )?;
        writeln!(
            f,
            "  Impossible Traveler Events: {} ({:.1}%)",
            self.impossible_traveler_events,
            self.impossible_traveler_percentage()
        )?;
        writeln!(
            f,
            "  Badge Technical Failures: {} ({:.1}%)",
            self.badge_reader_failure_events,
            self.badge_reader_failure_percentage()
        )?;
        write!(
            f,
            "  Night-Shift Events: {} ({:.1}%)",
            self.night_shift_events,
            self.night_shift_percentage()
        )
    }
}

/// Runtime statistics about the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatistics {
    /// Current simulated time
    pub current_simulated_time: DateTime<Utc>,
    /// Number of users currently engaged in activities
    pub users_with_current_activities: usize,
    /// Number of users currently idle
    pub users_idle: usize,
    /// How long the simulation has been running in real time
    pub simulation_uptime: Duration,
}

impl RuntimeStatistics {
    /// Create new runtime statistics
    pub fn new(
        current_simulated_time: DateTime<Utc>,
        users_with_current_activities: usize,
        users_idle: usize,
        simulation_uptime: Duration,
    ) -> Self {
        Self {
            current_simulated_time,
            users_with_current_activities,
            users_idle,
            simulation_uptime,
        }
    }

    /// Get the total number of users
    pub fn total_users(&self) -> usize {
        self.users_with_current_activities + self.users_idle
    }

    /// Get the percentage of users currently active
    pub fn active_user_percentage(&self) -> f64 {
        let total = self.total_users();
        if total == 0 {
            0.0
        } else {
            (self.users_with_current_activities as f64 / total as f64) * 100.0
        }
    }

    /// Get the percentage of users currently idle
    pub fn idle_user_percentage(&self) -> f64 {
        let total = self.total_users();
        if total == 0 {
            0.0
        } else {
            (self.users_idle as f64 / total as f64) * 100.0
        }
    }

    /// Get simulation uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.simulation_uptime.as_secs()
    }

    /// Get simulation uptime in minutes
    pub fn uptime_minutes(&self) -> f64 {
        self.simulation_uptime.as_secs_f64() / 60.0
    }

    /// Get simulation uptime in hours
    pub fn uptime_hours(&self) -> f64 {
        self.simulation_uptime.as_secs_f64() / 3600.0
    }
}

impl Default for RuntimeStatistics {
    fn default() -> Self {
        Self {
            current_simulated_time: Utc::now(),
            users_with_current_activities: 0,
            users_idle: 0,
            simulation_uptime: Duration::from_secs(0),
        }
    }
}

/// Statistics about access complexity and patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessComplexityStats {
    /// Total number of access attempts
    pub total_access_attempts: usize,
    /// Number of successful access attempts
    pub successful_attempts: usize,
    /// Number of failed access attempts
    pub failed_attempts: usize,
    /// Number of unauthorized access attempts
    pub unauthorized_attempts: usize,
    /// Average access attempts per user per day
    pub avg_attempts_per_user_per_day: f64,
    /// Most accessed room type
    pub most_accessed_room_type: String,
    /// Peak access hour (0-23)
    pub peak_access_hour: u8,
}

impl AccessComplexityStats {
    /// Create new access complexity statistics
    pub fn new() -> Self {
        Self {
            total_access_attempts: 0,
            successful_attempts: 0,
            failed_attempts: 0,
            unauthorized_attempts: 0,
            avg_attempts_per_user_per_day: 0.0,
            most_accessed_room_type: "Unknown".to_string(),
            peak_access_hour: 9, // Default to 9 AM
        }
    }

    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_access_attempts == 0 {
            0.0
        } else {
            (self.successful_attempts as f64 / self.total_access_attempts as f64) * 100.0
        }
    }

    /// Get the failure rate as a percentage
    pub fn failure_rate(&self) -> f64 {
        if self.total_access_attempts == 0 {
            0.0
        } else {
            (self.failed_attempts as f64 / self.total_access_attempts as f64) * 100.0
        }
    }

    /// Get the unauthorized attempt rate as a percentage
    pub fn unauthorized_rate(&self) -> f64 {
        if self.total_access_attempts == 0 {
            0.0
        } else {
            (self.unauthorized_attempts as f64 / self.total_access_attempts as f64) * 100.0
        }
    }

    /// Update statistics with a new access attempt
    pub fn record_access_attempt(&mut self, success: bool, unauthorized: bool) {
        self.total_access_attempts += 1;

        if success {
            self.successful_attempts += 1;
        } else {
            self.failed_attempts += 1;
        }

        if unauthorized {
            self.unauthorized_attempts += 1;
        }
    }

    /// Update the most accessed room type
    pub fn update_most_accessed_room_type(&mut self, room_type: String) {
        self.most_accessed_room_type = room_type;
    }

    /// Update the peak access hour
    pub fn update_peak_access_hour(&mut self, hour: u8) {
        if hour < 24 {
            self.peak_access_hour = hour;
        }
    }

    /// Update average attempts per user per day
    pub fn update_avg_attempts_per_user_per_day(&mut self, avg: f64) {
        self.avg_attempts_per_user_per_day = avg;
    }
}

impl Default for AccessComplexityStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;

    #[test]
    fn test_consolidated_statistics_creation() {
        let stats = ConsolidatedStatistics::new(
            100, // users
            5,   // locations
            20,  // buildings
            500, // rooms
            10,  // curious users
            5,   // cloned badge users
            3,   // night-shift users
        );

        assert_eq!(stats.total_users, 100);
        assert_eq!(stats.total_locations, 5);
        assert_eq!(stats.total_buildings, 20);
        assert_eq!(stats.total_rooms, 500);
        assert_eq!(stats.curious_users, 10);
        assert_eq!(stats.cloned_badge_users, 5);
        assert_eq!(stats.night_shift_users, 3);
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.success_events, 0);
        assert_eq!(stats.failure_events, 0);
        assert_eq!(stats.curious_events, 0);
        assert_eq!(stats.impossible_traveler_events, 0);
        assert_eq!(stats.night_shift_events, 0);
        assert_eq!(stats.days_simulated, 0);
        assert_eq!(stats.simulation_duration, Duration::from_secs(0));
    }

    #[test]
    fn test_consolidated_statistics_event_incrementing() {
        let mut stats = ConsolidatedStatistics::new(100, 5, 20, 500, 10, 5, 3);

        // Test event incrementing
        stats.increment_success_events();
        stats.increment_success_events();
        stats.increment_failure_events();
        stats.increment_curious_events();
        stats.increment_impossible_traveler_events();
        stats.increment_night_shift_events();

        assert_eq!(stats.total_events, 3); // success and failure events count toward total
        assert_eq!(stats.success_events, 2);
        assert_eq!(stats.failure_events, 1);
        assert_eq!(stats.curious_events, 1);
        assert_eq!(stats.impossible_traveler_events, 1);
        assert_eq!(stats.night_shift_events, 1);
    }

    #[test]
    fn test_consolidated_statistics_percentages() {
        let mut stats = ConsolidatedStatistics::new(100, 5, 20, 500, 10, 5, 3);

        // Add some events
        stats.increment_success_events();
        stats.increment_success_events();
        stats.increment_failure_events();
        stats.increment_curious_events();
        stats.increment_impossible_traveler_events();

        assert!((stats.success_percentage() - 66.66666666666666).abs() < 0.0001); // 2/3 * 100
        assert!((stats.failure_percentage() - 33.33333333333333).abs() < 0.0001); // 1/3 * 100
        assert_eq!(stats.curious_user_percentage(), 10.0); // 10/100 * 100
        assert_eq!(stats.cloned_badge_percentage(), 5.0); // 5/100 * 100
        assert_eq!(stats.night_shift_user_percentage(), 3.0); // 3/100 * 100
    }

    #[test]
    fn test_consolidated_statistics_daily_averages() {
        let mut stats = ConsolidatedStatistics::new(100, 5, 20, 500, 10, 5, 3);

        // Set simulation metadata
        stats.set_days_simulated(2);
        stats.set_simulation_duration(Duration::from_secs(120));

        // Add events
        for _ in 0..10 {
            stats.increment_success_events();
        }
        for _ in 0..4 {
            stats.increment_curious_events();
        }
        for _ in 0..2 {
            stats.increment_impossible_traveler_events();
        }
        for _ in 0..6 {
            stats.increment_night_shift_events();
        }

        assert_eq!(stats.average_events_per_day(), 5.0); // 10 total events / 2 days
        assert_eq!(stats.average_curious_events_per_day(), 2.0); // 4 curious events / 2 days
        assert_eq!(stats.average_impossible_traveler_per_day(), 1.0); // 2 impossible traveler / 2 days
        assert_eq!(stats.average_night_shift_events_per_day(), 3.0); // 6 night shift / 2 days
    }

    #[test]
    fn test_consolidated_statistics_zero_division() {
        let stats = ConsolidatedStatistics::default();

        // Test that percentages handle zero division gracefully
        assert_eq!(stats.success_percentage(), 0.0);
        assert_eq!(stats.failure_percentage(), 0.0);
        assert_eq!(stats.curious_user_percentage(), 0.0);
        assert_eq!(stats.average_events_per_day(), 0.0);
        assert_eq!(stats.average_curious_events_per_day(), 0.0);
    }

    #[test]
    fn test_consolidated_statistics_summary_generation() {
        let mut stats = ConsolidatedStatistics::new(100, 5, 20, 500, 10, 5, 3);
        stats.set_days_simulated(1);
        stats.set_simulation_duration(Duration::from_secs(60));
        stats.increment_success_events();
        stats.increment_failure_events();
        stats.increment_curious_events();

        let summary = stats.generate_summary_report();
        assert!(summary.contains("=== Simulation Summary Report ==="));
        assert!(summary.contains("Days Simulated: 1"));
        assert!(summary.contains("Total Events: 2"));
        assert!(summary.contains("Success Events: 1"));
        assert!(summary.contains("Failure Events: 1"));
        assert!(summary.contains("Curious Events: 1"));

        let compact = stats.generate_compact_summary();
        assert!(compact.contains("1 days"));
        assert!(compact.contains("2 events"));
        assert!(compact.contains("1 success"));
        assert!(compact.contains("1 failures"));
    }

    #[test]
    fn test_consolidated_statistics_display() {
        let mut stats = ConsolidatedStatistics::new(50, 3, 10, 200, 5, 2, 1);
        stats.set_days_simulated(1);
        stats.increment_success_events();

        let display_output = format!("{}", stats);
        assert!(display_output.contains("=== Simulation Summary Report ==="));
        assert!(display_output.contains("Total Users: 50"));
    }

    #[test]
    fn test_simulation_statistics_creation() {
        let stats = SimulationStatistics::new(
            100, // users
            5,   // locations
            20,  // buildings
            500, // rooms
            10,  // curious users
            5,   // cloned badge users
            3,   // night-shift users
        );

        assert_eq!(stats.total_users, 100);
        assert_eq!(stats.total_locations, 5);
        assert_eq!(stats.curious_users, 10);
        // Event type stats are always enabled now
        assert_eq!(stats.event_type_statistics().total_events, 0);
    }

    #[test]
    fn test_simulation_statistics_percentages() {
        let stats = SimulationStatistics::new(100, 5, 20, 500, 10, 5, 3);

        assert_eq!(stats.curious_user_percentage(), 10.0);
        assert_eq!(stats.cloned_badge_percentage(), 5.0);
        assert_eq!(stats.average_rooms_per_building(), 25.0);
        assert_eq!(stats.average_buildings_per_location(), 4.0);
    }

    #[test]
    fn test_simulation_statistics_zero_division() {
        let stats = SimulationStatistics::default();

        assert_eq!(stats.curious_user_percentage(), 0.0);
        assert_eq!(stats.cloned_badge_percentage(), 0.0);
        assert_eq!(stats.average_rooms_per_building(), 0.0);
        assert_eq!(stats.average_buildings_per_location(), 0.0);
    }

    #[test]
    fn test_runtime_statistics_creation() {
        let now = Utc::now();
        let uptime = Duration::from_secs(3600); // 1 hour

        let stats = RuntimeStatistics::new(
            now, 75, // active users
            25, // idle users
            uptime,
        );

        assert_eq!(stats.current_simulated_time, now);
        assert_eq!(stats.users_with_current_activities, 75);
        assert_eq!(stats.users_idle, 25);
        assert_eq!(stats.simulation_uptime, uptime);
    }

    #[test]
    fn test_runtime_statistics_calculations() {
        let stats = RuntimeStatistics::new(
            Utc::now(),
            75,                        // active
            25,                        // idle
            Duration::from_secs(7200), // 2 hours
        );

        assert_eq!(stats.total_users(), 100);
        assert_eq!(stats.active_user_percentage(), 75.0);
        assert_eq!(stats.idle_user_percentage(), 25.0);
        assert_eq!(stats.uptime_seconds(), 7200);
        assert_eq!(stats.uptime_minutes(), 120.0);
        assert_eq!(stats.uptime_hours(), 2.0);
    }

    #[test]
    fn test_access_complexity_stats_creation() {
        let stats = AccessComplexityStats::new();

        assert_eq!(stats.total_access_attempts, 0);
        assert_eq!(stats.successful_attempts, 0);
        assert_eq!(stats.failed_attempts, 0);
        assert_eq!(stats.unauthorized_attempts, 0);
        assert_eq!(stats.peak_access_hour, 9);
    }

    #[test]
    fn test_access_complexity_stats_recording() {
        let mut stats = AccessComplexityStats::new();

        // Record some access attempts
        stats.record_access_attempt(true, false); // successful, authorized
        stats.record_access_attempt(false, false); // failed, authorized
        stats.record_access_attempt(false, true); // failed, unauthorized

        assert_eq!(stats.total_access_attempts, 3);
        assert_eq!(stats.successful_attempts, 1);
        assert_eq!(stats.failed_attempts, 2);
        assert_eq!(stats.unauthorized_attempts, 1);
    }

    #[test]
    fn test_access_complexity_stats_rates() {
        let mut stats = AccessComplexityStats::new();

        stats.record_access_attempt(true, false);
        stats.record_access_attempt(true, false);
        stats.record_access_attempt(false, true);
        stats.record_access_attempt(false, false);

        assert_eq!(stats.success_rate(), 50.0);
        assert_eq!(stats.failure_rate(), 50.0);
        assert_eq!(stats.unauthorized_rate(), 25.0);
    }

    #[test]
    fn test_access_complexity_stats_updates() {
        let mut stats = AccessComplexityStats::new();

        stats.update_most_accessed_room_type("Meeting Room".to_string());
        stats.update_peak_access_hour(14);
        stats.update_avg_attempts_per_user_per_day(12.5);

        assert_eq!(stats.most_accessed_room_type, "Meeting Room");
        assert_eq!(stats.peak_access_hour, 14);
        assert_eq!(stats.avg_attempts_per_user_per_day, 12.5);
    }

    #[test]
    fn test_access_complexity_stats_invalid_hour() {
        let mut stats = AccessComplexityStats::new();
        let original_hour = stats.peak_access_hour;

        stats.update_peak_access_hour(25); // Invalid hour
        assert_eq!(stats.peak_access_hour, original_hour); // Should remain unchanged
    }

    #[test]
    fn test_event_type_statistics_creation() {
        let stats = EventTypeStatistics::new();

        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.success_events, 0);
        assert_eq!(stats.failure_events, 0);
        assert_eq!(stats.invalid_badge_events, 0);
        assert_eq!(stats.outside_hours_events, 0);
        assert_eq!(stats.suspicious_events, 0);
        assert_eq!(stats.curious_events, 0);
        assert_eq!(stats.impossible_traveler_events, 0);
        assert_eq!(stats.night_shift_events, 0);
    }

    #[test]
    fn test_event_type_statistics_default() {
        let stats = EventTypeStatistics::default();

        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.success_events, 0);
        assert_eq!(stats.failure_events, 0);
        assert_eq!(stats.invalid_badge_events, 0);
        assert_eq!(stats.outside_hours_events, 0);
        assert_eq!(stats.suspicious_events, 0);
        assert_eq!(stats.curious_events, 0);
        assert_eq!(stats.impossible_traveler_events, 0);
        assert_eq!(stats.night_shift_events, 0);
    }

    #[test]
    fn test_event_type_statistics_increment_methods() {
        let mut stats = EventTypeStatistics::new();

        // Test increment_success
        stats.increment_success();
        assert_eq!(stats.success_events, 1);
        assert_eq!(stats.total_events, 1);

        // Test increment_failure
        stats.increment_failure();
        assert_eq!(stats.failure_events, 1);
        assert_eq!(stats.total_events, 2);

        // Test increment_invalid_badge
        stats.increment_invalid_badge();
        assert_eq!(stats.invalid_badge_events, 1);
        assert_eq!(stats.total_events, 3);

        // Test increment_outside_hours
        stats.increment_outside_hours();
        assert_eq!(stats.outside_hours_events, 1);
        assert_eq!(stats.total_events, 4);

        // Test increment_suspicious
        stats.increment_suspicious();
        assert_eq!(stats.suspicious_events, 1);
        assert_eq!(stats.total_events, 5);

        // Test increment_curious (doesn't increment total_events)
        stats.increment_curious();
        assert_eq!(stats.curious_events, 1);
        assert_eq!(stats.total_events, 5);

        // Test increment_impossible_traveler (doesn't increment total_events)
        stats.increment_impossible_traveler();
        assert_eq!(stats.impossible_traveler_events, 1);
        assert_eq!(stats.total_events, 5);

        // Test increment_night_shift (doesn't increment total_events)
        stats.increment_night_shift();
        assert_eq!(stats.night_shift_events, 1);
        assert_eq!(stats.total_events, 5);
    }

    #[test]
    fn test_event_type_statistics_percentage_calculations() {
        let mut stats = EventTypeStatistics::new();

        // Add some events
        stats.increment_success();
        stats.increment_success();
        stats.increment_failure();
        stats.increment_invalid_badge();
        stats.increment_outside_hours();
        stats.increment_suspicious();
        stats.increment_curious();
        stats.increment_curious();
        stats.increment_impossible_traveler();

        // Total events should be 6 (curious and impossible traveler don't count toward total)
        assert_eq!(stats.total_events, 6);

        // Test percentage calculations (using approximate equality for floating point)
        assert!((stats.success_percentage() - 33.333333333333336).abs() < 0.0001);
        // 2/6 * 100
    }

    #[test]
    fn test_night_shift_event_statistics() {
        let mut stats = EventTypeStatistics::new();

        // Add some regular events
        stats.increment_success();
        stats.increment_failure();
        stats.increment_outside_hours();

        // Add night-shift events (should not count toward total_events)
        stats.increment_night_shift();
        stats.increment_night_shift();

        // Total events should only include regular events, not night-shift
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.night_shift_events, 2);

        // Test percentage calculation
        assert!((stats.night_shift_percentage() - 66.66666666666667).abs() < 0.0001);
        // 2/3 * 100
    }

    #[test]
    fn test_simulation_statistics_night_shift_increment() {
        let mut stats = SimulationStatistics::new(100, 5, 20, 500, 10, 5, 3);

        stats.increment_night_shift_events();
        stats.increment_night_shift_events();

        assert_eq!(stats.event_type_statistics().night_shift_events, 2);
        assert_eq!(stats.event_type_statistics().total_events, 0); // Night-shift events don't count toward total
    }

    #[test]
    fn test_event_type_statistics_display_includes_night_shift() {
        let mut stats = EventTypeStatistics::new();
        stats.increment_success();
        stats.increment_night_shift();

        let display_string = format!("{}", stats);
        assert!(display_string.contains("Night-Shift Events: 1"));
    }

    #[test]
    fn test_event_type_statistics_detailed_breakdown_includes_night_shift() {
        let mut stats = EventTypeStatistics::new();
        stats.increment_success();
        stats.increment_night_shift();

        let breakdown = stats.detailed_breakdown();
        assert!(breakdown.contains("Authorized Off-Hours Events:"));
        assert!(breakdown.contains("Night-Shift Events: 1"));
    }

    #[test]
    fn test_event_type_statistics_format_event_counts_includes_night_shift() {
        let mut stats = EventTypeStatistics::new();
        stats.increment_night_shift();

        let event_counts = stats.format_event_counts();
        let night_shift_entry = event_counts
            .iter()
            .find(|(name, _, _)| name == "Night-Shift Events")
            .expect("Night-Shift Events should be in the list");

        assert_eq!(night_shift_entry.1, 1); // count
        assert!((night_shift_entry.2 - 0.0).abs() < 0.0001); // percentage (0% since total_events is 0)

        // Since only night-shift events were added (which don't count toward total_events),
        // all other percentages should be 0.0
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.failure_percentage(), 0.0);
        assert_eq!(stats.invalid_badge_percentage(), 0.0);
        assert_eq!(stats.outside_hours_percentage(), 0.0);
        assert_eq!(stats.suspicious_percentage(), 0.0);
        assert_eq!(stats.curious_event_percentage(), 0.0);
        assert_eq!(stats.impossible_traveler_percentage(), 0.0);
    }

    #[test]
    fn test_event_type_statistics_zero_division() {
        let stats = EventTypeStatistics::new();

        // All percentages should be 0.0 when total_events is 0
        assert_eq!(stats.success_percentage(), 0.0);
        assert_eq!(stats.failure_percentage(), 0.0);
        assert_eq!(stats.invalid_badge_percentage(), 0.0);
        assert_eq!(stats.outside_hours_percentage(), 0.0);
        assert_eq!(stats.suspicious_percentage(), 0.0);
        assert_eq!(stats.curious_event_percentage(), 0.0);
        assert_eq!(stats.impossible_traveler_percentage(), 0.0);
    }

    #[test]
    fn test_event_type_statistics_multiple_increments() {
        let mut stats = EventTypeStatistics::new();

        // Test multiple increments of the same type
        for _ in 0..5 {
            stats.increment_success();
        }
        assert_eq!(stats.success_events, 5);
        assert_eq!(stats.total_events, 5);

        for _ in 0..3 {
            stats.increment_curious();
        }
        assert_eq!(stats.curious_events, 3);
        assert_eq!(stats.total_events, 5); // Should not change

        for _ in 0..2 {
            stats.increment_impossible_traveler();
        }
        assert_eq!(stats.impossible_traveler_events, 2);
        assert_eq!(stats.total_events, 5); // Should not change
    }

    #[test]
    fn test_event_type_statistics_display() {
        let mut stats = EventTypeStatistics::new();

        stats.increment_success();
        stats.increment_failure();
        stats.increment_curious();
        stats.increment_impossible_traveler();

        let display_output = format!("{}", stats);

        // Check that the display contains expected information
        assert!(display_output.contains("Event Type Statistics:"));
        assert!(display_output.contains("Total Events: 2"));
        assert!(display_output.contains("Success Events: 1 (50.0%)"));
        assert!(display_output.contains("Failure Events: 1 (50.0%)"));
        assert!(display_output.contains("Curious Events: 1 (50.0%)"));
        assert!(display_output.contains("Impossible Traveler Events: 1 (50.0%)"));
    }

    #[test]
    fn test_event_type_statistics_serialization() {
        let mut stats = EventTypeStatistics::new();
        stats.increment_success();
        stats.increment_curious();

        // Test serialization to JSON
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: EventTypeStatistics = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.total_events, deserialized.total_events);
        assert_eq!(stats.success_events, deserialized.success_events);
        assert_eq!(stats.curious_events, deserialized.curious_events);
    }

    #[test]
    fn test_simulation_statistics_event_counter_methods() {
        let mut stats = SimulationStatistics::new(100, 5, 20, 500, 10, 5, 3);

        // Test all counter increment methods
        stats.increment_success_events();
        stats.increment_success_events();
        stats.increment_failure_events();
        stats.increment_invalid_badge_events();
        stats.increment_outside_hours_events();
        stats.increment_suspicious_events();
        stats.increment_curious_events();
        stats.increment_impossible_traveler_events();

        let event_stats = stats.event_type_statistics();
        assert_eq!(event_stats.total_events, 6); // curious and impossible traveler don't count toward total
        assert_eq!(event_stats.success_events, 2);
        assert_eq!(event_stats.failure_events, 4); // 1 direct + 3 indirect (from invalid_badge, outside_hours, suspicious)
        assert_eq!(event_stats.invalid_badge_events, 1);
        assert_eq!(event_stats.outside_hours_events, 1);
        assert_eq!(event_stats.suspicious_events, 1);
        assert_eq!(event_stats.curious_events, 1);
        assert_eq!(event_stats.impossible_traveler_events, 1);
    }

    #[test]
    fn test_simulation_statistics_mutable_reference() {
        let mut stats = SimulationStatistics::new(100, 5, 20, 500, 10, 5, 3);

        // Test mutable reference access
        let event_stats = stats.event_type_statistics_mut();
        event_stats.increment_success();
        event_stats.increment_curious();

        let event_stats = stats.event_type_statistics();
        assert_eq!(event_stats.success_events, 1);
        assert_eq!(event_stats.curious_events, 1);
        assert_eq!(event_stats.total_events, 1);
    }

    #[test]
    fn test_simulation_statistics_update_event_type_counters() {
        let mut stats = SimulationStatistics::new(100, 5, 20, 500, 10, 5, 3);

        // Test batch update using closure
        stats.update_event_type_counters(|event_stats| {
            event_stats.increment_success();
            event_stats.increment_failure();
            event_stats.increment_curious();
        });

        let event_stats = stats.event_type_statistics();
        assert_eq!(event_stats.success_events, 1);
        assert_eq!(event_stats.failure_events, 1);
        assert_eq!(event_stats.curious_events, 1);
        assert_eq!(event_stats.total_events, 2);
    }

    #[test]
    fn test_event_type_statistics_summary() {
        let mut stats = EventTypeStatistics::new();

        // Add some test data
        stats.increment_success();
        stats.increment_success();
        stats.increment_failure();
        stats.increment_curious();
        stats.increment_impossible_traveler();

        let summary = stats.summary();

        // Check that summary contains expected information
        assert!(summary.contains("3 total events"));
        assert!(summary.contains("Success: 2 (66.7%)"));
        assert!(summary.contains("Failures: 1 (33.3%)"));
        assert!(summary.contains("1 curious (33.3%)"));
        assert!(summary.contains("1 impossible traveler (33.3%)"));
    }

    #[test]
    fn test_event_type_statistics_detailed_breakdown() {
        let mut stats = EventTypeStatistics::new();

        // Add comprehensive test data
        stats.increment_success();
        stats.increment_failure();
        stats.increment_invalid_badge();
        stats.increment_outside_hours();
        stats.increment_suspicious();
        stats.increment_curious();
        stats.increment_impossible_traveler();

        let breakdown = stats.detailed_breakdown();

        // Check that breakdown contains expected sections and data
        assert!(breakdown.contains("=== Event Type Breakdown ==="));
        assert!(breakdown.contains("Total Events Generated: 5"));
        assert!(breakdown.contains("Standard Event Types:"));
        assert!(breakdown.contains("Security Anomaly Events:"));
        assert!(breakdown.contains("• Success Events: 1 (20.0%)"));
        assert!(breakdown.contains("• Failure Events: 1 (20.0%)"));
        assert!(breakdown.contains("• Invalid Badge Events: 1 (20.0%)"));
        assert!(breakdown.contains("• Outside Hours Events: 1 (20.0%)"));
        assert!(breakdown.contains("• Suspicious Events: 1 (20.0%)"));
        assert!(breakdown.contains("• Curious Events: 1 (20.0%)"));
        assert!(breakdown.contains("• Impossible Traveler Events: 1 (20.0%)"));
    }

    #[test]
    fn test_event_type_statistics_compact_summary() {
        let mut stats = EventTypeStatistics::new();

        stats.increment_success();
        stats.increment_success();
        stats.increment_failure();
        stats.increment_curious();

        let compact = stats.compact_summary();

        // Check that compact summary contains expected information in one line
        assert!(compact.contains("3 total"));
        assert!(compact.contains("2 success (66.7%)"));
        assert!(compact.contains("1 failures (33.3%)"));
        assert!(compact.contains("1 curious (33.3%)"));
        assert!(compact.contains("0 impossible traveler (0.0%)"));
    }

    #[test]
    fn test_event_type_statistics_format_event_counts() {
        let mut stats = EventTypeStatistics::new();

        stats.increment_success();
        stats.increment_failure();
        stats.increment_curious();

        let formatted_counts = stats.format_event_counts();

        // Should return 9 tuples (one for each event type including badge reader failures and night-shift events)
        assert_eq!(formatted_counts.len(), 9);

        // Check specific entries
        assert_eq!(formatted_counts[0], ("Success Events".to_string(), 1, 50.0));
        assert_eq!(formatted_counts[1], ("Failure Events".to_string(), 1, 50.0));
        assert_eq!(formatted_counts[2], ("Invalid Badge Events".to_string(), 0, 0.0));
        assert_eq!(formatted_counts[5], ("Curious Events".to_string(), 1, 50.0));
    }

    #[test]
    fn test_event_type_statistics_total_failure_events() {
        let mut stats = EventTypeStatistics::new();

        stats.increment_failure();
        stats.increment_invalid_badge();
        stats.increment_outside_hours();
        stats.increment_suspicious();
        stats.increment_curious(); // Should not count toward failures

        assert_eq!(stats.total_failure_events(), 4);
        assert_eq!(stats.total_failure_percentage(), 100.0); // 4/4 * 100
    }

    #[test]
    fn test_event_type_statistics_total_anomaly_events() {
        let mut stats = EventTypeStatistics::new();

        stats.increment_success();
        stats.increment_curious();
        stats.increment_curious();
        stats.increment_impossible_traveler();

        assert_eq!(stats.total_anomaly_events(), 3);
        assert_eq!(stats.total_anomaly_percentage(), 300.0); // 3/1 * 100 (only success counts toward total)
    }

    #[test]
    fn test_event_type_statistics_zero_events_formatting() {
        let stats = EventTypeStatistics::new();

        // Test all formatting methods with zero events
        let summary = stats.summary();
        let breakdown = stats.detailed_breakdown();
        let compact = stats.compact_summary();
        let formatted_counts = stats.format_event_counts();

        assert!(summary.contains("0 total events"));
        assert!(breakdown.contains("Total Events Generated: 0"));
        assert!(compact.contains("0 total"));
        assert_eq!(formatted_counts.len(), 9);

        // All percentages should be 0.0
        assert_eq!(stats.total_failure_percentage(), 0.0);
        assert_eq!(stats.total_anomaly_percentage(), 0.0);
    }

    #[test]
    fn test_event_type_statistics_display_formatting() {
        let mut stats = EventTypeStatistics::new();

        // Add varied test data
        stats.increment_success();
        stats.increment_success();
        stats.increment_success();
        stats.increment_failure();
        stats.increment_invalid_badge();
        stats.increment_curious();
        stats.increment_impossible_traveler();

        let display_output = format!("{}", stats);

        // Check that display output is properly formatted
        assert!(display_output.contains("Event Type Statistics:"));
        assert!(display_output.contains("Total Events: 5"));
        assert!(display_output.contains("Success Events: 3 (60.0%)"));
        assert!(display_output.contains("Failure Events: 1 (20.0%)"));
        assert!(display_output.contains("Invalid Badge Events: 1 (20.0%)"));
        assert!(display_output.contains("Outside Hours Events: 0 (0.0%)"));
        assert!(display_output.contains("Suspicious Events: 0 (0.0%)"));
        assert!(display_output.contains("Curious Events: 1 (20.0%)"));
        assert!(display_output.contains("Impossible Traveler Events: 1 (20.0%)"));

        // Check that percentages are formatted to 1 decimal place
        assert!(display_output.contains("60.0%"));
        assert!(display_output.contains("20.0%"));
        assert!(display_output.contains("0.0%"));
    }

    #[test]
    fn test_event_type_statistics_edge_case_percentages() {
        let mut stats = EventTypeStatistics::new();

        // Test with only anomaly events (which don't count toward total)
        stats.increment_curious();
        stats.increment_impossible_traveler();

        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.curious_event_percentage(), 0.0); // Should be 0 when total_events is 0
        assert_eq!(stats.impossible_traveler_percentage(), 0.0);

        // Now add one regular event
        stats.increment_success();

        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.curious_event_percentage(), 100.0); // 1/1 * 100
        assert_eq!(stats.impossible_traveler_percentage(), 100.0); // 1/1 * 100
    }

    #[test]
    fn test_event_type_statistics_large_numbers() {
        let mut stats = EventTypeStatistics::new();

        // Test with large numbers to ensure no overflow
        for _ in 0..10000 {
            stats.increment_success();
        }
        for _ in 0..5000 {
            stats.increment_failure();
        }
        for _ in 0..1000 {
            stats.increment_curious();
        }

        assert_eq!(stats.total_events, 15000);
        assert_eq!(stats.success_events, 10000);
        assert_eq!(stats.failure_events, 5000);
        assert_eq!(stats.curious_events, 1000);

        // Test percentage calculations with large numbers
        assert!((stats.success_percentage() - 66.66666666666667).abs() < 0.0001);
        assert!((stats.failure_percentage() - 33.333333333333336).abs() < 0.0001);
        assert!((stats.curious_event_percentage() - 6.666666666666667).abs() < 0.0001);
    }

    #[test]
    fn test_simulation_statistics_serialization() {
        let mut stats = SimulationStatistics::new(100, 5, 20, 500, 10, 5, 3);
        stats.increment_success_events();
        stats.increment_curious_events();

        // Test serialization to JSON
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: SimulationStatistics = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.total_users, deserialized.total_users);

        let original = stats.event_type_statistics();
        let deserialized_stats = deserialized.event_type_statistics();
        assert_eq!(original.success_events, deserialized_stats.success_events);
        assert_eq!(original.curious_events, deserialized_stats.curious_events);
        assert_eq!(original.total_events, deserialized_stats.total_events);
    }
}
