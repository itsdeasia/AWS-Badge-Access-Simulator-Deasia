//! Main simulation orchestrator
//!
//! This module contains the SimulationOrchestrator and main simulation control logic.

use crate::user::User;
use crate::events::EventGenerator;
use crate::facility::LocationRegistry;
use crate::simulation::{
    BehaviorEngine, ErrorHandler, RuntimeStatistics, SimulationResult, SimulationStatistics,
    TimeManager,
};
use crate::types::SimulationConfig;
use tracing::{debug, info, instrument, warn};

/// Main simulation orchestrator that coordinates all components
#[derive(Debug)]
pub struct SimulationOrchestrator {
    /// Configuration for the simulation
    #[allow(dead_code)]
    config: SimulationConfig,
    /// Registry of all locations, buildings, and rooms
    location_registry: LocationRegistry,
    /// All users in the simulation
    users: Vec<User>,
    /// Time management system
    time_manager: TimeManager,
    /// Behavioral engine for activity generation
    #[allow(dead_code)]
    behavior_engine: BehaviorEngine,
    /// Event generator for creating access events
    event_generator: EventGenerator,
    /// Random number generator with optional seed
    #[allow(dead_code)]
    rng: rand::rngs::StdRng,
    /// Error handler for graceful error recovery
    #[allow(dead_code)]
    error_handler: ErrorHandler,
    /// Enhanced simulation statistics with detailed event tracking
    statistics: SimulationStatistics,
}

impl SimulationOrchestrator {
    /// Create a new simulation orchestrator
    #[instrument(skip(config), fields(user_count = config.user_count, location_count = config.location_count))]
    pub fn new(config: SimulationConfig) -> SimulationResult<Self> {
        info!(
            "Initializing simulation orchestrator with {} users across {} locations",
            config.user_count, config.location_count
        );

        // Initialize random number generator with optional seed
        let rng: rand::rngs::StdRng = if let Some(seed) = config.seed {
            info!("Using deterministic seed: {}", seed);
            rand::SeedableRng::seed_from_u64(seed)
        } else {
            debug!("Using entropy-based random seed");
            rand::SeedableRng::from_entropy()
        };

        // Create time manager for batch processing
        let time_manager = TimeManager::new();

        // Initialize empty components that will be populated during setup
        let location_registry = LocationRegistry::new();
        let users = Vec::new();
        let behavior_engine = BehaviorEngine::new(config.clone(), time_manager.clone());
        
        // Initialize statistics with detailed event tracking always enabled
        let statistics = SimulationStatistics::new(
            0, // users (will be updated during setup)
            0, // locations (will be updated during setup)
            0, // buildings (will be updated during setup)
            0, // rooms (will be updated during setup)
            0, // curious users (will be updated during setup)
            0, // cloned badge users (will be updated during setup)
            0, // night-shift users (will be updated during setup)
        );
        
        // Create event generator (statistics are now handled centrally)
        let event_generator = EventGenerator::new(
            config.clone(),
            location_registry.clone(),
            time_manager.clone(),
        );

        Ok(Self {
            config,
            location_registry,
            users,
            time_manager,
            behavior_engine,
            event_generator,
            rng,
            error_handler: ErrorHandler::new(),
            statistics,
        })
    }

    /// Get comprehensive simulation statistics with enhanced event tracking
    pub fn get_statistics(&self) -> SimulationStatistics {
        // Return the current statistics which includes all event tracking data
        self.statistics.clone()
    }

    /// Get a mutable reference to the statistics for updates during simulation
    pub fn get_statistics_mut(&mut self) -> &mut SimulationStatistics {
        &mut self.statistics
    }

    /// Update the orchestrator with generated facilities and users
    pub fn initialize_with_data(&mut self, location_registry: LocationRegistry, users: Vec<User>) -> SimulationResult<()> {
        let curious_count = users.iter().filter(|e| e.is_curious).count();
        let cloned_badge_count = users.iter().filter(|e| e.has_cloned_badge).count();
        let night_shift_count = users.iter().filter(|e| e.is_night_shift).count();

        // Update statistics with actual counts
        self.statistics = SimulationStatistics::new(
            users.len(),
            location_registry.location_count(),
            location_registry.total_building_count(),
            location_registry.total_room_count(),
            curious_count,
            cloned_badge_count,
            night_shift_count,
        );

        // NOTE: Event generator no longer tracks statistics - handled centrally

        // Store the data
        self.location_registry = location_registry;
        self.users = users;

        info!(
            "Orchestrator initialized with {} users, {} locations, {} buildings, {} rooms",
            self.statistics.total_users,
            self.statistics.total_locations,
            self.statistics.total_buildings,
            self.statistics.total_rooms
        );

        Ok(())
    }

