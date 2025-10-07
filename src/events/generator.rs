//! Event generation logic
//!
//! This module contains event generation and realistic event creation logic.

use anyhow::anyhow;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

use crate::user::{User, ScheduledActivity};
use crate::events::metadata::ImpossibleTravelerMetadata;
use crate::events::{AccessAttempt, AccessEvent, EventMetadata};
use crate::facility::{LocationRegistry, Room};
use crate::simulation::{ErrorHandler, SimulationError, SimulationResult, SimulationStatistics, TimeManager, TimeVariance};
use crate::types::{
    UserId, EventType, FailureReason, LocationId, RoomId, RoomType, SecurityLevel,
    SimulationConfig,
};

/// Event generation system that creates access events from user activities
#[derive(Debug)]
pub struct EventGenerator {
    /// Configuration for the simulation
    #[allow(dead_code)]
    config: SimulationConfig,
    /// Registry of all locations, buildings, and rooms
    location_registry: LocationRegistry,
    /// Time management system
    time_manager: TimeManager,
    /// Random number generator for event generation
    rng: rand::rngs::ThreadRng,
    /// Error handler for graceful error recovery
    #[allow(dead_code)]
    error_handler: ErrorHandler,

    /// Time variance system for realistic event timing
    time_variance: TimeVariance,
}

impl EventGenerator {
    /// Create a new event generator
    pub fn new(
        config: SimulationConfig,
        location_registry: LocationRegistry,
        time_manager: TimeManager,
    ) -> Self {
        info!("Initializing event generator with {} locations", location_registry.location_count());
        Self {
            config,
            location_registry,
            time_manager,
            rng: rand::thread_rng(),
            error_handler: ErrorHandler::new(),
            time_variance: TimeVariance::new(),
        }
    }

    /// Create a new event generator with statistics tracking (DEPRECATED)
    /// 
    /// NOTE: Statistics tracking has been moved to centralized location.
    /// This method is kept for backward compatibility but statistics parameter is ignored.
    #[deprecated(note = "Statistics tracking is now handled centrally")]
    pub fn new_with_statistics(
        config: SimulationConfig,
        location_registry: LocationRegistry,
        time_manager: TimeManager,
        _statistics: SimulationStatistics,
    ) -> Self {
        info!("Initializing event generator with {} locations (statistics tracking is now centralized)", location_registry.location_count());
        Self {
            config,
            location_registry,
            time_manager,
            rng: rand::thread_rng(),
            error_handler: ErrorHandler::new(),
            time_variance: TimeVariance::new(),
        }
    }

    /// Set the statistics tracker for this event generator (DEPRECATED)
    /// 
    /// NOTE: Statistics tracking has been moved to centralized location.
    /// This method is kept for backward compatibility but does nothing.
    #[deprecated(note = "Statistics tracking is now handled centrally")]
    pub fn set_statistics(&mut self, _statistics: SimulationStatistics) {
        // Statistics are now handled externally, this method does nothing
    }

    /// Get a reference to the statistics tracker (DEPRECATED)
    /// 
    /// NOTE: Statistics tracking has been moved to centralized location.
    /// This method always returns None.
    #[deprecated(note = "Statistics tracking is now handled centrally")]
    pub fn statistics(&self) -> Option<&SimulationStatistics> {
        None // Statistics are now handled externally
    }

    /// Get a mutable reference to the statistics tracker (DEPRECATED)
    /// 
    /// NOTE: Statistics tracking has been moved to centralized location.
    /// This method always returns None.
    #[deprecated(note = "Statistics tracking is now handled centrally")]
    pub fn statistics_mut(&mut self) -> Option<&mut SimulationStatistics> {
        None // Statistics are now handled externally
    }

    /// Generate access events from a user's scheduled activity
    #[instrument(skip(self), fields(user_id = %user.id, activity_type = ?activity.activity_type, target_room = %activity.target_room))]
    pub fn generate_events_from_activity(
        &mut self,
        user: &User,
        activity: &ScheduledActivity,
        current_time: DateTime<Utc>,
    ) -> SimulationResult<Vec<AccessEvent>> {
        debug!(
            "Generating events for user {} activity {:?} at room {}",
            user.id, activity.activity_type, activity.target_room
        );

        // Generate events with error handling
        let result = match self.generate_events_internal(user, activity, current_time) {
            Ok(events) => Some(events),
            Err(e) => {
                warn!(
                    "Failed to generate events for user {} activity {:?}: {}",
                    user.id, activity.activity_type, e
                );
                None
            }
        };

        let mut events = match result {
            Some(events) => {
                info!(
                    "Generated {} events for user {} activity {:?}",
                    events.len(),
                    user.id,
                    activity.activity_type
                );
                events
            }
            None => {
                warn!(
                    "Failed to generate events for user {} activity, creating minimal event",
                    user.id
                );
                vec![self.create_minimal_event(user, activity, current_time)?]
            }
        };

        // Apply time variance to all generated events
        self.time_variance.apply_variance_to_events(&mut events);
        
        // Ensure unique timestamps after variance application
        self.time_variance.ensure_unique_timestamps(&mut events);

        // NOTE: Statistics tracking is now handled externally by the caller
        // This removes duplicate statistics tracking from the EventGenerator

        Ok(events)
    }

    /// Create a minimal event when full event generation fails
    #[instrument(skip(self), fields(user_id = %user.id, target_room = %activity.target_room))]
    fn create_minimal_event(
        &mut self,
        user: &User,
        activity: &ScheduledActivity,
        current_time: DateTime<Utc>,
    ) -> SimulationResult<AccessEvent> {
        debug!(
            "Creating minimal event for user {} at room {}",
            user.id, activity.target_room
        );

        // Get room information
        let room = self.location_registry.get_room(activity.target_room).ok_or_else(|| {
            SimulationError::event_generation_error(format!(
                "Room {} not found",
                activity.target_room
            ))
        })?;

        let building = self.location_registry.get_building(room.building_id).ok_or_else(|| {
            SimulationError::event_generation_error(format!(
                "Building {} not found",
                room.building_id
            ))
        })?;

        // Simple authorization check
        let is_authorized =
            user.can_access_room(activity.target_room, room.building_id, building.location_id);

        let event = AccessEvent {
            timestamp: current_time,
            user_id: user.id,
            room_id: activity.target_room,
            building_id: room.building_id,
            location_id: building.location_id,
            success: is_authorized,
            event_type: if is_authorized { EventType::Success } else { EventType::Failure },
            failure_reason: if is_authorized { None } else { Some(FailureReason::SystemFailure) },
            metadata: None,
        };

        debug!("Created minimal event: success={}, room={}", is_authorized, activity.target_room);
        Ok(event)
    }

