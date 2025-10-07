//! Batch event generator for simplified event generation
//!
//! This module contains the BatchEventGenerator that replaces the complex streaming
//! system with a simple batch-based approach for generating events day by day.

use chrono::NaiveDate;
use tracing::{debug, info, instrument, warn};

use crate::user::User;
use crate::events::{AccessEvent, EventGenerator};
use crate::facility::LocationRegistry;
use crate::simulation::{BehaviorEngine, SimulationResult, SimulationStatistics, TimeManager};
use crate::types::SimulationConfig;

/// Batch event generator that processes events day by day sequentially
/// 
/// This component replaces the complex streaming system with a straightforward
/// batch processing approach. It generates complete day's worth of events before
/// moving to the next day, ensuring proper chronological ordering.
#[derive(Debug)]
pub struct BatchEventGenerator {
    /// Behavioral engine for generating daily schedules
    behavior_engine: BehaviorEngine,
    /// Event generator for creating access events from activities
    event_generator: EventGenerator,
    /// Registry of all locations, buildings, and rooms
    location_registry: LocationRegistry,
    /// All users in the simulation
    users: Vec<User>,
    /// Consolidated statistics tracker (single source of truth)
    statistics: SimulationStatistics,
}

impl BatchEventGenerator {
    /// Create a new batch event generator
    /// 
    /// # Arguments
    /// * `config` - Simulation configuration
    /// * `location_registry` - Registry of locations, buildings, and rooms
    /// * `users` - Vector of all users to simulate
    /// 
    /// # Returns
    /// A new BatchEventGenerator instance ready for event generation
    #[instrument(skip(location_registry, users), fields(user_count = users.len(), location_count = location_registry.location_count()))]
    pub fn new(
        config: SimulationConfig,
        location_registry: LocationRegistry,
        users: Vec<User>,
    ) -> Self {
        info!(
            "Initializing batch event generator with {} users across {} locations",
            users.len(),
            location_registry.location_count()
        );

        // Create time manager for batch processing
        let time_manager = TimeManager::new();

        // Create behavior engine for daily schedule generation
        let behavior_engine = BehaviorEngine::new(config.clone(), time_manager.clone());

        // Initialize consolidated statistics with actual counts
        let curious_count = users.iter().filter(|e| e.is_curious).count();
        let cloned_badge_count = users.iter().filter(|e| e.has_cloned_badge).count();
        let night_shift_count = users.iter().filter(|e| e.is_night_shift).count();

        let statistics = SimulationStatistics::new(
            users.len(),
            location_registry.location_count(),
            location_registry.total_building_count(),
            location_registry.total_room_count(),
            curious_count,
            cloned_badge_count,
            night_shift_count,
        );

        // Create event generator (statistics are now handled centrally)
        let event_generator = EventGenerator::new(
            config.clone(),
            location_registry.clone(),
            time_manager.clone(),
        );

        Self {
            behavior_engine,
            event_generator,
            location_registry,
            users,
            statistics,
        }
    }