    /// Get detailed runtime statistics
    pub fn get_runtime_statistics(&self) -> RuntimeStatistics {
        let current_time = self.time_manager.current_simulated_time();

        // Count users by current activity
        let mut users_with_activities = 0;
        let mut users_idle = 0;

        for user in &self.users {
            if user.get_current_activity(current_time).is_some() {
                users_with_activities += 1;
            } else {
                users_idle += 1;
            }
        }

        RuntimeStatistics::new(
            current_time,
            users_with_activities,
            users_idle,
            std::time::Duration::from_secs(0), // This would need to be tracked from start time
        )
    }

    /// Generate events for a single user activity
    /// 
    /// NOTE: Statistics tracking is now handled externally by the caller.
    /// This method only generates events and does not update statistics.
    pub fn generate_events_for_activity(
        &mut self,
        user: &User,
        activity: &crate::user::ScheduledActivity,
        current_time: chrono::DateTime<chrono::Utc>,
    ) -> SimulationResult<Vec<crate::events::AccessEvent>> {
        // Generate events using the event generator
        let events = self.event_generator.generate_events_from_activity(user, activity, current_time)?;

        debug!(
            "Generated {} events for user {} activity {:?}",
            events.len(),
            user.id,
            activity.activity_type
        );

        Ok(events)
    }

    /// Update statistics with a batch of events (DEPRECATED)
    /// 
    /// NOTE: This method is deprecated. Statistics tracking is now handled
    /// centrally by the BatchEventGenerator to avoid duplicate counting.
    /// This method is kept for backward compatibility but should not be used.
    #[deprecated(note = "Use centralized statistics tracking in BatchEventGenerator instead")]
    pub fn update_statistics_with_events(&mut self, events: &[crate::events::AccessEvent]) {
        warn!("update_statistics_with_events is deprecated - statistics should be handled centrally");
        
        // For backward compatibility, we'll still update statistics but log a warning
        for event in events {
            // First, check if this is a night-shift event
            if let Some(metadata) = &event.metadata {
                if metadata.is_night_shift_event {
                    self.statistics.increment_night_shift_events();
                    continue; // Night-shift events are tracked separately
                }
            }

            // Then, check for specific failure reasons that need special handling
            if let Some(failure_reason) = &event.failure_reason {
                match failure_reason {
                    crate::types::FailureReason::BadgeReaderError => {
                        self.statistics.increment_badge_reader_failure_events();
                        continue; // Don't double-count as a regular failure
                    }
                    _ => {
                        // Other failure reasons fall through to normal event type classification
                    }
                }
            }

            // Standard event type classification
            match event.event_type {
                crate::types::EventType::Success => {
                    self.statistics.increment_success_events();
                }
                crate::types::EventType::Failure => {
                    self.statistics.increment_failure_events();
                }
                crate::types::EventType::InvalidBadge => {
                    self.statistics.increment_invalid_badge_events();
                }
                crate::types::EventType::OutsideHours => {
                    self.statistics.increment_outside_hours_events();
                }
                crate::types::EventType::Suspicious => {
                    self.statistics.increment_suspicious_events();
                }
            }
        }

        debug!("Updated statistics with {} events, total events now: {}", events.len(), self.statistics.event_type_statistics().total_events);
    }

    /// Get a summary of current event statistics
    pub fn get_event_statistics_summary(&self) -> String {
        self.statistics.event_type_statistics().summary()
    }

    /// Get detailed event statistics breakdown
    pub fn get_detailed_event_statistics(&self) -> String {
        self.statistics.event_type_statistics().detailed_breakdown()
    }

    /// Increment curious events counter (for testing and external event generation)
    pub fn increment_curious_events(&mut self, count: usize) {
        for _ in 0..count {
            self.statistics.increment_curious_events();
        }
    }

    /// Increment impossible traveler events counter (for testing and external event generation)
    pub fn increment_impossible_traveler_events(&mut self, count: usize) {
        for _ in 0..count {
            self.statistics.increment_impossible_traveler_events();
        }
    }

    /// Increment badge technical failure events counter (for testing and external event generation)
    pub fn increment_badge_technical_failure_events(&mut self, count: usize) {
        for _ in 0..count {
            self.statistics.increment_badge_reader_failure_events();
        }
    }

    // NOTE: Statistics synchronization has been removed to eliminate duplicate tracking
    // All statistics are now handled centrally by the BatchEventGenerator
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SimulationConfig;