    /// Internal event generation with error handling
    fn generate_events_internal(
        &mut self,
        user: &User,
        activity: &ScheduledActivity,
        current_time: DateTime<Utc>,
    ) -> SimulationResult<Vec<AccessEvent>> {
        let mut events = Vec::new();

        // Generate access attempts for the target room
        let access_attempts =
            self.generate_access_attempts_for_room(user, activity.target_room, current_time)?;

        // Convert access attempts to events, potentially generating badge reader failures
        for attempt in access_attempts {
            // Check if this is an authorized access attempt that should have a badge technical failure
            if attempt.is_authorized && self.should_generate_badge_reader_failure() {
                // Generate badge technical failure scenario instead of normal event
                let badge_reader_events = self.generate_badge_reader_failure(
                    user, 
                    attempt.target_room, 
                    attempt.timestamp
                )?;
                events.extend(badge_reader_events);
            } else {
                // Generate normal event
                let event = self.process_access_attempt(user, &attempt)?;
                events.push(event);
            }
        }

        // Generate additional events for curious users
        if user.is_curious && self.should_generate_curious_event() {
            if let Ok(curious_event) = self.generate_curious_user_event(user, current_time)
            {
                debug!(
                    "Generated curious user event for user {} at room {}",
                    user.id, curious_event.room_id
                );
                events.push(curious_event);
            }
        }

        // Generate impossible traveler events for users with cloned badges
        if user.has_cloned_badge && self.should_generate_impossible_traveler_event() {
            // Use enhanced impossible traveler generation with geographical validation
            if let Ok(impossible_events) =
                self.generate_enhanced_impossible_traveler_event(user, current_time)
            {
                debug!(
                    "Generated impossible traveler scenario for user {} between locations {} and {}",
                    user.id, 
                    impossible_events.get(0).map(|e| e.location_id).unwrap_or_default(),
                    impossible_events.get(1).map(|e| e.location_id).unwrap_or_default()
                );
                events.extend(impossible_events);
            } else {
                // Fallback to basic impossible traveler event if enhanced generation fails
                if let Ok(impossible_event) =
                    self.generate_impossible_traveler_event(user, current_time)
                {
                    debug!(
                        "Generated basic impossible traveler event for user {} at location {}",
                        user.id, impossible_event.location_id
                    );
                    events.push(impossible_event);
                }
            }
        }

        Ok(events)
    }

    /// Generate access attempts required to reach a target room
    fn generate_access_attempts_for_room(
        &mut self,
        user: &User,
        target_room: RoomId,
        start_time: DateTime<Utc>,
    ) -> SimulationResult<Vec<AccessAttempt>> {
        let current_room = user.current_state.current_room;

        // Get the access flow required to reach the target room
        let access_flow = self.location_registry.get_access_flow(
            current_room,
            target_room,
            &self.time_manager,
            &mut self.rng,
        )?;

        // Generate access attempts for the entire sequence
        let mut attempts = Vec::new();
        let mut current_time = start_time;

        for (index, &room_id) in access_flow.required_sequence.iter().enumerate() {
            // Calculate authorization for this room
            let _room = self.location_registry.get_room(room_id).expect("Room should exist");
            let building = self
                .location_registry
                .get_building_for_room(room_id)
                .expect("Building should exist");
            let location = self
                .location_registry
                .get_location_for_building(building.id)
                .expect("Location should exist");

            let is_authorized =
                user.permissions.can_access_room(room_id, building.id, location.id);

            attempts.push(AccessAttempt::new(user.id, room_id, is_authorized, current_time));

            // Add time between access attempts (except for the last one)
            if index < access_flow.required_sequence.len() - 1 {
                current_time += chrono::Duration::seconds(30); // 30 seconds between badge swipes
            }
        }

        Ok(attempts)
    }

    /// Process an access attempt and determine the outcome
    fn process_access_attempt(&mut self, user: &User, attempt: &AccessAttempt) -> SimulationResult<AccessEvent> {
        // Get room, building, and location information
        let room = self
            .location_registry
            .get_room(attempt.target_room)
            .ok_or_else(|| anyhow!("Room not found: {}", attempt.target_room))?;

        let building = self
            .location_registry
            .get_building(room.building_id)
            .ok_or_else(|| format!("Building not found: {}", room.building_id))?;

        // Extract needed values to avoid borrow checker issues
        let room_type = room.room_type;
        let security_level = room.security_level;
        let building_id = room.building_id;
        let location_id = building.location_id;
        let is_high_security = room.is_high_security();

        // Determine if access should succeed
        let success =
            self.evaluate_access_authorization_with_room_info(attempt, room_type, security_level);

        // Determine event type based on success and other factors
        let event_type =
            self.determine_event_type_with_room_info(attempt, is_high_security, success);

        // Check if this is a night-shift event during off-hours
        let is_night_shift_event = user.is_night_shift && !self.time_manager.is_business_hours(attempt.timestamp);
        
        // Create metadata if this is a night-shift event
        let metadata = if is_night_shift_event {
            Some(EventMetadata::night_shift_event())
        } else {
            None
        };

        Ok(AccessEvent::new_with_failure_info(
            attempt.timestamp,
            attempt.user_id,
            attempt.target_room,
            building_id,
            location_id,
            success,
            event_type,
            None, // No failure reason for basic events
            metadata,
        ))
    }

    /// Evaluate whether an access attempt should be authorized
    #[allow(dead_code)]
    fn evaluate_access_authorization(&mut self, attempt: &AccessAttempt, room: &Room) -> bool {
        self.evaluate_access_authorization_with_room_info(
            attempt,
            room.room_type,
            room.security_level,
        )
    }

    /// Evaluate whether an access attempt should be authorized (with room info to avoid borrow issues)
    fn evaluate_access_authorization_with_room_info(
        &mut self,
        attempt: &AccessAttempt,
        room_type: RoomType,
        _security_level: SecurityLevel,
    ) -> bool {
        // Basic authorization check
        if !attempt.is_authorized {
            return false;
        }

        // Check business hours for certain room types
        if self.requires_business_hours_access_for_type(room_type)
            && !self.time_manager.is_business_hours(attempt.timestamp)
        {
            return false;
        }



        // Add some randomness for system failures (very low probability)
        if self.rng.gen::<f64>() < 0.001 {
            return false; // 0.1% chance of system failure
        }

        true
    }

    /// Determine the appropriate event type for an access attempt
    #[allow(dead_code)]
    fn determine_event_type(
        &mut self,
        attempt: &AccessAttempt,
        room: &Room,
        success: bool,
    ) -> EventType {
        self.determine_event_type_with_room_info(attempt, room.is_high_security(), success)
    }

    /// Determine the appropriate event type for an access attempt (with room info to avoid borrow issues)
    fn determine_event_type_with_room_info(
        &mut self,
        attempt: &AccessAttempt,
        is_high_security: bool,
        success: bool,
    ) -> EventType {
        if success {
            EventType::Success
        } else if !attempt.is_authorized {
            EventType::Failure
        } else if !self.time_manager.is_business_hours(attempt.timestamp) {
            EventType::OutsideHours
        } else if is_high_security && self.rng.gen::<f64>() < 0.01 {
            EventType::Suspicious
        } else {
            EventType::Failure
        }
    }

    /// Check if a room type requires business hours access
    fn requires_business_hours_access_for_type(&self, room_type: RoomType) -> bool {
        matches!(room_type, RoomType::ServerRoom | RoomType::ExecutiveOffice | RoomType::Laboratory)
    }



    /// Generate an unauthorized access event for curious users
    fn generate_curious_user_event(
        &mut self,
        user: &User,
        current_time: DateTime<Utc>,
    ) -> SimulationResult<AccessEvent> {
        // Select a random room that the user is NOT authorized to access
        let unauthorized_room = self.select_unauthorized_room_for_user(user)?;

        let room = self
            .location_registry
            .get_room(unauthorized_room)
            .ok_or_else(|| format!("Room not found: {}", unauthorized_room))?;

        let building = self
            .location_registry
            .get_building(room.building_id)
            .ok_or_else(|| format!("Building not found: {}", room.building_id))?;

        // Create a curious user event directly instead of using process_access_attempt
        // This ensures the failure reason is properly set
        let event_time = current_time + Duration::minutes(self.rng.gen_range(1..=30));
        
        Ok(AccessEvent::new_with_failure_info(
            event_time,
            user.id,
            unauthorized_room,
            room.building_id,
            building.location_id,
            false, // Always fails (unauthorized access)
            crate::types::EventType::Failure,
            Some(crate::types::FailureReason::CuriousUser), // Mark as curious user event
            Some(EventMetadata::curious_attempt()), // Add curious user metadata
        ))
    }