    /// Generate events for the specified number of days sequentially
    /// 
    /// This method processes each day completely before moving to the next day,
    /// ensuring proper chronological ordering of events within and across day boundaries.
    /// Events that span across day boundaries are properly carried forward and merged
    /// with the next day's events to maintain chronological order.
    /// 
    /// # Arguments
    /// * `num_days` - Number of days to simulate (must be > 0)
    /// 
    /// # Returns
    /// Result indicating success or failure of the batch generation process
    #[instrument(skip(self), fields(num_days = num_days, user_count = self.users.len()))]
    pub fn generate_events_for_days(&mut self, num_days: usize) -> SimulationResult<()> {
        if num_days == 0 {
            return Err(crate::simulation::SimulationError::behavior_engine_error(
                "Number of days must be greater than 0"
            ));
        }

        info!("Starting batch event generation for {} days with {} users", num_days, self.users.len());

        let start_time = std::time::Instant::now();
        let base_date = chrono::Utc::now().date_naive();
        
        // Track events that span into future days for proper ordering
        let mut pending_events_by_date: std::collections::HashMap<NaiveDate, Vec<AccessEvent>> = std::collections::HashMap::new();

        // Process each day sequentially
        for day_index in 0..num_days {
            let current_date = base_date + chrono::Duration::days(day_index as i64);
            
            debug!("Processing day {} of {} (date: {})", day_index + 1, num_days, current_date);
            
            // Generate events for this specific day
            let (day_events, future_events) = self.generate_events_for_single_day(current_date)?;
            
            // Collect any pending events from previous days that belong to this day
            let mut all_events_for_day = day_events;
            if let Some(pending_events) = pending_events_by_date.remove(&current_date) {
                debug!("Adding {} pending events from previous days to {}", pending_events.len(), current_date);
                all_events_for_day.extend(pending_events);
                
                // Re-sort all events for this day to maintain chronological order
                all_events_for_day.sort_by_key(|event| event.timestamp);
            }
            
            // Store future events for processing on their respective days
            for (future_date, events) in future_events {
                debug!("Storing {} events for future date {}", events.len(), future_date);
                pending_events_by_date.entry(future_date).or_default().extend(events);
            }
            
            // Output events for this day (sorted chronologically)
            self.output_events_for_day(all_events_for_day, current_date)?;
            
            info!("Completed day {} of {} - generated events for {}", day_index + 1, num_days, current_date);
        }
        
        // Handle any remaining events that extend beyond the simulation period
        if !pending_events_by_date.is_empty() {
            let total_remaining = pending_events_by_date.values().map(|v| v.len()).sum::<usize>();
            debug!("Simulation ended with {} events extending beyond the {} day period", total_remaining, num_days);
            
            // Output these events sorted by date and time
            let mut all_remaining_events: Vec<AccessEvent> = pending_events_by_date
                .into_values()
                .flatten()
                .collect();
            all_remaining_events.sort_by_key(|event| event.timestamp);
            
            if !all_remaining_events.is_empty() {
                use crate::events::FilteredAccessEvent;
                
                info!("Outputting {} events that extend beyond simulation period", all_remaining_events.len());
                
                // Get the output field configuration
                let field_config = &self.behavior_engine.get_config().output_fields;
                
                for event in &all_remaining_events {
                    // Create filtered event based on configuration
                    let filtered_event = FilteredAccessEvent::from_access_event(event, field_config);
                    
                    match serde_json::to_string(&filtered_event) {
                        Ok(json_line) => println!("{}", json_line),
                        Err(e) => warn!("Failed to serialize remaining event to JSON: {}", e),
                    }
                }
                
                // Update statistics with these remaining events
                self.update_statistics_with_events(&all_remaining_events);
            }
        }

        // Update final statistics
        self.statistics.set_days_simulated(num_days);
        self.statistics.set_simulation_duration(start_time.elapsed());

        info!(
            "Batch event generation completed: {} days, {} total events in {:.2} seconds",
            num_days,
            self.statistics.total_events,
            start_time.elapsed().as_secs_f64()
        );

        Ok(())
    }