    #[test]
    fn test_orchestrator_creation() {
        let config = SimulationConfig::default();
        let orchestrator = SimulationOrchestrator::new(config);

        assert!(orchestrator.is_ok());
        let orchestrator = orchestrator.unwrap();
        assert_eq!(orchestrator.users.len(), 0); // No users before initialization
        
        // Verify statistics are initialized with detailed tracking enabled
        let stats = orchestrator.get_statistics();
        assert_eq!(stats.total_users, 0);
        assert_eq!(stats.event_type_statistics().total_events, 0);
    }

    #[test]
    fn test_orchestrator_methods() {
        let config = SimulationConfig::default();
        let orchestrator = SimulationOrchestrator::new(config).unwrap();

        // Test that we can get statistics from an uninitialized orchestrator
        let stats = orchestrator.get_statistics();
        assert_eq!(stats.total_users, 0);
        assert_eq!(stats.event_type_statistics().total_events, 0);

        let runtime_stats = orchestrator.get_runtime_statistics();
        assert_eq!(runtime_stats.total_users(), 0);
        
        // Test statistics summary methods
        let summary = orchestrator.get_event_statistics_summary();
        assert!(summary.contains("0 total events"));
        
        let detailed = orchestrator.get_detailed_event_statistics();
        assert!(detailed.contains("Total Events Generated: 0"));
    }

    #[test]
    fn test_statistics_generation() {
        let config = SimulationConfig::default();
        let orchestrator = SimulationOrchestrator::new(config).unwrap();

        let stats = orchestrator.get_statistics();
        assert_eq!(stats.total_users, 0);
        assert_eq!(stats.total_locations, 0);
        
        // Verify enhanced statistics are available
        let event_stats = stats.event_type_statistics();
        assert_eq!(event_stats.total_events, 0);
        assert_eq!(event_stats.success_events, 0);
        assert_eq!(event_stats.curious_events, 0);
        assert_eq!(event_stats.impossible_traveler_events, 0);

        let runtime_stats = orchestrator.get_runtime_statistics();
        assert_eq!(runtime_stats.total_users(), 0);
    }

    #[test]
    fn test_orchestrator_initialization_with_data() {
        use crate::facility::LocationRegistry;
        use crate::user::User;
        use crate::types::{UserId, LocationId, BuildingId};

        let config = SimulationConfig::default();
        let mut orchestrator = SimulationOrchestrator::new(config).unwrap();

        // Create test data
        let location_registry = LocationRegistry::new();
        let users = vec![
            User::new_for_testing(UserId::new(), LocationId::new(), BuildingId::new(), false, false),
            User::new_for_testing(UserId::new(), LocationId::new(), BuildingId::new(), true, false),
            User::new_for_testing(UserId::new(), LocationId::new(), BuildingId::new(), false, true),
        ];

        // Initialize with data
        let result = orchestrator.initialize_with_data(location_registry, users);
        assert!(result.is_ok());

        // Verify statistics were updated
        let stats = orchestrator.get_statistics();
        assert_eq!(stats.total_users, 3);
        assert_eq!(stats.curious_users, 1);
        assert_eq!(stats.cloned_badge_users, 1);
    }

    #[test]
    fn test_statistics_updates_with_events() {
        use crate::events::AccessEvent;
        use crate::types::{UserId, RoomId, BuildingId, LocationId, EventType};
        use chrono::Utc;

        let config = SimulationConfig::default();
        let mut orchestrator = SimulationOrchestrator::new(config).unwrap();

        // Create test events
        let events = vec![
            AccessEvent::new(
                Utc::now(),
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                true,
                EventType::Success,
            ),
            AccessEvent::new(
                Utc::now(),
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                false,
                EventType::Failure,
            ),
            AccessEvent::new(
                Utc::now(),
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                false,
                EventType::Suspicious,
            ),
        ];

        // Statistics are now handled centrally in BatchEventGenerator
        // This test method is deprecated but kept for compatibility
        #[allow(deprecated)]
        orchestrator.update_statistics_with_events(&events);

        // Verify statistics were updated correctly
        let stats = orchestrator.get_statistics();
        let event_stats = stats.event_type_statistics();
        assert_eq!(event_stats.total_events, 3);
        assert_eq!(event_stats.success_events, 1);
        assert_eq!(event_stats.failure_events, 2); // Both failure and suspicious events count as failures
        assert_eq!(event_stats.suspicious_events, 1);

        // Test summary methods
        let summary = orchestrator.get_event_statistics_summary();
        assert!(summary.contains("3 total events"));
        assert!(summary.contains("Success: 1"));

        let detailed = orchestrator.get_detailed_event_statistics();
        assert!(detailed.contains("Total Events Generated: 3"));
        assert!(detailed.contains("Success Events: 1"));
    }
}