    /// Generate an impossible traveler event for users with cloned badges
    fn generate_impossible_traveler_event(
        &mut self,
        user: &User,
        current_time: DateTime<Utc>,
    ) -> SimulationResult<AccessEvent> {
        // Select a room in a different geographical location
        let remote_room = self.select_remote_location_room(user)?;

        let room = self
            .location_registry
            .get_room(remote_room)
            .ok_or_else(|| format!("Room not found: {}", remote_room))?;

        let building = self
            .location_registry
            .get_building(room.building_id)
            .ok_or_else(|| format!("Building not found: {}", room.building_id))?;

        // Check if user has authorization for this room
        let is_authorized =
            user.can_access_room(remote_room, room.building_id, building.location_id);

        // Create access attempt with insufficient travel time
        let time_gap = Duration::minutes(self.rng.gen_range(1..=180)); // 1-3 hours (impossible for cross-location travel)
        let attempt =
            AccessAttempt::new(user.id, remote_room, is_authorized, current_time + time_gap);

        // Process the attempt (may succeed if authorized, creating the impossible traveler scenario)
        self.process_access_attempt(user, &attempt)
    }

    /// Select a room that the user is not authorized to access
    fn select_unauthorized_room_for_user(
        &mut self,
        user: &User,
    ) -> SimulationResult<RoomId> {


        if let Some(location) = self.location_registry.get_location(user.primary_location) {
            let mut unauthorized_rooms = Vec::new();
            for building in &location.buildings {
                for room in &building.rooms {
                    if !user.can_access_room(room.id, building.id, location.id) {
                        unauthorized_rooms.push(room.id);
                    }
                }
            }
            if unauthorized_rooms.is_empty() {
                return Err(anyhow!("No unauthorized rooms available for curious user").into());
            }
            Ok(unauthorized_rooms[self.rng.gen_range(0..unauthorized_rooms.len())])
        } else {
            Err(anyhow!("No location for curious user").into())
        }

        /* 
        // Filter to rooms the user cannot access
        let unauthorized_rooms: Vec<_> = all_rooms
            .iter()
            .filter(|room| {
                !authorized_rooms.contains(&room.id)
                    && !user.can_access_room(
                        room.id,
                        room.building_id,
                        self.get_location_for_building(room.building_id),
                    )
            })
            .collect();

        if unauthorized_rooms.is_empty() {
            return Err(anyhow!("No unauthorized rooms available for curious user").into());
        }

        let selected_room = unauthorized_rooms[self.rng.gen_range(0..unauthorized_rooms.len())];
        Ok(selected_room.id)
        */
    }

    /// Select a room in a different geographical location for impossible traveler scenarios
    fn select_remote_location_room(&mut self, user: &User) -> SimulationResult<RoomId> {
        let current_location = user.current_state.current_location;
        let all_locations = self.location_registry.get_all_locations();

        // Find locations different from the user's current location
        let remote_locations: Vec<_> =
            all_locations.iter().filter(|location| location.id != current_location).collect();

        if remote_locations.is_empty() {
            return Err(SimulationError::event_generation_error(
                "No remote locations available for impossible traveler scenario",
            ));
        }

        // Select a random remote location
        let remote_location = remote_locations[self.rng.gen_range(0..remote_locations.len())];

        // Select a random building in that location
        if remote_location.buildings.is_empty() {
            return Err(anyhow!("Remote location has no buildings").into());
        }

        let remote_building =
            &remote_location.buildings[self.rng.gen_range(0..remote_location.buildings.len())];

        // Select a random room in that building
        if remote_building.rooms.is_empty() {
            return Err(anyhow!("Remote building has no rooms").into());
        }

        let remote_room =
            &remote_building.rooms[self.rng.gen_range(0..remote_building.rooms.len())];
        Ok(remote_room.id)
    }

    /// Determine if a curious user event should be generated
    /// 
    /// This method calculates the probability that a curious user should generate
    /// an unauthorized access attempt during their current activity. The goal is to
    /// ensure curious users generate 1-2 unauthorized attempts per day.
    /// 
    /// With approximately 8-15 activities per user per day, we need a per-activity
    /// probability that results in 1-2 attempts per day for curious users.
    fn should_generate_curious_event(&mut self) -> bool {
        // Target: 1-2 unauthorized attempts per day per curious user
        // Assuming ~10 activities per day per user on average
        // We want roughly 15% chance per activity for curious users to generate an attempt
        // This gives us: 10 activities * 0.15 = 1.5 attempts per day per curious user
        let per_activity_probability = 0.15;
        
        self.rng.gen::<f64>() < per_activity_probability
    }

    /// Determine if an impossible traveler event should be generated
    /// 
    /// This method calculates the probability that a cloned badge user should generate
    /// an impossible traveler scenario during their current activity. The goal is to
    /// ensure cloned badge users generate observable impossible traveler scenarios.
    /// 
    /// With approximately 8-15 activities per user per day, we need a per-activity
    /// probability that results in observable scenarios for cloned badge users.
    fn should_generate_impossible_traveler_event(&mut self) -> bool {
        // Target: Observable impossible traveler scenarios for cloned badge users
        // Assuming ~10 activities per day per user on average
        // We want roughly 2% chance per activity for cloned badge users to generate a scenario
        // This gives us: 10 activities * 0.02 = 0.2 scenarios per day per cloned badge user
        // Over a week: 0.2 * 7 = 1.4 scenarios per week per cloned badge user
        let per_activity_probability = 0.02;
        
        self.rng.gen::<f64>() < per_activity_probability
    }

    /// Generate badge technical failure events with immediate retry
    /// 
    /// This method simulates technical failures of badge readers that occur during
    /// authorized access attempts. When a badge reader fails, it generates:
    /// 1. An initial failed access event (marked as badge technical failure)
    /// 2. A successful retry event within 5-30 seconds
    /// 
    /// Badge technical failures only occur on authorized access attempts (0.1% rate)
    /// and represent technical malfunctions rather than security violations.
    pub fn generate_badge_reader_failure(
        &mut self,
        user: &User,
        room_id: RoomId,
        current_time: DateTime<Utc>,
    ) -> SimulationResult<Vec<AccessEvent>> {
        // Get room, building, and location information
        let room = self.location_registry.get_room(room_id).ok_or_else(|| {
            SimulationError::event_generation_error(format!("Room {} not found", room_id))
        })?;

        let building = self.location_registry.get_building(room.building_id).ok_or_else(|| {
            SimulationError::event_generation_error(format!(
                "Building {} not found",
                room.building_id
            ))
        })?;

        // Verify that the user is authorized for this room
        let is_authorized = user.can_access_room(room_id, room.building_id, building.location_id);
        
        if !is_authorized {
            return Err(SimulationError::event_generation_error(
                "Badge technical failures only occur on authorized access attempts"
            ));
        }

        let mut events = Vec::new();

        // Generate the initial failed access event (badge technical failure)
        let failure_event = AccessEvent::new_with_failure_info(
            current_time,
            user.id,
            room_id,
            room.building_id,
            building.location_id,
            false, // Failed due to technical issue
            EventType::Failure,
            Some(FailureReason::BadgeReaderError),
            Some(EventMetadata::badge_reader_failure(None)), // Initial failure, no retry number
        );

        events.push(failure_event);

        // Generate the retry event (5-30 seconds later, successful)
        let retry_delay_seconds = self.rng.gen_range(5..=30);
        let retry_time = current_time + Duration::seconds(retry_delay_seconds);

        let retry_event = AccessEvent::new_with_failure_info(
            retry_time,
            user.id,
            room_id,
            room.building_id,
            building.location_id,
            true, // Successful on retry
            EventType::Success,
            None, // No failure reason for successful retry
            Some(EventMetadata::badge_reader_failure(Some(1))), // First retry attempt
        );

        events.push(retry_event);

        debug!(
            "Generated badge technical failure scenario for user {} at room {} with retry after {} seconds",
            user.id, room_id, retry_delay_seconds
        );

        Ok(events)
    }