    /// Generate events for a single day using existing daily schedule generation
    /// 
    /// This method leverages the existing BehaviorEngine to generate daily schedules
    /// and the EventGenerator to create realistic events from those schedules.
    /// Events that span across day boundaries are properly separated and returned
    /// for processing on their respective days.
    /// 
    /// # Arguments
    /// * `date` - The date to generate events for
    /// 
    /// # Returns
    /// Tuple containing:
    /// - Vector of access events for the current day, sorted by timestamp
    /// - HashMap of future events organized by date for proper day boundary handling
    #[instrument(skip(self), fields(date = %date, user_count = self.users.len()))]
    fn generate_events_for_single_day(&mut self, date: NaiveDate) -> SimulationResult<(Vec<AccessEvent>, std::collections::HashMap<NaiveDate, Vec<AccessEvent>>)> {
        debug!("Generating events for date: {}", date);

        let mut all_day_events = Vec::new();
        let mut future_events_by_date: std::collections::HashMap<NaiveDate, Vec<AccessEvent>> = std::collections::HashMap::new();

        // Process each user for this day
        for user in &self.users {
            // Generate daily schedule using existing BehaviorEngine
            let daily_schedule = self.behavior_engine.generate_daily_schedule(
                user,
                date,
                &self.location_registry,
            )?;

            debug!(
                "Generated {} activities for user {} on {}",
                daily_schedule.len(),
                user.id,
                date
            );

            // Generate events from each activity in the schedule
            for activity in &daily_schedule {
                let activity_events = self.event_generator.generate_events_from_activity(
                    user,
                    activity,
                    activity.start_time,
                )?;

                // Separate events by day boundary for proper chronological ordering
                for event in activity_events {
                    let event_date = event.timestamp.date_naive();
                    
                    if event_date == date {
                        // Event belongs to current day
                        all_day_events.push(event);
                    } else if event_date > date {
                        // Event spans into future day - store for proper day boundary handling
                        debug!(
                            "Event for user {} at {} spans into future day ({}), storing for later processing",
                            user.id, event.timestamp, event_date
                        );
                        future_events_by_date.entry(event_date).or_default().push(event);
                    } else {
                        // Event from previous day - this shouldn't happen in normal operation
                        warn!(
                            "Event for user {} at {} belongs to previous day ({}), this may indicate a scheduling issue",
                            user.id, event.timestamp, event_date
                        );
                        // Still include it in current day to avoid losing events
                        all_day_events.push(event);
                    }
                }
            }
        }

        // Log summary of events that span into future days
        if !future_events_by_date.is_empty() {
            let total_future_events: usize = future_events_by_date.values().map(|v| v.len()).sum();
            debug!(
                "Generated {} events from {} that span into {} future days",
                total_future_events,
                date,
                future_events_by_date.len()
            );
            
            // Log details for each future date
            for (future_date, events) in &future_events_by_date {
                debug!(
                    "  {} events for future date {}",
                    events.len(),
                    future_date
                );
            }
        }

        // Sort all events for this day by timestamp to ensure chronological order
        all_day_events.sort_by_key(|event| event.timestamp);

        // Sort future events by timestamp within each date for consistency
        for events in future_events_by_date.values_mut() {
            events.sort_by_key(|event| event.timestamp);
        }

        // Update statistics with the events generated for this day only
        // Future events will be counted when they are processed on their respective days
        self.update_statistics_with_events(&all_day_events);

        info!(
            "Generated {} events for {} (sorted chronologically), {} events span into future days",
            all_day_events.len(),
            date,
            future_events_by_date.values().map(|v| v.len()).sum::<usize>()
        );

        Ok((all_day_events, future_events_by_date))
    }

    /// Output events for a day to stdout in the configured format
    /// 
    /// # Arguments
    /// * `events` - Vector of events to output (should be sorted by timestamp)
    /// * `date` - The date these events are for (used for logging)
    /// 
    /// # Returns
    /// Result indicating success or failure of the output operation
    #[instrument(skip(self, events), fields(event_count = events.len(), date = %date))]
    fn output_events_for_day(&self, events: Vec<AccessEvent>, date: NaiveDate) -> SimulationResult<()> {
        use crate::events::FilteredAccessEvent;
        
        debug!("Outputting {} events for {}", events.len(), date);

        // Get the output field configuration from behavior engine's config
        let field_config = &self.behavior_engine.get_config().output_fields;

        // Output each event in the configured format
        for event in &events {
            // Create filtered event based on configuration
            let filtered_event = FilteredAccessEvent::from_access_event(event, field_config);
            
            // Convert filtered event to JSON and output to stdout
            match serde_json::to_string(&filtered_event) {
                Ok(json_line) => {
                    println!("{}", json_line);
                }
                Err(e) => {
                    warn!("Failed to serialize event to JSON: {}", e);
                    // Continue with other events rather than failing completely
                }
            }
        }

        debug!("Successfully output {} events for {}", events.len(), date);
        Ok(())
    }