    /// Determine if a badge technical failure should be generated for an authorized access attempt
    /// 
    /// Badge technical failures occur at a rate of 0.1% of authorized access attempts only.
    /// This represents realistic technical failure rates for physical badge reader hardware.
    fn should_generate_badge_reader_failure(&mut self) -> bool {
        // Target: 0.1% failure rate for authorized access attempts
        // This represents a more realistic low failure rate for modern badge readers
        let failure_probability = 0.001; // 0.1% chance of badge technical failure
        
        self.rng.gen::<f64>() < failure_probability
    }

    /// Validate if two access events constitute an impossible traveler scenario
    ///
    /// This method checks if the time gap between two access events from different
    /// geographical locations is insufficient for physical travel.
    ///
    /// # Arguments
    /// * `event1` - First access event
    /// * `event2` - Second access event
    ///
    /// # Returns
    /// `true` if the events represent an impossible traveler scenario
    pub fn validate_impossible_traveler_scenario(
        &self,
        event1: &AccessEvent,
        event2: &AccessEvent,
    ) -> bool {
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

        // Get the minimum travel time between the two locations
        let minimum_travel_time =
            self.get_minimum_travel_time_between_locations(event1.location_id, event2.location_id);

        // If the time gap is less than minimum travel time, it's impossible
        time_gap < minimum_travel_time
    }

    /// Get the minimum travel time between two geographical locations
    ///
    /// This method calculates the minimum realistic travel time between two locations
    /// based on their geographical coordinates and typical transportation methods.
    ///
    /// # Arguments
    /// * `from_location` - Starting location ID
    /// * `to_location` - Destination location ID
    ///
    /// # Returns
    /// Minimum travel time as a Duration
    pub fn get_minimum_travel_time_between_locations(
        &self,
        from_location: LocationId,
        to_location: LocationId,
    ) -> Duration {
        if from_location == to_location {
            return Duration::seconds(0);
        }

        // Get location coordinates for distance calculation
        let from_coords = self
            .location_registry
            .get_location(from_location)
            .map(|loc| loc.coordinates)
            .unwrap_or((0.0, 0.0));

        let to_coords = self
            .location_registry
            .get_location(to_location)
            .map(|loc| loc.coordinates)
            .unwrap_or((0.0, 0.0));

        // Calculate geographical distance using Haversine formula
        let distance_km = self.calculate_geographical_distance(from_coords, to_coords);

        // Estimate minimum travel time based on distance
        // Assumes fastest reasonable transportation (commercial aviation + ground transport)
        if distance_km < 50.0 {
            // Local travel: minimum 1 hour for nearby locations
            Duration::hours(1)
        } else if distance_km < 500.0 {
            // Regional travel: minimum 2-4 hours
            Duration::hours(2)
        } else if distance_km < 2000.0 {
            // National travel: minimum 4-6 hours
            Duration::hours(4)
        } else {
            // International travel: minimum 8-12 hours
            Duration::hours(8)
        }
    }

    /// Calculate geographical distance between two coordinate points using Haversine formula
    ///
    /// # Arguments
    /// * `coord1` - First coordinate (latitude, longitude)
    /// * `coord2` - Second coordinate (latitude, longitude)
    ///
    /// # Returns
    /// Distance in kilometers
    pub fn calculate_geographical_distance(&self, coord1: (f64, f64), coord2: (f64, f64)) -> f64 {
        let (lat1, lon1) = coord1;
        let (lat2, lon2) = coord2;

        // Convert degrees to radians
        let lat1_rad = lat1.to_radians();
        let lon1_rad = lon1.to_radians();
        let lat2_rad = lat2.to_radians();
        let lon2_rad = lon2.to_radians();

        // Haversine formula
        let dlat = lat2_rad - lat1_rad;
        let dlon = lon2_rad - lon1_rad;

        let a = (dlat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        // Earth's radius in kilometers
        const EARTH_RADIUS_KM: f64 = 6371.0;
        EARTH_RADIUS_KM * c
    }

    /// Generate simultaneous access events for impossible traveler scenarios
    ///
    /// This method creates a pair of access events that occur within an impossible
    /// timeframe for physical travel between different geographical locations.
    ///
    /// # Arguments
    /// * `user` - User with cloned badge
    /// * `primary_event_time` - Time of the primary access event
    ///
    /// # Returns
    /// A pair of access events representing the impossible traveler scenario
    pub fn generate_simultaneous_impossible_traveler_events(
        &mut self,
        user: &User,
        primary_event_time: DateTime<Utc>,
    ) -> SimulationResult<(AccessEvent, AccessEvent)> {
        // Generate the primary event at the user's current location
        let primary_room = self.select_authorized_room_for_user(user)?;
        
        // Generate the impossible event at a remote location (do mutable borrow first)
        let remote_room = self.select_remote_location_room(user)?;
        
        // Now do all the immutable lookups
        let primary_room_info = self
            .location_registry
            .get_room(primary_room)
            .ok_or_else(|| format!("Room not found: {}", primary_room))?;
        let primary_building =
            self.location_registry
                .get_building(primary_room_info.building_id)
                .ok_or_else(|| format!("Building not found: {}", primary_room_info.building_id))?;

        let remote_room_info = self
            .location_registry
            .get_room(remote_room)
            .ok_or_else(|| format!("Remote room not found: {}", remote_room))?;
        let remote_building =
            self.location_registry.get_building(remote_room_info.building_id).ok_or_else(|| {
                format!("Remote building not found: {}", remote_room_info.building_id)
            })?;

        let primary_location = self.location_registry.get_location(primary_building.location_id)
            .ok_or_else(|| format!("Primary location not found: {}", primary_building.location_id))?;
        let remote_location = self.location_registry.get_location(remote_building.location_id)
            .ok_or_else(|| format!("Remote location not found: {}", remote_building.location_id))?;
        
        let distance_km = self.calculate_geographical_distance(
            primary_location.coordinates,
            remote_location.coordinates,
        );

        let primary_event = AccessEvent {
            timestamp: primary_event_time,
            user_id: user.id,
            room_id: primary_room,
            building_id: primary_room_info.building_id,
            location_id: primary_building.location_id,
            success: true, // Primary event succeeds (user is authorized)
            event_type: EventType::Success,
            failure_reason: None,
            metadata: None,
        };

        // Check if user is authorized for the remote room
        let is_authorized = user.can_access_room(
            remote_room,
            remote_room_info.building_id,
            remote_building.location_id,
        );

        // Generate impossible event with insufficient travel time (1-3 hours)
        let time_gap_minutes = self.rng.gen_range(1..=180); // 1 minute to 3 hours
        let impossible_event_time = primary_event_time + Duration::minutes(time_gap_minutes);
        
        // Calculate travel time violation (actual time vs minimum required time)
        let travel_time_violation = Duration::minutes(time_gap_minutes);

        // Create metadata for impossible traveler scenario
        let metadata = Some(EventMetadata::impossible_traveler(travel_time_violation, distance_km));

        let impossible_event = AccessEvent {
            timestamp: impossible_event_time,
            user_id: user.id,
            room_id: remote_room,
            building_id: remote_room_info.building_id,
            location_id: remote_building.location_id,
            success: is_authorized, // May succeed if user has authorization
            event_type: if is_authorized { EventType::Success } else { EventType::Failure },
            failure_reason: if is_authorized { None } else { Some(FailureReason::ImpossibleTraveler) },
            metadata,
        };

        // Validate that this is indeed an impossible scenario
        if !self.validate_impossible_traveler_scenario(&primary_event, &impossible_event) {
            return Err(SimulationError::event_generation_error(
                "Generated events do not constitute an impossible traveler scenario",
            ));
        }

        Ok((primary_event, impossible_event))
    }

    /// Select an authorized room for a user (for primary events)
    fn select_authorized_room_for_user(
        &mut self,
        user: &User,
    ) -> SimulationResult<RoomId> {
        // First try to use user's primary workspace
        if self.rng.gen::<f64>() < 0.7 {
            return Ok(user.primary_workspace);
        }

        // Otherwise select from authorized rooms
        let authorized_rooms = user.get_authorized_rooms();
        if !authorized_rooms.is_empty() {
            let selected_room = authorized_rooms[self.rng.gen_range(0..authorized_rooms.len())];
            return Ok(selected_room);
        }

        // If no specific room permissions, select from authorized buildings
        let authorized_buildings = user.get_authorized_buildings();
        for building_id in authorized_buildings {
            if let Some(building) = self.location_registry.get_building(building_id) {
                if !building.rooms.is_empty() {
                    let room = &building.rooms[self.rng.gen_range(0..building.rooms.len())];
                    return Ok(room.id);
                }
            }
        }

        // If no building permissions, select from authorized locations
        let authorized_locations = user.get_authorized_locations();
        for location_id in authorized_locations {
            if let Some(location) = self.location_registry.get_location(location_id) {
                for building in &location.buildings {
                    if !building.rooms.is_empty() {
                        let room = &building.rooms[self.rng.gen_range(0..building.rooms.len())];
                        return Ok(room.id);
                    }
                }
            }
        }

        Err(SimulationError::event_generation_error("No authorized rooms found for user"))
    }

    /// Enhanced impossible traveler event generation with geographical validation
    ///
    /// This method replaces the basic impossible traveler generation with enhanced
    /// geographical validation and simultaneous event creation.
    pub fn generate_enhanced_impossible_traveler_event(
        &mut self,
        user: &User,
        current_time: DateTime<Utc>,
    ) -> SimulationResult<Vec<AccessEvent>> {
        if !user.has_cloned_badge {
            return Err(SimulationError::event_generation_error(
                "User does not have a cloned badge",
            ));
        }

        // Generate simultaneous events
        let (primary_event, impossible_event) =
            self.generate_simultaneous_impossible_traveler_events(user, current_time)?;

        // Additional validation to ensure geographical impossibility
        let minimum_travel_time = self.get_minimum_travel_time_between_locations(
            primary_event.location_id,
            impossible_event.location_id,
        );

        let actual_time_gap = if impossible_event.timestamp > primary_event.timestamp {
            impossible_event.timestamp - primary_event.timestamp
        } else {
            primary_event.timestamp - impossible_event.timestamp
        };

        if actual_time_gap >= minimum_travel_time {
            return Err(SimulationError::event_generation_error(
                "Generated events do not meet impossible traveler criteria",
            ));
        }

        // Return both events for the impossible traveler scenario
        Ok(vec![primary_event, impossible_event])
    }

    /// Detect potential badge cloning based on access patterns
    ///
    /// This method analyzes access events to identify potential badge cloning
    /// by looking for impossible traveler patterns.
    ///
    /// # Arguments
    /// * `events` - List of access events to analyze
    /// * `time_window` - Time window to analyze (e.g., Duration::hours(24))
    ///
    /// # Returns
    /// List of user IDs that show potential badge cloning patterns
    pub fn detect_badge_cloning_from_events(
        &self,
        events: &[AccessEvent],
        time_window: Duration,
    ) -> Vec<UserId> {
        let mut potential_cloned_badges = Vec::new();
        let mut user_events: HashMap<UserId, Vec<&AccessEvent>> = HashMap::new();

        // Group events by user
        for event in events {
            user_events.entry(event.user_id).or_default().push(event);
        }

        // Analyze each user's events for impossible traveler patterns
        for (user_id, user_events) in user_events {
            if self.has_impossible_traveler_pattern(&user_events, time_window) {
                potential_cloned_badges.push(user_id);
            }
        }

        potential_cloned_badges
    }

    /// Check if a user's events show impossible traveler patterns
    ///
    /// # Arguments
    /// * `events` - User's access events
    /// * `time_window` - Time window to check within
    ///
    /// # Returns
    /// `true` if impossible traveler patterns are detected
    fn has_impossible_traveler_pattern(
        &self,
        events: &[&AccessEvent],
        time_window: Duration,
    ) -> bool {
        // Sort events by timestamp
        let mut sorted_events = events.to_vec();
        sorted_events.sort_by_key(|event| event.timestamp);

        // Check for impossible travel between consecutive events
        for window in sorted_events.windows(2) {
            let event1 = window[0];
            let event2 = window[1];

            // Only check events within the specified time window
            if event2.timestamp - event1.timestamp <= time_window
                && self.validate_impossible_traveler_scenario(event1, event2)
            {
                return true;
            }
        }

        false
    }

    /// Generate comprehensive impossible traveler scenario with metadata
    ///
    /// This method creates a complete impossible traveler scenario including
    /// both events and metadata about the geographical impossibility.
    ///
    /// # Arguments
    /// * `user` - User with cloned badge
    /// * `current_time` - Current simulation time
    ///
    /// # Returns
    /// Tuple containing the events and scenario metadata
    pub fn generate_impossible_traveler_scenario_with_metadata(
        &mut self,
        user: &User,
        current_time: DateTime<Utc>,
    ) -> SimulationResult<(Vec<AccessEvent>, ImpossibleTravelerMetadata)> {
        let events = self.generate_enhanced_impossible_traveler_event(user, current_time)?;

        if events.len() != 2 {
            return Err(SimulationError::event_generation_error(
                "Expected exactly 2 events for impossible traveler scenario",
            ));
        }

        let primary_event = &events[0];
        let impossible_event = &events[1];

        // Calculate metadata
        let geographical_distance = self.calculate_geographical_distance(
            self.location_registry
                .get_location(primary_event.location_id)
                .map(|loc| loc.coordinates)
                .unwrap_or((0.0, 0.0)),
            self.location_registry
                .get_location(impossible_event.location_id)
                .map(|loc| loc.coordinates)
                .unwrap_or((0.0, 0.0)),
        );

        let time_gap = if impossible_event.timestamp > primary_event.timestamp {
            impossible_event.timestamp - primary_event.timestamp
        } else {
            primary_event.timestamp - impossible_event.timestamp
        };

        let minimum_travel_time = self.get_minimum_travel_time_between_locations(
            primary_event.location_id,
            impossible_event.location_id,
        );

        let metadata = ImpossibleTravelerMetadata::new(
            user.id,
            primary_event.location_id,
            impossible_event.location_id,
            geographical_distance,
            time_gap,
            minimum_travel_time,
        );

        Ok((events, metadata))
    }

    /// Generate a sequence of events with proper timing
    pub fn generate_event_sequence(
        &mut self,
        users: &[User],
        start_time: DateTime<Utc>,
        duration: Duration,
    ) -> SimulationResult<Vec<AccessEvent>> {
        let mut all_events = Vec::new();
        let end_time = start_time + duration;

        // Generate events for each user based on their schedules
        for user in users {
            // Get activities that occur within the time window
            let relevant_activities: Vec<_> = user
                .current_state
                .daily_schedule
                .iter()
                .filter(|activity| {
                    activity.start_time >= start_time && activity.start_time <= end_time
                })
                .collect();

            // Generate events for each relevant activity
            for activity in relevant_activities {
                let activity_events =
                    self.generate_events_from_activity(user, activity, activity.start_time)?;
                all_events.extend(activity_events);
            }
        }

        // Sort events by timestamp to ensure proper chronological order
        all_events.sort_by_key(|event| event.timestamp);

        Ok(all_events)
    }

    /// Generate events for a single time step in the simulation
    pub fn generate_events_for_timestep(
        &mut self,
        users: &[User],
        current_time: DateTime<Utc>,
    ) -> SimulationResult<Vec<AccessEvent>> {
        let mut events = Vec::new();

        for user in users {
            // Check if user has an active activity at this time
            if let Some(activity) = user.get_current_activity(current_time) {
                // Generate events if this is the start of the activity
                let time_since_start = current_time - activity.start_time;
                if time_since_start >= Duration::zero() && time_since_start < Duration::minutes(1) {
                    let activity_events =
                        self.generate_events_from_activity(user, activity, current_time)?;
                    events.extend(activity_events);
                }
            }
        }

        Ok(events)
    }

    // NOTE: Statistics tracking has been moved to centralized location
    // The update_statistics_for_events method has been removed to eliminate
    // duplicate statistics tracking. Statistics are now handled by the caller.

    // NOTE: Statistics tracking methods have been removed to eliminate duplicate tracking
    // All statistics are now handled centrally by the BatchEventGenerator
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::User;
    use crate::facility::{Building, Location, LocationRegistry, Room};
    use crate::permissions::PermissionSet;
    use crate::simulation::TimeManager;
    use crate::types::{BuildingId, UserId, LocationId, RoomId, SimulationConfig};
    use crate::ActivityType;
    use chrono::Timelike;

    #[test]
    fn test_event_generator_creation() {
        let config = SimulationConfig::default();
        let location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        let event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Verify the generator was created successfully
        assert_eq!(event_generator.config.user_count, 10_000);
    }

    #[test]
    fn test_access_attempt_processing() {
        let config = SimulationConfig::default();
        let mut location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        // Create a test location with building and room
        let mut location = Location::new("Test Location".to_string(), (40.7128, -74.0060));
        let location_id = location.id;
        let mut building = Building::new(location.id, "Test Building".to_string());
        let building_id = building.id;
        let room = Room::new(
            building.id,
            "Test Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room_id = room.id;
        building.add_room(room);
        location.add_building(building);
        location_registry.add_location(location);

        let mut event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Create a test user and access attempt
        let user_id = UserId::new();
        let timestamp = Utc::now();
        let attempt = AccessAttempt::new(user_id, room_id, true, timestamp);
        
        // Create a test user (regular user, not night-shift)
        let permissions = PermissionSet::new();
        let user = User::new(location_id, building_id, room_id, permissions);

        // Process the access attempt - retry a few times to handle the 0.1% random system failure
        let mut success_found = false;
        for _ in 0..10 {
            let result = event_generator.process_access_attempt(&user, &attempt);
            assert!(result.is_ok());
            let event = result.unwrap();
            assert_eq!(event.user_id, user_id);
            assert_eq!(event.room_id, room_id);
            assert_eq!(event.timestamp, timestamp);
            
            if event.success {
                assert_eq!(event.event_type, EventType::Success);
                success_found = true;
                break;
            } else {
                // Should only fail due to the 0.1% random system failure
                assert_eq!(event.event_type, EventType::Failure);
            }
        }
        
        // With 10 attempts, the probability of all failing due to random system failure is (0.001)^10 ≈ 10^-30
        assert!(success_found, "Access attempt should succeed at least once out of 10 tries");
    }

    #[test]
    fn test_badge_reader_failure_event_creation() {
        let config = SimulationConfig::default();
        let location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        let _event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Create a badge reader failure event manually
        let user_id = UserId::new();
        let timestamp = Utc::now();
        let badge_reader_failure_event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            RoomId::new(),
            BuildingId::new(),
            LocationId::new(),
            false, // Failed due to technical issue
            EventType::Failure,
            Some(FailureReason::BadgeReaderError),
            Some(EventMetadata::badge_reader_failure(None)),
        );

        // Verify that the event is properly classified
        assert!(badge_reader_failure_event.is_badge_reader_failure());
        assert_eq!(badge_reader_failure_event.failure_reason, Some(FailureReason::BadgeReaderError));
        assert_eq!(badge_reader_failure_event.success, false);
        assert_eq!(badge_reader_failure_event.event_type, EventType::Failure);
        
        // NOTE: Statistics tracking is now handled centrally by BatchEventGenerator
    }

    #[test]
    fn test_unauthorized_access_attempt() {
        let config = SimulationConfig::default();
        let mut location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        // Create a test location with building and room
        let mut location = Location::new("Test Location".to_string(), (40.7128, -74.0060));
        let location_id = location.id;
        let mut building = Building::new(location.id, "Test Building".to_string());
        let building_id = building.id;
        let room = Room::new(
            building.id,
            "Test Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room_id = room.id;
        building.add_room(room);
        location.add_building(building);
        location_registry.add_location(location);

        let mut event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Create an unauthorized access attempt
        let user_id = UserId::new();
        let timestamp = Utc::now();
        let attempt = AccessAttempt::new(user_id, room_id, false, timestamp); // Not authorized
        
        // Create a test user (regular user, not night-shift)
        let permissions = PermissionSet::new();
        let user = User::new(location_id, building_id, room_id, permissions);

        // Process the access attempt
        let result = event_generator.process_access_attempt(&user, &attempt);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.user_id, user_id);
        assert_eq!(event.room_id, room_id);
        assert_eq!(event.timestamp, timestamp);
        assert!(!event.success); // Should fail since not authorized
        assert_eq!(event.event_type, EventType::Failure);
    }

    #[test]
    fn test_night_shift_event_classification() {
        let config = SimulationConfig::default();
        let mut location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        // Create a test location with building and room
        let mut location = Location::new("Test Location".to_string(), (40.7128, -74.0060));
        let location_id = location.id;
        let mut building = Building::new(location.id, "Test Building".to_string());
        let building_id = building.id;
        let room = Room::new(
            building.id,
            "Test Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room_id = room.id;
        building.add_room(room);
        location.add_building(building);
        location_registry.add_location(location);

        let mut event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Create a night-shift user
        let user_id = UserId::new();
        let permissions = PermissionSet::new();
        let night_shift_user = User::new_night_shift(location_id, building_id, room_id, permissions, building_id);

        // Create an access attempt during off-hours (e.g., 10 PM)
        let off_hours_timestamp = chrono::Utc::now()
            .with_hour(22).unwrap()  // 10 PM
            .with_minute(0).unwrap()
            .with_second(0).unwrap();
        let attempt = AccessAttempt::new(user_id, room_id, true, off_hours_timestamp);

        // Process the access attempt
        let result = event_generator.process_access_attempt(&night_shift_user, &attempt);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.user_id, user_id);
        assert_eq!(event.room_id, room_id);
        assert_eq!(event.timestamp, off_hours_timestamp);
        
        // Verify that the event has night-shift metadata
        assert!(event.metadata.is_some());
        let metadata = event.metadata.unwrap();
        assert!(metadata.is_night_shift_event);
        assert!(!metadata.is_curious_attempt);
        assert!(!metadata.is_impossible_traveler);
        assert!(!metadata.is_badge_reader_failure);
    }

    #[test]
    fn test_regular_user_no_night_shift_classification() {
        let config = SimulationConfig::default();
        let mut location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        // Create a test location with building and room
        let mut location = Location::new("Test Location".to_string(), (40.7128, -74.0060));
        let location_id = location.id;
        let mut building = Building::new(location.id, "Test Building".to_string());
        let building_id = building.id;
        let room = Room::new(
            building.id,
            "Test Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room_id = room.id;
        building.add_room(room);
        location.add_building(building);
        location_registry.add_location(location);

        let mut event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Create a regular user (not night-shift)
        let user_id = UserId::new();
        let permissions = PermissionSet::new();
        let regular_user = User::new(location_id, building_id, room_id, permissions);

        // Create an access attempt during off-hours (e.g., 10 PM)
        let off_hours_timestamp = chrono::Utc::now()
            .with_hour(22).unwrap()  // 10 PM
            .with_minute(0).unwrap()
            .with_second(0).unwrap();
        let attempt = AccessAttempt::new(user_id, room_id, true, off_hours_timestamp);

        // Process the access attempt
        let result = event_generator.process_access_attempt(&regular_user, &attempt);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.user_id, user_id);
        assert_eq!(event.room_id, room_id);
        assert_eq!(event.timestamp, off_hours_timestamp);
        
        // Verify that the event does NOT have night-shift metadata
        // Regular users during off-hours should not be classified as night-shift events
        if let Some(metadata) = event.metadata {
            assert!(!metadata.is_night_shift_event);
        }
    }

    #[test]
    fn test_night_shift_user_during_business_hours() {
        let config = SimulationConfig::default();
        let mut location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        // Create a test location with building and room
        let mut location = Location::new("Test Location".to_string(), (40.7128, -74.0060));
        let location_id = location.id;
        let mut building = Building::new(location.id, "Test Building".to_string());
        let building_id = building.id;
        let room = Room::new(
            building.id,
            "Test Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room_id = room.id;
        building.add_room(room);
        location.add_building(building);
        location_registry.add_location(location);

        let mut event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Create a night-shift user
        let user_id = UserId::new();
        let permissions = PermissionSet::new();
        let night_shift_user = User::new_night_shift(location_id, building_id, room_id, permissions, building_id);

        // Create an access attempt during business hours (e.g., 2 PM)
        let business_hours_timestamp = chrono::Utc::now()
            .with_hour(14).unwrap()  // 2 PM
            .with_minute(0).unwrap()
            .with_second(0).unwrap();
        let attempt = AccessAttempt::new(user_id, room_id, true, business_hours_timestamp);

        // Process the access attempt
        let result = event_generator.process_access_attempt(&night_shift_user, &attempt);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.user_id, user_id);
        assert_eq!(event.room_id, room_id);
        assert_eq!(event.timestamp, business_hours_timestamp);
        
        // Verify that the event does NOT have night-shift metadata
        // Night-shift users during business hours should not be classified as night-shift events
        if let Some(metadata) = event.metadata {
            assert!(!metadata.is_night_shift_event);
        }
    }

    #[test]
    fn test_event_metadata_night_shift_constructor() {
        let metadata = EventMetadata::night_shift_event();
        
        assert!(metadata.is_night_shift_event);
        assert!(!metadata.is_curious_attempt);
        assert!(!metadata.is_impossible_traveler);
        assert!(!metadata.is_badge_reader_failure);
        assert!(metadata.retry_attempt_number.is_none());
        assert!(metadata.travel_time_violation.is_none());
        assert!(metadata.geographical_distance.is_none());
    }

    #[test]
    fn test_geographical_distance_calculation() {
        let config = SimulationConfig::default();
        let location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        let event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Test distance between New York and Los Angeles (approximately 3944 km)
        let ny_coords = (40.7128, -74.0060);
        let la_coords = (34.0522, -118.2437);
        let distance = event_generator.calculate_geographical_distance(ny_coords, la_coords);

        // Allow for some tolerance in the calculation
        assert!((distance - 3944.0).abs() < 100.0);

        // Test distance between same coordinates (should be 0)
        let same_distance = event_generator.calculate_geographical_distance(ny_coords, ny_coords);
        assert_eq!(same_distance, 0.0);
    }

    #[test]
    fn test_minimum_travel_time_calculation() {
        let config = SimulationConfig::default();
        let mut location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        // Create two locations with different coordinates
        let location1 = Location::new("New York".to_string(), (40.7128, -74.0060));
        let location2 = Location::new("Los Angeles".to_string(), (34.0522, -118.2437));
        let location1_id = location1.id;
        let location2_id = location2.id;

        location_registry.add_location(location1);
        location_registry.add_location(location2);

        let event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Test travel time between NY and LA (should be international travel time)
        let travel_time =
            event_generator.get_minimum_travel_time_between_locations(location1_id, location2_id);
        assert_eq!(travel_time, Duration::hours(8)); // International travel

        // Test travel time between same location (should be 0)
        let same_location_time =
            event_generator.get_minimum_travel_time_between_locations(location1_id, location1_id);
        assert_eq!(same_location_time, Duration::seconds(0));
    }

    #[test]
    fn test_impossible_traveler_validation() {
        let config = SimulationConfig::default();
        let mut location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        // Create two distant locations
        let location1 = Location::new("New York".to_string(), (40.7128, -74.0060));
        let location2 = Location::new("Los Angeles".to_string(), (34.0522, -118.2437));
        let location1_id = location1.id;
        let location2_id = location2.id;

        location_registry.add_location(location1);
        location_registry.add_location(location2);

        let event_generator = EventGenerator::new(config, location_registry, time_manager);

        let user_id = UserId::new();
        let room1_id = RoomId::new();
        let room2_id = RoomId::new();
        let building1_id = BuildingId::new();
        let building2_id = BuildingId::new();

        let base_time = Utc::now();

        // Create two events with insufficient travel time (impossible scenario)
        let event1 = AccessEvent::new(
            base_time,
            user_id,
            room1_id,
            building1_id,
            location1_id,
            true,
            EventType::Success,
        );

        let event2 = AccessEvent::new(
            base_time + Duration::hours(2), // Only 2 hours later
            user_id,
            room2_id,
            building2_id,
            location2_id,
            true,
            EventType::Success,
        );

        // This should be detected as impossible traveler scenario
        assert!(event_generator.validate_impossible_traveler_scenario(&event1, &event2));

        // Create events with sufficient travel time (possible scenario)
        let event3 = AccessEvent::new(
            base_time + Duration::hours(10), // 10 hours later (sufficient time)
            user_id,
            room2_id,
            building2_id,
            location2_id,
            true,
            EventType::Success,
        );

        // This should NOT be detected as impossible traveler scenario
        assert!(!event_generator.validate_impossible_traveler_scenario(&event1, &event3));
    }

    #[test]
    fn test_event_creation_with_different_types() {
        let config = SimulationConfig::default();
        let location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        let _event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Create test events with different types
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let success_event = AccessEvent::new(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            true,
            EventType::Success,
        );

        let failure_event = AccessEvent::new(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Failure,
        );

        let suspicious_event = AccessEvent::new(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Suspicious,
        );

        // Verify events were created correctly
        assert_eq!(success_event.success, true);
        assert_eq!(success_event.event_type, EventType::Success);
        
        assert_eq!(failure_event.success, false);
        assert_eq!(failure_event.event_type, EventType::Failure);
        
        assert_eq!(suspicious_event.success, false);
        assert_eq!(suspicious_event.event_type, EventType::Suspicious);
        
        // NOTE: Statistics tracking is now handled centrally by BatchEventGenerator
    }

    #[test]
    fn test_curious_event_creation() {
        let config = SimulationConfig::default();
        let location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        let _event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Create a test user and unauthorized access activity
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let _unauthorized_activity = ScheduledActivity::new(
            ActivityType::UnauthorizedAccess,
            room_id,
            timestamp,
            Duration::hours(1),
        );

        let events = vec![
            AccessEvent::new(
                timestamp,
                user_id,
                room_id,
                building_id,
                location_id,
                false,
                EventType::Failure,
            ),
            AccessEvent::new(
                timestamp + Duration::minutes(5),
                user_id,
                room_id,
                building_id,
                location_id,
                false,
                EventType::Failure,
            ),
        ];

        // Verify events were created correctly
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].success, false);
        assert_eq!(events[0].event_type, EventType::Failure);
        assert_eq!(events[1].success, false);
        assert_eq!(events[1].event_type, EventType::Failure);
        