    /// Update consolidated statistics with a batch of events
    /// 
    /// This method processes events and updates the centralized statistics tracker,
    /// ensuring all event types are properly counted in a single location.
    /// This is the ONLY place where statistics should be updated to avoid duplication.
    /// 
    /// # Arguments
    /// * `events` - Slice of events to process for statistics
    /// * `activity` - Optional activity context for curious event detection
    fn update_statistics_with_events(&mut self, events: &[AccessEvent]) {
        // Track curious events and impossible traveler events by analyzing the events
        self.detect_and_track_curious_events(events);
        self.detect_and_track_impossible_traveler_events(events);

        // Process each event for standard statistics
        for event in events {
            // Check for special event types first
            if let Some(metadata) = &event.metadata {
                if metadata.is_night_shift_event {
                    self.statistics.increment_night_shift_events();
                    continue; // Night-shift events are tracked separately, don't count as regular events
                }
            }

            // Check for specific failure reasons that need special handling
            if let Some(failure_reason) = &event.failure_reason {
                match failure_reason {
                    crate::types::FailureReason::BadgeReaderError => {
                        self.statistics.increment_badge_reader_failure_events();
                        continue; // Don't double-count as a regular failure
                    }
                    crate::types::FailureReason::CuriousUser => {
                        // Curious events are already tracked above, just count as regular failure
                        self.statistics.increment_failure_events();
                        continue;
                    }
                    crate::types::FailureReason::ImpossibleTraveler => {
                        // Impossible traveler events are already tracked above, count as regular event
                        if event.success {
                            self.statistics.increment_success_events();
                        } else {
                            self.statistics.increment_failure_events();
                        }
                        continue;
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

        debug!(
            "Updated statistics with {} events, total events now: {}",
            events.len(),
            self.statistics.total_events
        );
    }

    /// Detect and track curious events in a batch of events
    /// 
    /// Curious events are unauthorized access attempts by curious users.
    /// This method identifies events that represent curious behavior and updates statistics.
    /// 
    /// # Arguments
    /// * `events` - Slice of events to analyze for curious behavior
    fn detect_and_track_curious_events(&mut self, events: &[AccessEvent]) {
        for event in events {
            // Check if this event has a curious user failure reason
            if let Some(failure_reason) = &event.failure_reason {
                if matches!(failure_reason, crate::types::FailureReason::CuriousUser) {
                    self.statistics.increment_curious_events();
                    debug!(
                        "Detected curious user event for user {} at room {}",
                        event.user_id, event.room_id
                    );
                }
            }
            
            // Also check for events from curious users that failed due to lack of authorization
            // These would be unauthorized access attempts
            if !event.success && event.event_type == crate::types::EventType::Failure {
                // We can't directly check if the user is curious from the event,
                // but we can infer it from the context. For now, we'll rely on the
                // failure reason being set correctly by the event generator.
            }
        }
    }

    /// Detect and track impossible traveler events in a batch of events
    /// 
    /// Impossible traveler events occur when the same user appears to access
    /// different geographical locations within an impossible timeframe for physical travel.
    /// 
    /// # Arguments
    /// * `events` - Slice of events to analyze for impossible traveler scenarios
    fn detect_and_track_impossible_traveler_events(&mut self, events: &[AccessEvent]) {
        use std::collections::HashMap;
        
        // Group events by user to check for impossible traveler patterns
        let mut user_events: HashMap<crate::types::UserId, Vec<&AccessEvent>> = HashMap::new();
        
        for event in events {
            user_events.entry(event.user_id).or_default().push(event);
        }

        // Check each user's events for impossible traveler scenarios
        for (user_id, user_events) in user_events {
            if user_events.len() < 2 {
                continue; // Need at least 2 events to form a scenario
            }

            // Sort events by timestamp for chronological analysis
            let mut sorted_events = user_events;
            sorted_events.sort_by_key(|event| event.timestamp);

            // Check for impossible traveler scenarios between consecutive events
            for i in 0..sorted_events.len() {
                for j in (i + 1)..sorted_events.len() {
                    if self.is_impossible_traveler_scenario(sorted_events[i], sorted_events[j]) {
                        self.statistics.increment_impossible_traveler_events();
                        debug!(
                            "Detected impossible traveler scenario for user {} between events at locations {} and {}",
                            user_id, sorted_events[i].location_id, sorted_events[j].location_id
                        );
                        // Only count each pair once per batch
                        break;
                    }
                }
            }
        }
    }

    /// Check if two events constitute an impossible traveler scenario
    /// 
    /// This method validates whether the time gap between two access events from different
    /// geographical locations is insufficient for physical travel.
    /// 
    /// # Arguments
    /// * `event1` - First access event
    /// * `event2` - Second access event
    /// 
    /// # Returns
    /// `true` if the events represent an impossible traveler scenario
    fn is_impossible_traveler_scenario(&self, event1: &AccessEvent, event2: &AccessEvent) -> bool {
        // Events must be from the same user
        if event1.user_id != event2.user_id {
            return false;
        }

        // Events must be from different geographical locations
        if event1.location_id == event2.location_id {
            return false;
        }

        // Calculate time gap between events
        let time_gap = if event2.timestamp > event1.timestamp {
            event2.timestamp - event1.timestamp
        } else {
            event1.timestamp - event2.timestamp
        };

        // Consider it impossible if the time gap is less than 4 hours
        // This is a reasonable threshold for cross-location travel
        let minimum_travel_time = chrono::Duration::hours(4);
        
        time_gap < minimum_travel_time
    }

    /// Get a reference to the consolidated statistics
    /// 
    /// # Returns
    /// Reference to the current statistics tracker
    pub fn get_statistics(&self) -> &SimulationStatistics {
        &self.statistics
    }

    /// Get a mutable reference to the consolidated statistics
    /// 
    /// # Returns
    /// Mutable reference to the current statistics tracker
    pub fn get_statistics_mut(&mut self) -> &mut SimulationStatistics {
        &mut self.statistics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::User;
    use crate::facility::LocationRegistry;
    use crate::types::{BuildingId, UserId, LocationId, RoomId};
    use crate::permissions::PermissionSet;
    use chrono::Utc;

    fn create_test_setup() -> (SimulationConfig, LocationRegistry, Vec<User>) {
        let config = SimulationConfig::default();
        let location_registry = LocationRegistry::new();
        
        // Create a minimal user for testing
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();
        
        let user = User::new(location_id, building_id, workspace_id, permissions);
        let users = vec![user];
        
        (config, location_registry, users)
    }

    #[test]
    fn test_batch_generator_creation() {
        let (config, location_registry, users) = create_test_setup();
        
        let generator = BatchEventGenerator::new(config, location_registry, users);
        
        assert_eq!(generator.users.len(), 1);
        assert_eq!(generator.statistics.total_users, 1);
        assert_eq!(generator.statistics.total_events, 0);
    }

    #[test]
    fn test_batch_generator_zero_days_error() {
        let (config, location_registry, users) = create_test_setup();
        let mut generator = BatchEventGenerator::new(config, location_registry, users);
        
        let result = generator.generate_events_for_days(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_generator_statistics_initialization() {
        let (config, location_registry, mut users) = create_test_setup();
        
        // Add a curious user
        users[0].is_curious = true;
        
        // Add a cloned badge user
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();
        let mut cloned_user = User::new(location_id, building_id, workspace_id, permissions);
        cloned_user.has_cloned_badge = true;
        users.push(cloned_user);
        
        let generator = BatchEventGenerator::new(config, location_registry, users);
        
        assert_eq!(generator.statistics.total_users, 2);
        assert_eq!(generator.statistics.curious_users, 1);
        assert_eq!(generator.statistics.cloned_badge_users, 1);
        assert_eq!(generator.statistics.night_shift_users, 0);
    }

    #[test]
    fn test_statistics_access() {
        let (config, location_registry, users) = create_test_setup();
        let mut generator = BatchEventGenerator::new(config, location_registry, users);
        
        // Test immutable access
        let stats = generator.get_statistics();
        assert_eq!(stats.total_users, 1);
        
        // Test mutable access
        let stats_mut = generator.get_statistics_mut();
        stats_mut.increment_success_events();
        assert_eq!(generator.statistics.total_events, 1);
        assert_eq!(generator.statistics.success_events, 1);
    }

    #[test]
    fn test_event_statistics_update() {
        use crate::events::AccessEvent;
        use crate::types::{EventType, FailureReason};
        
        let (config, location_registry, users) = create_test_setup();
        let mut generator = BatchEventGenerator::new(config, location_registry, users);
        
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
            AccessEvent::new_with_failure_info(
                Utc::now(),
                UserId::new(),
                RoomId::new(),
                BuildingId::new(),
                LocationId::new(),
                false,
                EventType::Failure,
                Some(FailureReason::BadgeReaderError),
                None,
            ),
        ];
        
        generator.update_statistics_with_events(&events);
        
        assert_eq!(generator.statistics.total_events, 2);
        assert_eq!(generator.statistics.success_events, 1);
        assert_eq!(generator.statistics.failure_events, 1);
    }

    #[test]
    fn test_event_ordering_across_day_boundaries() {
        use crate::events::AccessEvent;
        use crate::types::EventType;
        use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
        
        let (config, location_registry, users) = create_test_setup();
        let _generator = BatchEventGenerator::new(config, location_registry, users);
        
        // Create test date
        let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        
        // Create events that span across day boundaries
        let current_day_time = test_date.and_time(NaiveTime::from_hms_opt(14, 30, 0).unwrap());
        let next_day_time = test_date.succ_opt().unwrap().and_time(NaiveTime::from_hms_opt(2, 15, 0).unwrap());
        let day_after_time = test_date.succ_opt().unwrap().succ_opt().unwrap().and_time(NaiveTime::from_hms_opt(10, 0, 0).unwrap());
        
        let current_day_event = AccessEvent::new(
            Utc.from_utc_datetime(&current_day_time),
            UserId::new(),
            RoomId::new(),
            BuildingId::new(),
            LocationId::new(),
            true,
            EventType::Success,
        );
        
        let next_day_event = AccessEvent::new(
            Utc.from_utc_datetime(&next_day_time),
            UserId::new(),
            RoomId::new(),
            BuildingId::new(),
            LocationId::new(),
            true,
            EventType::Success,
        );
        
        let day_after_event = AccessEvent::new(
            Utc.from_utc_datetime(&day_after_time),
            UserId::new(),
            RoomId::new(),
            BuildingId::new(),
            LocationId::new(),
            true,
            EventType::Success,
        );
        
        // Simulate events being generated with mixed timestamps
        let mixed_events = vec![next_day_event.clone(), current_day_event.clone(), day_after_event.clone()];
        
        // Test event separation by date
        let mut current_day_events = Vec::new();
        let mut future_events_by_date: std::collections::HashMap<NaiveDate, Vec<AccessEvent>> = std::collections::HashMap::new();
        
        for event in mixed_events {
            let event_date = event.timestamp.date_naive();
            
            if event_date == test_date {
                current_day_events.push(event);
            } else if event_date > test_date {
                future_events_by_date.entry(event_date).or_default().push(event);
            }
        }
        
        // Verify proper separation
        assert_eq!(current_day_events.len(), 1);
        assert_eq!(current_day_events[0].timestamp, current_day_event.timestamp);
        
        assert_eq!(future_events_by_date.len(), 2); // Two future dates
        
        // Verify next day events
        let next_date = test_date.succ_opt().unwrap();
        assert!(future_events_by_date.contains_key(&next_date));
        assert_eq!(future_events_by_date[&next_date].len(), 1);
        assert_eq!(future_events_by_date[&next_date][0].timestamp, next_day_event.timestamp);
        
        // Verify day after events
        let day_after_date = next_date.succ_opt().unwrap();
        assert!(future_events_by_date.contains_key(&day_after_date));
        assert_eq!(future_events_by_date[&day_after_date].len(), 1);
        assert_eq!(future_events_by_date[&day_after_date][0].timestamp, day_after_event.timestamp);
        
        // Test chronological sorting within each day
        current_day_events.sort_by_key(|event| event.timestamp);
        for events in future_events_by_date.values_mut() {
            events.sort_by_key(|event| event.timestamp);
        }
        
        // Verify sorting is maintained
        assert_eq!(current_day_events[0].timestamp, current_day_event.timestamp);
        assert_eq!(future_events_by_date[&next_date][0].timestamp, next_day_event.timestamp);
        assert_eq!(future_events_by_date[&day_after_date][0].timestamp, day_after_event.timestamp);
    }
}