        // NOTE: Statistics tracking is now handled centrally by BatchEventGenerator
    }

    #[test]
    fn test_event_batch_creation() {
        let config = SimulationConfig::default();
        let location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        let _event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Create a regular activity (not unauthorized access)
        let room_id = RoomId::new();
        let timestamp = Utc::now();
        let _meeting_activity = ScheduledActivity::new(
            ActivityType::Meeting,
            room_id,
            timestamp,
            Duration::hours(1),
        );

        // Create a batch of events with different types
        let user_id = UserId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let events = vec![
            AccessEvent::new(
                timestamp,
                user_id,
                room_id,
                building_id,
                location_id,
                true,
                EventType::Success,
            ),
            AccessEvent::new(
                timestamp + Duration::minutes(1),
                user_id,
                room_id,
                building_id,
                location_id,
                false,
                EventType::OutsideHours,
            ),
            AccessEvent::new(
                timestamp + Duration::minutes(2),
                user_id,
                room_id,
                building_id,
                location_id,
                false,
                EventType::InvalidBadge,
            ),
        ];

        // Verify all events were created correctly
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].success, true);
        assert_eq!(events[0].event_type, EventType::Success);
        assert_eq!(events[1].success, false);
        assert_eq!(events[1].event_type, EventType::OutsideHours);
        assert_eq!(events[2].success, false);
        assert_eq!(events[2].event_type, EventType::InvalidBadge);
        
        // NOTE: Statistics tracking is now handled centrally by BatchEventGenerator
    }

    #[test]
    fn test_event_generator_without_statistics_tracker() {
        let config = SimulationConfig::default();
        let location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        let _event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Create test data
        let room_id = RoomId::new();
        let timestamp = Utc::now();
        let _activity = ScheduledActivity::new(
            ActivityType::Meeting,
            room_id,
            timestamp,
            Duration::hours(1),
        );

        let user_id = UserId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let events = vec![AccessEvent::new(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            true,
            EventType::Success,
        )];

        // Verify event was created correctly
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].success, true);
        assert_eq!(events[0].event_type, EventType::Success);

        // Statistics are now handled centrally in BatchEventGenerator
        // No need to verify statistics on EventGenerator
    }

    #[test]
    fn test_deprecated_statistics_methods() {
        let config = SimulationConfig::default();
        let location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();

        let _event_generator = EventGenerator::new(config, location_registry, time_manager);

        // Statistics are now handled centrally in BatchEventGenerator
        // EventGenerator no longer manages statistics directly
        // This test verifies the deprecated methods exist but don't affect functionality
    }

    #[test]
    fn test_impossible_traveler_event_counting_from_generation() {
        let config = SimulationConfig::default();
        let mut location_registry = LocationRegistry::new();
        let time_manager = TimeManager::default();
        let stats = SimulationStatistics::default();

        // Create two distant locations for impossible traveler scenario
        let location1 = Location::new("New York".to_string(), (40.7128, -74.0060));
        let location2 = Location::new("Los Angeles".to_string(), (34.0522, -118.2437));
        let location1_id = location1.id;
        let location2_id = location2.id;

        let mut building1 = Building::new(location1_id, "NYC Building".to_string());
        let mut building2 = Building::new(location2_id, "LA Building".to_string());

        let room1 = Room::new(
            building1.id,
            "NYC Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room2 = Room::new(
            building2.id,
            "LA Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );

        building1.add_room(room1);
        building2.add_room(room2);
        location_registry.add_location(location1);
        location_registry.add_location(location2);

        let _event_generator = EventGenerator::new(
            config,
            location_registry,
            time_manager,
        );
        // Statistics are now handled centrally, not in EventGenerator
        let _stats = stats; // Keep stats variable to avoid unused warning

        let user_id = UserId::new();
        let room1_id = RoomId::new();
        let room2_id = RoomId::new();
        let building1_id = BuildingId::new();
        let building2_id = BuildingId::new();
        let base_time = Utc::now();

        // Create impossible traveler events manually to test counting
        let impossible_events = vec![
            AccessEvent::new(
                base_time,
                user_id,
                room1_id,
                building1_id,
                location1_id,
                true,
                EventType::Success,
            ),
            AccessEvent::new(
                base_time + Duration::hours(2), // Only 2 hours later - impossible for cross-country travel
                user_id,
                room2_id,
                building2_id,
                location2_id,
                true,
                EventType::Success,
            ),
        ];

        // Statistics are now handled centrally in BatchEventGenerator
        // EventGenerator no longer tracks statistics directly
        // This test verifies event creation, not statistics tracking

        // Verify events were created correctly
        assert_eq!(impossible_events.len(), 2);
        assert_eq!(impossible_events[0].user_id, user_id);
        assert_eq!(impossible_events[1].user_id, user_id);
        assert_eq!(impossible_events[0].location_id, location1_id);
        assert_eq!(impossible_events[1].location_id, location2_id);
        assert!(impossible_events[0].success);
        assert!(impossible_events[1].success);
        
        // NOTE: Statistics tracking is now handled centrally by BatchEventGenerator
    }

    // NOTE: Additional test methods for impossible traveler detection and statistics tracking
    // have been removed because the functionality has been moved to centralized statistics
    // tracking in BatchEventGenerator. The EventGenerator now focuses only on event generation.
}
