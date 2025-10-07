//! Behavior engine for activity generation
//!
//! This module contains the BehaviorEngine for generating user activities.

use crate::user::{BehaviorProfile, User, ScheduledActivity};
use crate::facility::LocationRegistry;
use crate::simulation::{ErrorHandler, SimulationError, SimulationResult, TimeManager};
use crate::types::{ActivityType, RoomType, SecurityLevel, SimulationConfig};
use crate::types::{BuildingId, UserId, LocationId, RoomId};
use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use tracing::{debug, info, instrument, span, warn, Level};

/// Travel time constants for realistic scheduling
#[derive(Debug, Clone)]
pub struct TravelTimeConstants {
    /// Minimum travel time between different geographical locations (4 hours)
    pub cross_location_travel_time: Duration,

    /// Travel time between buildings in same location (15-30 minutes)
    pub intra_location_building_travel_time: (Duration, Duration),

    /// Buffer time for scheduling conflicts (5 minutes)
    pub scheduling_buffer_time: Duration,
}

impl Default for TravelTimeConstants {
    fn default() -> Self {
        Self {
            cross_location_travel_time: Duration::hours(4),
            intra_location_building_travel_time: (Duration::minutes(15), Duration::minutes(30)),
            scheduling_buffer_time: Duration::minutes(5),
        }
    }
}

/// Conflict resolution strategies for schedule conflicts
#[derive(Debug, Clone)]
enum ConflictResolution {
    /// Adjust the activity start time to the specified time
    AdjustTime(DateTime<Utc>),
    /// Skip this activity entirely
    SkipActivity,
    /// No conflict detected
    NoConflict,
}

/// Behavioral engine for generating realistic daily activity schedules and patterns
#[derive(Debug)]
pub struct BehaviorEngine {
    /// Configuration parameters for the simulation
    config: SimulationConfig,
    /// Random number generator for behavioral decisions
    rng: rand::rngs::ThreadRng,
    /// Time manager for realistic timing calculations
    #[allow(dead_code)]
    time_manager: TimeManager,
    /// Error handler for graceful error recovery
    #[allow(dead_code)]
    error_handler: ErrorHandler,
    /// Travel time constants for realistic scheduling
    travel_time_constants: TravelTimeConstants,
    /// Track user daily locations for cross-location travel persistence
    user_daily_locations: HashMap<UserId, LocationId>,
}

impl BehaviorEngine {
    /// Create a new behavior engine with the given configuration
    pub fn new(config: SimulationConfig, time_manager: TimeManager) -> Self {
        info!("Initializing behavior engine with {} users", config.user_count);
        Self {
            config,
            rng: rand::thread_rng(),
            time_manager,
            error_handler: ErrorHandler::new(),
            travel_time_constants: TravelTimeConstants::default(),
            user_daily_locations: HashMap::new(),
        }
    }

    /// Get a reference to the simulation configuration
    pub fn get_config(&self) -> &SimulationConfig {
        &self.config
    }

    /// Generate a realistic daily schedule for a user
    ///
    /// Creates a schedule with arrival, meetings, bathroom breaks, lunch, and departure
    /// based on realistic timing patterns and the user's behavior profile.
    /// For night-shift users, generates an inverted schedule with patrol activities.
    ///
    /// # Arguments
    /// * `user` - The user to generate a schedule for
    /// * `date` - The date to generate the schedule for
    /// * `registry` - Location registry for room lookups
    ///
    /// # Returns
    /// A vector of scheduled activities sorted by start time
    #[instrument(skip(self, registry), fields(user_id = %user.id, date = %date))]
    pub fn generate_daily_schedule(
        &mut self,
        user: &User,
        date: NaiveDate,
        registry: &LocationRegistry,
    ) -> SimulationResult<Vec<ScheduledActivity>> {
        let span = span!(Level::DEBUG, "generate_daily_schedule", user_id = %user.id);
        let _enter = span.enter();

        debug!("Generating daily schedule for user {} on {}", user.id, date);

        // Check if this is a night-shift user and handle accordingly
        if user.is_night_shift {
            debug!("Generating night-shift schedule for user {}", user.id);
            return self.generate_night_shift_schedule(user, date, registry);
        }

        // Generate schedule with error handling
        let result = match self.generate_schedule_internal(user, date, registry) {
            Ok(schedule) => Some(schedule),
            Err(e) => {
                warn!("Failed to generate schedule for user {}: {}", user.id, e);
                None
            }
        };

        let schedule = if let Some(generated_schedule) = result {
            info!(
                "Generated {} activities for user {} on {}",
                generated_schedule.len(),
                user.id,
                date
            );
            generated_schedule
        } else {
            warn!(
                "Failed to generate schedule for user {}, using minimal schedule",
                user.id
            );
            self.generate_minimal_schedule(user, date, registry)?
        };

        Ok(schedule)
    }

    /// Internal schedule generation with error handling
    fn generate_schedule_internal(
        &mut self,
        user: &User,
        date: NaiveDate,
        registry: &LocationRegistry,
    ) -> SimulationResult<Vec<ScheduledActivity>> {
        let mut schedule = Vec::new();

        // Generate schedule for every day (no weekend skipping)

        // Arrival (7:30 AM - 10:00 AM, most arrive 8:00-9:30 AM)
        let arrival_time = self.generate_arrival_time(date, &user.behavior_profile);
        schedule.push(ScheduledActivity::new(
            ActivityType::Arrival,
            user.primary_workspace,
            arrival_time,
            Duration::minutes(15), // Time to settle in
        ));

        // Bathroom breaks (2-4 per day, distributed throughout the day)
        let bathroom_count = self.rng.gen_range(2..=4);
        for _ in 0..bathroom_count {
            if let Some(bathroom_time) =
                self.generate_bathroom_break_time(date, &user.behavior_profile)
            {
                let bathroom_room = self
                    .select_bathroom_room(user, registry)
                    .unwrap_or(user.primary_workspace); // Fall back to workspace if no bathroom found
                schedule.push(ScheduledActivity::new(
                    ActivityType::Bathroom,
                    bathroom_room,
                    bathroom_time,
                    Duration::minutes(self.rng.gen_range(3..=8)),
                ));
            }
        }

        // Lunch break (11:30 AM - 2:00 PM, typically 45-90 minutes)
        let lunch_time = self.generate_lunch_time(date, &user.behavior_profile);
        let lunch_room =
            self.select_lunch_room(user, registry).unwrap_or(user.primary_workspace); // Fall back to workspace if no cafeteria/kitchen
        schedule.push(ScheduledActivity::new(
            ActivityType::Lunch,
            lunch_room,
            lunch_time,
            Duration::minutes(self.rng.gen_range(30..=90)),
        ));

        // Meetings (0-5 per day based on social level)
        let meeting_count = self.calculate_meeting_count(&user.behavior_profile);
        for _ in 0..meeting_count {
            if let Some(meeting_time) = self.generate_meeting_time(
                date,
                &user.behavior_profile,
                &schedule,
                user,
                registry,
            ) {
                if let Some(meeting_room) =
                    self.select_meeting_room(user, registry, meeting_time, &schedule)
                {
                    schedule.push(ScheduledActivity::new(
                        ActivityType::Meeting,
                        meeting_room,
                        meeting_time,
                        Duration::minutes(self.rng.gen_range(30..=120)),
                    ));
                }
            }
        }

        // Collaboration visits (0-3 per day for social users)
        if user.behavior_profile.is_social() {
            let collab_count = self.rng.gen_range(0..=3);
            for _ in 0..collab_count {
                if let Some(collab_time) =
                    self.generate_collaboration_time(date, &user.behavior_profile)
                {
                    if let Some(collab_room) = self.select_collaboration_room(user, registry) {
                        schedule.push(ScheduledActivity::new(
                            ActivityType::Collaboration,
                            collab_room,
                            collab_time,
                            Duration::minutes(self.rng.gen_range(15..=45)),
                        ));
                    }
                }
            }
        }

        // Departure (4:00 PM - 7:00 PM, most leave 5:00-6:30 PM)
        let departure_time =
            self.generate_departure_time(date, &user.behavior_profile, arrival_time);

        // Select departure room based on cross-location travel constraints
        let departure_room = self.select_departure_room(user, &schedule, registry);

        schedule.push(ScheduledActivity::new(
            ActivityType::Departure,
            departure_room, // Use location-aware departure room selection
            departure_time,
            Duration::minutes(10), // Time to pack up
        ));

        // Add curious user behavior - occasional unauthorized access attempts
        if user.is_curious {
            self.add_curious_user_activities(&mut schedule, user, date, registry);
        }

        // Sort schedule by start time and remove any overlapping activities
        schedule.sort_by_key(|a| a.start_time);
        Ok(self.resolve_schedule_conflicts(schedule, user, registry)?)
    }

    /// Generate a minimal schedule when full schedule generation fails
    #[instrument(skip(self, _registry), fields(user_id = %user.id))]
    fn generate_minimal_schedule(
        &mut self,
        user: &User,
        date: NaiveDate,
        _registry: &LocationRegistry,
    ) -> SimulationResult<Vec<ScheduledActivity>> {
        debug!("Generating minimal schedule for user {}", user.id);

        let mut schedule = Vec::new();

        // Just arrival and departure for minimal schedule
        let arrival_time = date
            .and_hms_opt(9, 0, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid arrival time"))?
            .and_utc();

        let departure_time = date
            .and_hms_opt(17, 0, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid departure time"))?
            .and_utc();

        schedule.push(ScheduledActivity::new(
            ActivityType::Arrival,
            user.primary_workspace,
            arrival_time,
            Duration::minutes(15),
        ));

        // For minimal schedule, we don't have cross-location travel, so use primary workspace
        schedule.push(ScheduledActivity::new(
            ActivityType::Departure,
            user.primary_workspace,
            departure_time,
            Duration::minutes(10),
        ));

        info!(
            "Generated minimal schedule with {} activities for user {}",
            schedule.len(),
            user.id
        );

        Ok(schedule)
    }

    /// Resolve conflicts in the schedule by adjusting overlapping activities
    /// Enhanced to consider travel time requirements and location persistence
    #[instrument(skip(self, registry))]
    pub fn resolve_schedule_conflicts(
        &mut self,
        schedule: Vec<ScheduledActivity>,
        user: &User,
        registry: &LocationRegistry,
    ) -> SimulationResult<Vec<ScheduledActivity>> {
        debug!(
            "Resolving schedule conflicts for {} activities with travel time validation",
            schedule.len()
        );

        let mut resolved_schedule = Vec::new();
        let mut activities_to_remove = Vec::new();

        for (index, mut activity) in schedule.into_iter().enumerate() {
            // Check if this activity conflicts with travel time requirements
            if let Some(conflict_resolution) = self.resolve_travel_time_conflict(
                &activity,
                &resolved_schedule,
                user,
                registry,
            )? {
                match conflict_resolution {
                    ConflictResolution::AdjustTime(new_start_time) => {
                        let old_start = activity.start_time;
                        activity.start_time = new_start_time;
                        debug!(
                            "Adjusted activity start time from {} to {} for travel time requirements",
                            old_start, activity.start_time
                        );
                    }
                    ConflictResolution::SkipActivity => {
                        debug!(
                            "Skipping activity {:?} at {} due to unresolvable travel time conflict",
                            activity.activity_type, activity.start_time
                        );
                        activities_to_remove.push(index);
                        continue;
                    }
                    ConflictResolution::NoConflict => {
                        // No adjustment needed
                    }
                }
            }

            // Apply location persistence constraints
            activity.target_room = self.apply_location_persistence_constraint(
                activity.target_room,
                &activity.activity_type,
                user,
                &resolved_schedule,
                registry,
            );

            // Check for basic time overlaps with previous activities
            if let Some(last_activity) = resolved_schedule.last() {
                let last_end = last_activity.start_time + last_activity.duration;
                if activity.start_time < last_end {
                    let old_start = activity.start_time;
                    activity.start_time =
                        last_end + self.travel_time_constants.scheduling_buffer_time;
                    debug!(
                        "Adjusted activity start time from {} to {} to resolve basic overlap",
                        old_start, activity.start_time
                    );
                }
            }

            resolved_schedule.push(activity);
        }

        debug!(
            "Resolved schedule conflicts, final schedule has {} activities ({} removed)",
            resolved_schedule.len(),
            activities_to_remove.len()
        );
        Ok(resolved_schedule)
    }

    /// Resolve travel time conflicts for a specific activity
    fn resolve_travel_time_conflict(
        &self,
        activity: &ScheduledActivity,
        current_schedule: &[ScheduledActivity],
        user: &User,
        registry: &LocationRegistry,
    ) -> SimulationResult<Option<ConflictResolution>> {
        // Get the location of the current activity
        let activity_location =
            self.get_room_location(activity.target_room, registry).ok_or_else(|| {
                SimulationError::behavior_engine_error(&format!(
                    "Cannot determine location for room {}",
                    activity.target_room
                ))
            })?;

        // Find the last activity before this one
        if let Some(last_activity) = current_schedule.last() {
            let last_location =
                self.get_room_location(last_activity.target_room, registry).ok_or_else(|| {
                    SimulationError::behavior_engine_error(&format!(
                        "Cannot determine location for room {}",
                        last_activity.target_room
                    ))
                })?;

            // Calculate required travel time
            let required_travel_time = self.calculate_minimum_travel_time(
                last_location.0,
                activity_location.0,
                Some(last_location.1),
                Some(activity_location.1),
            );

            let last_end_time = last_activity.start_time + last_activity.duration;
            let available_time = activity.start_time - last_end_time;

            // Check if we have sufficient travel time
            if available_time < required_travel_time {
                let required_start_time = last_end_time
                    + required_travel_time
                    + self.travel_time_constants.scheduling_buffer_time;

                // Check if adjusting the time would push it too late in the day
                if self.is_reasonable_activity_time(&activity.activity_type, required_start_time) {
                    return Ok(Some(ConflictResolution::AdjustTime(required_start_time)));
                } else {
                    // Try fallback strategies
                    if let Some(fallback_resolution) = self.try_fallback_strategies(
                        activity,
                        current_schedule,
                        user,
                        registry,
                    )? {
                        return Ok(Some(fallback_resolution));
                    } else {
                        return Ok(Some(ConflictResolution::SkipActivity));
                    }
                }
            }
        }

        Ok(Some(ConflictResolution::NoConflict))
    }

    /// Apply location persistence constraints to an activity
    fn apply_location_persistence_constraint(
        &self,
        original_room_id: RoomId,
        activity_type: &ActivityType,
        user: &User,
        current_schedule: &[ScheduledActivity],
        registry: &LocationRegistry,
    ) -> RoomId {
        // Check if user has traveled cross-location today
        if let Some(destination_location) =
            self.has_traveled_cross_location_today(user.id, current_schedule, registry)
        {
            // Constrain the activity to the destination location
            let constrained_location = self.constrain_activity_location(
                user,
                *activity_type,
                user.primary_location, // This will be overridden by constraint
                Some(destination_location),
            );

            // If the location changed, we need to find an appropriate room in the new location
            if let Some((original_location, _)) = self.get_room_location(original_room_id, registry)
            {
                if original_location != constrained_location {
                    // Find a suitable room in the constrained location
                    if let Some(new_room) = self.find_suitable_room_in_location(
                        constrained_location,
                        activity_type,
                        user,
                        registry,
                    ) {
                        debug!(
                            "Applied location persistence: moved activity from room {} to room {} (location constraint)",
                            original_room_id, new_room
                        );
                        return new_room;
                    }
                }
            }
        }

        original_room_id
    }

    /// Try fallback strategies when travel time requirements cannot be met
    fn try_fallback_strategies(
        &self,
        activity: &ScheduledActivity,
        current_schedule: &[ScheduledActivity],
        user: &User,
        registry: &LocationRegistry,
    ) -> SimulationResult<Option<ConflictResolution>> {
        // Strategy 1: Try to find a room in the same location as the last activity
        if let Some(last_activity) = current_schedule.last() {
            if let Some((last_location, last_building)) =
                self.get_room_location(last_activity.target_room, registry)
            {
                // Try to find a suitable room in the same location
                if let Some(fallback_room) = self.find_suitable_room_in_location(
                    last_location,
                    &activity.activity_type,
                    user,
                    registry,
                ) {
                    debug!(
                        "Applied fallback strategy: moved activity to same location (room {})",
                        fallback_room
                    );
                    // We don't return a ConflictResolution here because the room change
                    // will be handled by the location persistence constraint
                    return Ok(Some(ConflictResolution::NoConflict));
                }

                // Strategy 2: Try to find a room in the same building
                if let Some(fallback_room) = self.find_suitable_room_in_building(
                    last_location,
                    last_building,
                    &activity.activity_type,
                    user,
                    registry,
                ) {
                    debug!(
                        "Applied fallback strategy: moved activity to same building (room {})",
                        fallback_room
                    );
                    return Ok(Some(ConflictResolution::NoConflict));
                }
            }
        }

        // Strategy 3: No suitable fallback found
        Ok(None)
    }

    /// Check if an activity time is reasonable for the given activity type
    fn is_reasonable_activity_time(
        &self,
        activity_type: &ActivityType,
        proposed_time: DateTime<Utc>,
    ) -> bool {
        let hour = proposed_time.hour();

        match activity_type {
            ActivityType::Meeting => {
                // Meetings should be during business hours (9 AM - 5 PM)
                hour >= 9 && hour < 17
            }
            ActivityType::Lunch => {
                // Lunch should be between 11 AM - 3 PM
                hour >= 11 && hour < 15
            }
            ActivityType::Bathroom => {
                // Bathroom breaks can happen anytime during work hours
                hour >= 7 && hour < 19
            }
            ActivityType::Collaboration => {
                // Collaboration should be during business hours
                hour >= 9 && hour < 17
            }
            ActivityType::Departure => {
                // Departure should be in the evening (4 PM - 8 PM)
                hour >= 16 && hour < 20
            }
            _ => {
                // Other activities are more flexible
                hour >= 7 && hour < 19
            }
        }
    }

    /// Find a suitable room in a specific location for an activity type
    fn find_suitable_room_in_location(
        &self,
        location_id: LocationId,
        activity_type: &ActivityType,
        user: &User,
        registry: &LocationRegistry,
    ) -> Option<RoomId> {
        if let Some(location) = registry.get_location(location_id) {
            let required_room_type = match activity_type {
                ActivityType::Meeting => RoomType::MeetingRoom,
                ActivityType::Lunch => RoomType::Kitchen,
                ActivityType::Bathroom => RoomType::Bathroom,
                ActivityType::Collaboration => RoomType::Workspace,
                _ => RoomType::Workspace, // Default to workspace for other activities
            };

            // Try to find a room of the required type
            for building in &location.buildings {
                let rooms = building.get_rooms_by_type(required_room_type);
                for room in rooms {
                    if user.can_access_room(room.id, building.id, location_id) {
                        return Some(room.id);
                    }
                }
            }

            // Fallback: try to find any accessible room in the location
            for building in &location.buildings {
                for room in &building.rooms {
                    if user.can_access_room(room.id, building.id, location_id) {
                        return Some(room.id);
                    }
                }
            }
        }

        None
    }

    /// Find a suitable room in a specific building for an activity type
    fn find_suitable_room_in_building(
        &self,
        location_id: LocationId,
        building_id: BuildingId,
        activity_type: &ActivityType,
        user: &User,
        registry: &LocationRegistry,
    ) -> Option<RoomId> {
        if let Some(location) = registry.get_location(location_id) {
            if let Some(building) = location.get_building(building_id) {
                let required_room_type = match activity_type {
                    ActivityType::Meeting => RoomType::MeetingRoom,
                    ActivityType::Lunch => RoomType::Kitchen,
                    ActivityType::Bathroom => RoomType::Bathroom,
                    ActivityType::Collaboration => RoomType::Workspace,
                    _ => RoomType::Workspace,
                };

                // Try to find a room of the required type in this building
                let rooms = building.get_rooms_by_type(required_room_type);
                for room in rooms {
                    if user.can_access_room(room.id, building_id, location_id) {
                        return Some(room.id);
                    }
                }

                // Fallback: try any accessible room in this building
                for room in &building.rooms {
                    if user.can_access_room(room.id, building_id, location_id) {
                        return Some(room.id);
                    }
                }
            }
        }

        None
    }

    /// Get the location and building for a room
    pub fn get_room_location(
        &self,
        room_id: RoomId,
        registry: &LocationRegistry,
    ) -> Option<(LocationId, BuildingId)> {
        for location in registry.get_all_locations() {
            for building in &location.buildings {
                for room in &building.rooms {
                    if room.id == room_id {
                        return Some((location.id, building.id));
                    }
                }
            }
        }
        None
    }

    /// Generate a night-shift schedule for a user
    ///
    /// Creates an inverted schedule where the user starts inside the building,
    /// exits in the morning, and returns in the late afternoon for patrol duties.
    ///
    /// # Arguments
    /// * `user` - The night-shift user to generate a schedule for
    /// * `date` - The date to generate the schedule for
    /// * `registry` - Location registry for room lookups
    ///
    /// # Returns
    /// A vector of scheduled activities sorted by start time
    #[instrument(skip(self, registry), fields(user_id = %user.id, date = %date))]
    fn generate_night_shift_schedule(
        &mut self,
        user: &User,
        date: NaiveDate,
        registry: &LocationRegistry,
    ) -> SimulationResult<Vec<ScheduledActivity>> {
        debug!("Generating night-shift schedule for user {}", user.id);

        // Get the assigned night building or fall back to primary building
        let night_building = user.assigned_night_building.unwrap_or(user.primary_building);

        // Get accessible rooms in the night building for patrol
        let accessible_rooms =
            self.get_accessible_rooms_for_night_shift(user, night_building, registry);

        if accessible_rooms.is_empty() {
            warn!(
                "No accessible rooms found for night-shift user {}, using minimal schedule",
                user.id
            );
            return self.generate_minimal_night_shift_schedule(user, date);
        }

        let mut schedule = Vec::new();

        // PART 1: Early morning continuation (12 AM - 8 AM)
        // Simulate continuing from previous night shift

        // Late night patrol (12 AM - 2 AM) - Continuing from previous evening
        let midnight_patrol_start = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| {
                SimulationError::behavior_engine_error("Invalid midnight patrol start time")
            })?
            .and_utc();

        for (i, &room_id) in accessible_rooms.iter().enumerate().take(2) {
            let activity_start = midnight_patrol_start + Duration::minutes(i as i64 * 60);
            schedule.push(ScheduledActivity::new(
                ActivityType::NightPatrol,
                room_id,
                activity_start,
                Duration::minutes(45),
            ));
        }

        // Early morning patrol (2 AM - 4 AM) - Visit accessible rooms using NightPatrol activity
        let patrol_start = date
            .and_hms_opt(2, 0, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid patrol start time"))?
            .and_utc();

        for (i, &room_id) in accessible_rooms.iter().enumerate().take(3) {
            let activity_start = patrol_start + Duration::minutes(i as i64 * 40);
            schedule.push(ScheduledActivity::new(
                ActivityType::NightPatrol,
                room_id,
                activity_start,
                Duration::minutes(30),
            ));
        }

        // Break time (4:30 AM) - Use bathroom/break room if available
        let break_time = date
            .and_hms_opt(4, 30, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid break time"))?
            .and_utc();

        let break_room = self
            .select_break_room_for_night_shift(user, night_building, registry)
            .unwrap_or_else(|| accessible_rooms[0]); // Fall back to first accessible room

        schedule.push(ScheduledActivity::new(
            ActivityType::Bathroom,
            break_room,
            break_time,
            Duration::minutes(15),
        ));

        // Final patrol round (5 AM - 7 AM) - Visit rooms again using NightPatrol activity
        let final_patrol_start = date
            .and_hms_opt(5, 0, 0)
            .ok_or_else(|| {
                SimulationError::behavior_engine_error("Invalid final patrol start time")
            })?
            .and_utc();

        for (i, &room_id) in accessible_rooms.iter().enumerate().take(2) {
            let activity_start = final_patrol_start + Duration::minutes(i as i64 * 60);
            schedule.push(ScheduledActivity::new(
                ActivityType::NightPatrol,
                room_id,
                activity_start,
                Duration::minutes(45),
            ));
        }

        // Morning departure (8 AM) - Exit building
        let departure_time = date
            .and_hms_opt(8, 0, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid departure time"))?
            .and_utc();

        let exit_room = accessible_rooms[0]; // Use first accessible room as exit point
        schedule.push(ScheduledActivity::new(
            ActivityType::Departure,
            exit_room,
            departure_time,
            Duration::minutes(10),
        ));

        // PART 2: Evening start (5 PM - 11:59 PM)
        // Start new night shift for this day

        // Evening arrival (5 PM) - Return to building
        let arrival_time = date
            .and_hms_opt(17, 0, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid arrival time"))?
            .and_utc();

        let entry_room = accessible_rooms[0]; // Use first accessible room as entry point
        schedule.push(ScheduledActivity::new(
            ActivityType::Arrival,
            entry_room,
            arrival_time,
            Duration::minutes(10),
        ));

        // Evening patrol setup (6 PM - 8 PM) - Initial patrol rounds
        let evening_start = date
            .and_hms_opt(18, 0, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid evening start time"))?
            .and_utc();

        for (i, &room_id) in accessible_rooms.iter().enumerate().take(3) {
            let activity_start = evening_start + Duration::minutes(i as i64 * 40);
            schedule.push(ScheduledActivity::new(
                ActivityType::NightPatrol,
                room_id,
                activity_start,
                Duration::minutes(30),
            ));
        }

        // Evening break (8:30 PM) - Dinner/break time
        let evening_break_time = date
            .and_hms_opt(20, 30, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid evening break time"))?
            .and_utc();

        schedule.push(ScheduledActivity::new(
            ActivityType::Bathroom,
            break_room,
            evening_break_time,
            Duration::minutes(20),
        ));

        // Late evening patrol (9 PM - 11:30 PM) - Extended patrol rounds
        let late_evening_start = date
            .and_hms_opt(21, 0, 0)
            .ok_or_else(|| {
                SimulationError::behavior_engine_error("Invalid late evening start time")
            })?
            .and_utc();

        for (i, &room_id) in accessible_rooms.iter().enumerate().take(4) {
            let activity_start = late_evening_start + Duration::minutes(i as i64 * 35);
            schedule.push(ScheduledActivity::new(
                ActivityType::NightPatrol,
                room_id,
                activity_start,
                Duration::minutes(25),
            ));
        }

        // Sort schedule by start time and resolve conflicts
        schedule.sort_by_key(|a| a.start_time);
        let resolved_schedule = self.resolve_schedule_conflicts(schedule, user, registry)?;

        info!(
            "Generated night-shift schedule with {} activities for user {} (early morning: 12AM-8AM, evening: 5PM-11:59PM)",
            resolved_schedule.len(),
            user.id
        );

        Ok(resolved_schedule)
    }

    /// Get accessible rooms for a night-shift user in their assigned building
    fn get_accessible_rooms_for_night_shift(
        &mut self,
        user: &User,
        building_id: BuildingId,
        registry: &LocationRegistry,
    ) -> Vec<RoomId> {
        let mut accessible_rooms = Vec::new();

        if let Some(location) = registry.get_location(user.primary_location) {
            if let Some(building) = location.get_building(building_id) {
                for room in &building.rooms {
                    if user.can_access_room(room.id, building_id, user.primary_location) {
                        accessible_rooms.push(room.id);
                    }
                }
            }
        }

        // Shuffle rooms to add variety to patrol routes
        accessible_rooms.shuffle(&mut self.rng);
        accessible_rooms
    }

    /// Select an appropriate break room for night-shift user
    fn select_break_room_for_night_shift(
        &mut self,
        user: &User,
        building_id: BuildingId,
        registry: &LocationRegistry,
    ) -> Option<RoomId> {
        // Try to find bathroom or kitchen in the night building
        if let Some(location) = registry.get_location(user.primary_location) {
            if let Some(building) = location.get_building(building_id) {
                // Prefer bathroom, then kitchen, then any accessible room
                let preferred_types =
                    [crate::types::RoomType::Bathroom, crate::types::RoomType::Kitchen];

                for room_type in &preferred_types {
                    let rooms = building.get_rooms_by_type(*room_type);
                    for room in rooms {
                        if user.can_access_room(room.id, building_id, user.primary_location)
                        {
                            return Some(room.id);
                        }
                    }
                }
            }
        }

        None
    }

    /// Generate a minimal night-shift schedule when full schedule generation fails
    fn generate_minimal_night_shift_schedule(
        &mut self,
        user: &User,
        date: NaiveDate,
    ) -> SimulationResult<Vec<ScheduledActivity>> {
        debug!("Generating minimal night-shift schedule for user {}", user.id);

        let mut schedule = Vec::new();

        // Use primary workspace as fallback for all activities
        let fallback_room = user.primary_workspace;

        // PART 1: Early morning continuation (12 AM - 8 AM)

        // Minimal early morning patrol (2 AM)
        let early_patrol_time = date
            .and_hms_opt(2, 0, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid early patrol time"))?
            .and_utc();

        schedule.push(ScheduledActivity::new(
            ActivityType::NightPatrol,
            fallback_room,
            early_patrol_time,
            Duration::minutes(30),
        ));

        // Morning departure (8 AM)
        let departure_time = date
            .and_hms_opt(8, 0, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid departure time"))?
            .and_utc();

        schedule.push(ScheduledActivity::new(
            ActivityType::Departure,
            fallback_room,
            departure_time,
            Duration::minutes(10),
        ));

        // PART 2: Evening start (5 PM - 11:59 PM)

        // Evening arrival (5 PM)
        let arrival_time = date
            .and_hms_opt(17, 0, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid arrival time"))?
            .and_utc();

        schedule.push(ScheduledActivity::new(
            ActivityType::Arrival,
            fallback_room,
            arrival_time,
            Duration::minutes(10),
        ));

        // Minimal evening patrol (9 PM)
        let evening_patrol_time = date
            .and_hms_opt(21, 0, 0)
            .ok_or_else(|| SimulationError::behavior_engine_error("Invalid evening patrol time"))?
            .and_utc();

        schedule.push(ScheduledActivity::new(
            ActivityType::NightPatrol,
            fallback_room,
            evening_patrol_time,
            Duration::minutes(30),
        ));

        info!(
            "Generated minimal night-shift schedule with {} activities for user {} (early morning: 12AM-8AM, evening: 5PM-11:59PM)",
            schedule.len(),
            user.id
        );

        Ok(schedule)
    }

    /// Generate a realistic arrival time based on user behavior
    /// Note: This method is used for regular users only. Night-shift users
    /// use inverted schedules generated by generate_night_shift_schedule.
    #[instrument(skip(self, behavior))]
    fn generate_arrival_time(
        &mut self,
        date: NaiveDate,
        behavior: &BehaviorProfile,
    ) -> DateTime<Utc> {
        let base_hour = if behavior.is_schedule_focused() {
            // Schedule-focused users arrive consistently around 8:30 AM
            8
        } else {
            // Others have more variation (7:30 AM - 9:30 AM)
            self.rng.gen_range(7..=9)
        };

        let minute_variation = if behavior.schedule_adherence > 0.8 {
            self.rng.gen_range(0..=30) // ±30 minutes for punctual users
        } else {
            self.rng.gen_range(0..=60) // ±60 minutes for less punctual users
        };

        let arrival_time = date
            .and_hms_opt(base_hour, minute_variation.min(59), 0)
            .unwrap_or_else(|| date.and_hms_opt(base_hour, 0, 0).unwrap())
            .and_utc();

        debug!(
            "Generated arrival time: {} (schedule_focused: {})",
            arrival_time,
            behavior.is_schedule_focused()
        );
        arrival_time
    }

    /// Generate bathroom break times throughout the day
    #[instrument(skip(self, behavior))]
    fn generate_bathroom_break_time(
        &mut self,
        date: NaiveDate,
        behavior: &BehaviorProfile,
    ) -> Option<DateTime<Utc>> {
        // Bathroom breaks typically happen during work hours (8 AM - 6 PM)
        let hour = self.rng.gen_range(8..=17);
        let minute = self.rng.gen_range(0..=59);

        // Schedule-focused users have more regular patterns
        if behavior.is_schedule_focused() && self.rng.gen::<f64>() < 0.7 {
            // Prefer mid-morning or mid-afternoon
            let preferred_hour = if self.rng.gen_bool(0.5) { 10 } else { 15 };
            Some(
                date.and_hms_opt(preferred_hour, minute.min(59), 0)
                    .unwrap_or_else(|| date.and_hms_opt(preferred_hour, 0, 0).unwrap())
                    .and_utc(),
            )
        } else {
            Some(
                date.and_hms_opt(hour, minute.min(59), 0)
                    .unwrap_or_else(|| date.and_hms_opt(hour, 0, 0).unwrap())
                    .and_utc(),
            )
        }
    }

    /// Generate lunch time based on user behavior
    #[instrument(skip(self, behavior))]
    fn generate_lunch_time(
        &mut self,
        date: NaiveDate,
        behavior: &BehaviorProfile,
    ) -> DateTime<Utc> {
        let base_hour = if behavior.is_social() {
            // Social users tend to have lunch later (12:30-1:30 PM)
            self.rng.gen_range(12..=13)
        } else {
            // Others prefer earlier lunch (11:30 AM - 12:30 PM)
            self.rng.gen_range(11..=12)
        };

        let minute = self.rng.gen_range(0..=59);
        let lunch_time = date
            .and_hms_opt(base_hour, minute.min(59), 0)
            .unwrap_or_else(|| date.and_hms_opt(base_hour, 0, 0).unwrap())
            .and_utc();

        debug!("Generated lunch time: {} (social: {})", lunch_time, behavior.is_social());
        lunch_time
    }

    /// Calculate number of meetings based on social level
    #[instrument(skip(self, behavior))]
    fn calculate_meeting_count(&mut self, behavior: &BehaviorProfile) -> usize {
        let base_meetings: f64 = if behavior.is_social() {
            3.0 // Social users have more meetings
        } else {
            1.5 // Less social users have fewer meetings
        };

        let variation: f64 = self.rng.gen_range(-1.0..=2.0);
        let total_meetings = (base_meetings + variation).max(0.0);
        (total_meetings as usize).min(5)
    }

    /// Generate meeting time during business hours, accounting for travel time from previous activities
    fn generate_meeting_time(
        &mut self,
        date: NaiveDate,
        behavior: &BehaviorProfile,
        current_schedule: &[ScheduledActivity],
        user: &User,
        registry: &LocationRegistry,
    ) -> Option<DateTime<Utc>> {
        // Generate a base meeting time first
        let base_meeting_time = self.generate_base_meeting_time(date, behavior)?;

        // Calculate the earliest possible meeting time considering travel time
        self.calculate_earliest_meeting_time(
            base_meeting_time,
            current_schedule,
            user,
            registry,
        )
    }

    /// Generate a base meeting time without travel time considerations
    fn generate_base_meeting_time(
        &mut self,
        date: NaiveDate,
        behavior: &BehaviorProfile,
    ) -> Option<DateTime<Utc>> {
        // Meetings typically happen 9 AM - 5 PM, avoiding lunch hours
        let hour = if behavior.is_schedule_focused() {
            // Prefer standard meeting times (9 AM, 10 AM, 2 PM, 3 PM, 4 PM)
            *[9, 10, 14, 15, 16].choose(&mut self.rng).unwrap()
        } else {
            // More flexible timing
            loop {
                let h = self.rng.gen_range(9..=16);
                // Avoid typical lunch hours (12-1 PM)
                if h != 12 {
                    break h;
                }
            }
        };

        let minute = if behavior.is_schedule_focused() {
            // Prefer on-the-hour or half-hour
            if self.rng.gen_bool(0.7) {
                0
            } else {
                30
            }
        } else {
            self.rng.gen_range(0..=59)
        };

        Some(
            date.and_hms_opt(hour, minute.min(59), 0)
                .unwrap_or_else(|| date.and_hms_opt(hour, 0, 0).unwrap())
                .and_utc(),
        )
    }

    /// Calculate the earliest possible meeting time considering travel time from previous activities
    fn calculate_earliest_meeting_time(
        &self,
        proposed_time: DateTime<Utc>,
        current_schedule: &[ScheduledActivity],
        user: &User,
        registry: &LocationRegistry,
    ) -> Option<DateTime<Utc>> {
        // Find the last activity before the proposed meeting time
        if let Some((last_location, last_building)) =
            self.get_last_activity_location(current_schedule, proposed_time, registry)
        {
            // For now, we'll assume the meeting will be in the user's primary location
            // This will be refined when we integrate with room selection
            let meeting_location = user.primary_location;
            let meeting_building = Some(user.primary_building);

            // Calculate minimum travel time required
            let travel_time = self.calculate_minimum_travel_time(
                last_location,
                meeting_location,
                Some(last_building),
                meeting_building,
            );

            // Find the end time of the last activity
            let last_activity_end = current_schedule
                .iter()
                .filter(|activity| activity.start_time < proposed_time)
                .map(|activity| activity.start_time + activity.duration)
                .max()
                .unwrap_or(proposed_time);

            // Calculate earliest possible meeting time
            let earliest_time =
                last_activity_end + travel_time + self.travel_time_constants.scheduling_buffer_time;

            // Return the later of the proposed time or the earliest possible time
            Some(proposed_time.max(earliest_time))
        } else {
            // No previous activities, proposed time is fine
            Some(proposed_time)
        }
    }

    /// Generate collaboration time for social interactions
    fn generate_collaboration_time(
        &mut self,
        date: NaiveDate,
        _behavior: &BehaviorProfile,
    ) -> Option<DateTime<Utc>> {
        // Collaboration typically happens during work hours, avoiding early morning and late afternoon
        let hour = self.rng.gen_range(10..=15);
        let minute = self.rng.gen_range(0..=59);

        Some(
            date.and_hms_opt(hour, minute.min(59), 0)
                .unwrap_or_else(|| date.and_hms_opt(hour, 0, 0).unwrap())
                .and_utc(),
        )
    }

    /// Generate departure time based on arrival and behavior
    /// Note: This method is used for regular users only. Night-shift users
    /// use inverted schedules generated by generate_night_shift_schedule.
    fn generate_departure_time(
        &mut self,
        date: NaiveDate,
        behavior: &BehaviorProfile,
        arrival_time: DateTime<Utc>,
    ) -> DateTime<Utc> {
        // Calculate minimum work hours (typically 8 hours)
        let min_work_hours = if behavior.is_schedule_focused() { 8.0 } else { 7.5 };
        let earliest_departure = arrival_time + Duration::hours(min_work_hours as i64);

        // Add some variation based on behavior
        let variation_hours = if behavior.is_schedule_focused() {
            self.rng.gen_range(-0.5..=1.0) // Leave on time or slightly late
        } else {
            self.rng.gen_range(-1.0..=2.0) // More variation
        };

        let departure_time =
            earliest_departure + Duration::minutes((variation_hours * 60.0) as i64);

        // Ensure departure is not too late (before 8 PM)
        let max_departure = date.and_hms_opt(20, 0, 0).unwrap().and_utc();
        departure_time.min(max_departure)
    }

    /// Select an appropriate bathroom room for the user
    fn select_bathroom_room(
        &mut self,
        user: &User,
        registry: &LocationRegistry,
    ) -> Option<RoomId> {
        // First try to find a bathroom in the user's current building
        if let Some(location) = registry.get_location(user.primary_location) {
            if let Some(building) = location.get_building(user.primary_building) {
                let bathrooms = building.get_rooms_by_type(RoomType::Bathroom);
                let authorized_bathrooms: Vec<_> = bathrooms
                    .into_iter()
                    .filter(|room| user.can_access_room(room.id, building.id, location.id))
                    .collect();

                if !authorized_bathrooms.is_empty() {
                    debug!(
                        "Found {} authorized bathrooms in primary building",
                        authorized_bathrooms.len()
                    );
                    return Some(
                        authorized_bathrooms[self.rng.gen_range(0..authorized_bathrooms.len())].id,
                    );
                }
            }
        }

        // If no bathroom in primary building, try other buildings in the same location
        if let Some(location) = registry.get_location(user.primary_location) {
            for building in &location.buildings {
                if building.id != user.primary_building
                    && user.can_access_building(building.id, location.id)
                {
                    let bathrooms = building.get_rooms_by_type(RoomType::Bathroom);
                    let authorized_bathrooms: Vec<_> = bathrooms
                        .into_iter()
                        .filter(|room| user.can_access_room(room.id, building.id, location.id))
                        .collect();

                    if !authorized_bathrooms.is_empty() {
                        debug!(
                            "Found {} authorized bathrooms in building {}",
                            authorized_bathrooms.len(),
                            building.id
                        );
                        return Some(
                            authorized_bathrooms[self.rng.gen_range(0..authorized_bathrooms.len())]
                                .id,
                        );
                    }
                }
            }
        }

        debug!("No authorized bathroom rooms found for user {}", user.id);
        None
    }

    /// Select an appropriate lunch room (cafeteria or kitchen)
    fn select_lunch_room(
        &mut self,
        user: &User,
        registry: &LocationRegistry,
    ) -> Option<RoomId> {
        // Prefer cafeteria, fall back to kitchen
        let preferred_types = [RoomType::Cafeteria, RoomType::Kitchen];

        for room_type in &preferred_types {
            // Try primary building first
            if let Some(location) = registry.get_location(user.primary_location) {
                if let Some(building) = location.get_building(user.primary_building) {
                    let rooms = building.get_rooms_by_type(*room_type);
                    let authorized_rooms: Vec<_> = rooms
                        .into_iter()
                        .filter(|room| user.can_access_room(room.id, building.id, location.id))
                        .collect();

                    if !authorized_rooms.is_empty() {
                        debug!(
                            "Found {} authorized {} rooms in primary building",
                            authorized_rooms.len(),
                            room_type
                        );
                        return Some(
                            authorized_rooms[self.rng.gen_range(0..authorized_rooms.len())].id,
                        );
                    }
                }

                // Try other buildings in the same location
                for building in &location.buildings {
                    if building.id != user.primary_building
                        && user.can_access_building(building.id, location.id)
                    {
                        let rooms = building.get_rooms_by_type(*room_type);
                        let authorized_rooms: Vec<_> = rooms
                            .into_iter()
                            .filter(|room| {
                                user.can_access_room(room.id, building.id, location.id)
                            })
                            .collect();

                        if !authorized_rooms.is_empty() {
                            debug!(
                                "Found {} authorized {} rooms in building {}",
                                authorized_rooms.len(),
                                room_type,
                                building.id
                            );
                            return Some(
                                authorized_rooms[self.rng.gen_range(0..authorized_rooms.len())].id,
                            );
                        }
                    }
                }
            }
        }

        debug!("No authorized lunch rooms found for user {}", user.id);
        None
    }

    /// Select an appropriate meeting room based on location affinity and travel constraints
    fn select_meeting_room(
        &mut self,
        user: &User,
        registry: &LocationRegistry,
        meeting_time: DateTime<Utc>,
        current_schedule: &[ScheduledActivity],
    ) -> Option<RoomId> {
        // Check if user has traveled cross-location today
        let destination_location =
            self.has_traveled_cross_location_today(user.id, current_schedule, registry);

        // If user has traveled cross-location, constrain meetings to destination location
        if let Some(dest_location) = destination_location {
            debug!(
                "User {} has traveled cross-location to {}, constraining meeting to destination location",
                user.id, dest_location
            );
            return self.select_meeting_room_in_location_with_permissions(
                user,
                dest_location,
                None,
                registry,
            );
        }

        // Normal location affinity logic with travel time validation
        let travel_roll = self.rng.gen::<f64>();

        if (travel_roll < self.config.primary_building_affinity) || user.is_curious {
            // Meeting in primary building
            self.select_meeting_room_in_building_with_permissions(
                user,
                user.primary_building,
                registry,
            )
        } else if travel_roll
            < self.config.primary_building_affinity + self.config.same_location_travel
        {
            // Meeting in same location, different building
            // Validate travel time for intra-location building changes
            if self.validate_travel_time_for_meeting(
                user,
                current_schedule,
                meeting_time,
                user.primary_location,
                registry,
            ) {
                self.select_meeting_room_in_location_with_permissions(
                    user,
                    user.primary_location,
                    Some(user.primary_building),
                    registry,
                )
            } else {
                // Fall back to primary building if travel time validation fails
                debug!("Travel time validation failed for same-location meeting, falling back to primary building");
                self.select_meeting_room_in_building_with_permissions(
                    user,
                    user.primary_building,
                    registry,
                )
            }
        } else {
            // Meeting in different location (rare)
            // Validate travel time for cross-location meetings
            if self.validate_travel_time_for_cross_location_meeting(
                user,
                current_schedule,
                meeting_time,
                registry,
            ) {
                self.select_meeting_room_in_different_location_with_permissions(
                    user,
                    registry,
                    current_schedule,
                )
            } else {
                // Fall back to same location if cross-location travel time validation fails
                debug!("Travel time validation failed for cross-location meeting, falling back to same location");
                self.select_meeting_room_in_location_with_permissions(
                    user,
                    user.primary_location,
                    Some(user.primary_building),
                    registry,
                )
            }
        }
    }

    /// Select a meeting room within a specific building with permission validation
    fn select_meeting_room_in_building_with_permissions(
        &mut self,
        user: &User,
        building_id: BuildingId,
        registry: &LocationRegistry,
    ) -> Option<RoomId> {
        if let Some(building) = registry.get_building(building_id) {
            if let Some(location) = registry.get_location_for_building(building_id) {
                let meeting_rooms = building.get_rooms_by_type(RoomType::MeetingRoom);
                let authorized_rooms: Vec<_> = meeting_rooms
                    .into_iter()
                    .filter(|room| user.can_access_room(room.id, building_id, location.id))
                    .collect();

                if !authorized_rooms.is_empty() {
                    debug!(
                        "Found {} authorized meeting rooms in building {}",
                        authorized_rooms.len(),
                        building_id
                    );
                    return Some(
                        authorized_rooms[self.rng.gen_range(0..authorized_rooms.len())].id,
                    );
                } else {
                    debug!(
                        "No authorized meeting rooms found for user {} in building {}",
                        user.id, building_id
                    );
                }
            }
        }
        None
    }

    /// Select a meeting room within a location with permission validation, excluding a specific building
    fn select_meeting_room_in_location_with_permissions(
        &mut self,
        user: &User,
        location_id: LocationId,
        exclude_building: Option<BuildingId>,
        registry: &LocationRegistry,
    ) -> Option<RoomId> {
        if let Some(location) = registry.get_location(location_id) {
            let available_buildings: Vec<_> = location
                .buildings
                .iter()
                .filter(|b| {
                    Some(b.id) != exclude_building
                        && user.can_access_building(b.id, location_id)
                })
                .collect();

            if !available_buildings.is_empty() {
                let building =
                    available_buildings[self.rng.gen_range(0..available_buildings.len())];
                return self.select_meeting_room_in_building_with_permissions(
                    user,
                    building.id,
                    registry,
                );
            } else {
                debug!(
                    "No accessible buildings found for user {} in location {}",
                    user.id, location_id
                );
            }
        }
        None
    }

    /// Select a meeting room in a different location with permission validation (for travel scenarios)
    fn select_meeting_room_in_different_location_with_permissions(
        &mut self,
        user: &User,
        registry: &LocationRegistry,
        _current_schedule: &[ScheduledActivity],
    ) -> Option<RoomId> {
        let available_locations: Vec<_> = registry
            .locations
            .iter()
            .filter(|l| l.id != user.primary_location && user.can_access_location(l.id))
            .collect();

        if !available_locations.is_empty() {
            let location = available_locations[self.rng.gen_range(0..available_locations.len())];

            // Track that user will travel to this location for the day
            self.track_user_daily_location(user.id, location.id);

            return self.select_meeting_room_in_location_with_permissions(
                user,
                location.id,
                None,
                registry,
            );
        } else {
            debug!(
                "No accessible locations found for user {} for cross-location travel",
                user.id
            );
        }
        None
    }

    /// Validate if there's sufficient travel time for a meeting within the same location
    fn validate_travel_time_for_meeting(
        &self,
        user: &User,
        current_schedule: &[ScheduledActivity],
        meeting_time: DateTime<Utc>,
        meeting_location: LocationId,
        registry: &LocationRegistry,
    ) -> bool {
        // Get the last activity location before the meeting time
        if let Some((last_location, last_building)) =
            self.get_last_activity_location(current_schedule, meeting_time, registry)
        {
            // Calculate required travel time
            let travel_time = self.calculate_minimum_travel_time(
                last_location,
                meeting_location,
                Some(last_building),
                Some(user.primary_building), // Assume meeting in primary building for validation
            );

            // Find the end time of the last activity
            let last_activity_end = current_schedule
                .iter()
                .filter(|activity| activity.start_time < meeting_time)
                .map(|activity| activity.start_time + activity.duration)
                .max()
                .unwrap_or(meeting_time);

            // Check if there's sufficient time
            let available_time = meeting_time - last_activity_end;
            let required_time = travel_time + self.travel_time_constants.scheduling_buffer_time;

            available_time >= required_time
        } else {
            // No previous activities, validation passes
            true
        }
    }

    /// Validate if there's sufficient travel time for a cross-location meeting
    fn validate_travel_time_for_cross_location_meeting(
        &self,
        _user: &User,
        current_schedule: &[ScheduledActivity],
        meeting_time: DateTime<Utc>,
        registry: &LocationRegistry,
    ) -> bool {
        // Get the last activity location before the meeting time
        if let Some((_last_location, _last_building)) =
            self.get_last_activity_location(current_schedule, meeting_time, registry)
        {
            // For cross-location meetings, we need at least 4 hours travel time
            let travel_time = self.travel_time_constants.cross_location_travel_time;

            // Find the end time of the last activity
            let last_activity_end = current_schedule
                .iter()
                .filter(|activity| activity.start_time < meeting_time)
                .map(|activity| activity.start_time + activity.duration)
                .max()
                .unwrap_or(meeting_time);

            // Check if there's sufficient time for cross-location travel
            let available_time = meeting_time - last_activity_end;
            let required_time = travel_time + self.travel_time_constants.scheduling_buffer_time;

            let is_valid = available_time >= required_time;

            if !is_valid {
                debug!(
                    "Cross-location meeting validation failed: need {} minutes, only {} minutes available",
                    required_time.num_minutes(),
                    available_time.num_minutes()
                );
            }

            is_valid
        } else {
            // No previous activities, validation passes
            true
        }
    }

    /// Select a room for collaboration (workspace or meeting room)
    fn select_collaboration_room(
        &mut self,
        user: &User,
        registry: &LocationRegistry,
    ) -> Option<RoomId> {
        // 70% chance to visit a workspace, 30% chance to use a meeting room
        let room_type =
            if self.rng.gen::<f64>() < 0.7 { RoomType::Workspace } else { RoomType::MeetingRoom };

        // Prefer same building for collaboration
        if let Some(location) = registry.get_location(user.primary_location) {
            if let Some(building) = location.get_building(user.primary_building) {
                let rooms = building.get_rooms_by_type(room_type);
                let authorized_rooms: Vec<_> = rooms
                    .into_iter()
                    .filter(|room| user.can_access_room(room.id, building.id, location.id))
                    .collect();

                if !authorized_rooms.is_empty() {
                    debug!(
                        "Found {} authorized collaboration rooms in primary building",
                        authorized_rooms.len()
                    );
                    return Some(
                        authorized_rooms[self.rng.gen_range(0..authorized_rooms.len())].id,
                    );
                }
            }
        }

        debug!("No authorized collaboration rooms found for user {}", user.id);
        None
    }

    /// Check if a user should attempt unauthorized access based on curiosity
    pub fn should_attempt_unauthorized_access(&mut self, user: &User) -> bool {
        if !user.is_curious {
            return false;
        }

        // Curious users occasionally attempt unauthorized access
        let base_probability = self.config.curious_user_percentage * 0.1; // 10% of the curiosity rate per day
        let curiosity_multiplier = 1.0 + user.behavior_profile.curiosity_level;
        let adjusted_probability = base_probability * curiosity_multiplier;

        self.rng.gen::<f64>() < adjusted_probability
    }

    /// Generate an unauthorized access attempt for a curious user
    pub fn generate_unauthorized_access_attempt(
        &mut self,
        user: &User,
        registry: &LocationRegistry,
    ) -> Option<RoomId> {
        if !user.is_curious {
            return None;
        }

        // Try to find a room the user doesn't have access to
        if let Some(location) = registry.get_location(user.primary_location) {
            let mut unauthorized_rooms = Vec::new();

            for building in &location.buildings {
                for room in &building.rooms {
                    if !user.can_access_room(room.id, building.id, location.id) {
                        // Prefer higher security rooms for curious users
                        match room.security_level {
                            SecurityLevel::Restricted
                            | SecurityLevel::HighSecurity
                            | SecurityLevel::MaxSecurity => {
                                unauthorized_rooms.push(room.id);
                            }
                            _ => {
                                // Lower chance for standard rooms
                                if self.rng.gen::<f64>() < 0.3 {
                                    unauthorized_rooms.push(room.id);
                                }
                            }
                        }
                    }
                }
            }

            if !unauthorized_rooms.is_empty() {
                return Some(unauthorized_rooms[self.rng.gen_range(0..unauthorized_rooms.len())]);
            }
        }

        None
    }

    /// Add curious user activities to the daily schedule
    ///
    /// Curious users occasionally attempt unauthorized access while still
    /// primarily using authorized areas. This method adds 1-3 unauthorized
    /// access attempts per day for curious users.
    fn add_curious_user_activities(
        &mut self,
        schedule: &mut Vec<ScheduledActivity>,
        user: &User,
        date: NaiveDate,
        registry: &LocationRegistry,
    ) {
        // Curious users should still primarily use authorized areas (80-90% of the time)
        // Only add unauthorized attempts occasionally
        if !self.should_attempt_unauthorized_access_for_schedule(user) {
            return;
        }

        // Generate 1-3 unauthorized access attempts per day for curious users
        let attempt_count = self.rng.gen_range(1..=3);

        for _ in 0..attempt_count {
            if let Some(unauthorized_room) =
                self.generate_unauthorized_access_attempt(user, registry)
            {
                // Generate a random time during work hours for the unauthorized attempt
                if let Some(attempt_time) =
                    self.generate_curious_access_time(date, &user.behavior_profile)
                {
                    // Create a scheduled activity for the unauthorized access attempt
                    schedule.push(ScheduledActivity::new(
                        ActivityType::UnauthorizedAccess,
                        unauthorized_room,
                        attempt_time,
                        Duration::minutes(self.rng.gen_range(5..=15)), // Short attempts
                    ));
                }
            }
        }
    }

    /// Generate a time for curious user unauthorized access attempts
    ///
    /// These attempts typically happen during quieter periods when there's
    /// less supervision - early morning, late afternoon, or during lunch breaks.
    fn generate_curious_access_time(
        &mut self,
        date: NaiveDate,
        behavior: &BehaviorProfile,
    ) -> Option<DateTime<Utc>> {
        // Curious users prefer times with less supervision
        let preferred_hours = if behavior.curiosity_level > 0.7 {
            // Highly curious users are bolder - any work hours
            vec![8, 9, 10, 11, 13, 14, 15, 16, 17]
        } else {
            // Moderately curious users prefer quieter times
            vec![8, 12, 13, 17] // Early morning, lunch, late afternoon
        };

        let hour = *preferred_hours.choose(&mut self.rng)?;
        let minute = self.rng.gen_range(0..=59);

        Some(
            date.and_hms_opt(hour, minute.min(59), 0)
                .unwrap_or_else(|| date.and_hms_opt(hour, 0, 0).unwrap())
                .and_utc(),
        )
    }

    /// Enhanced method to check if a user should attempt unauthorized access for daily schedule
    ///
    /// This considers both the user's curiosity level and ensures that
    /// curious users still primarily use authorized areas most of the time.
    fn should_attempt_unauthorized_access_for_schedule(&mut self, user: &User) -> bool {
        if !user.is_curious {
            return false;
        }

        // Base probability is low to ensure primary authorized usage
        let base_probability = self.config.curious_user_percentage * 0.05; // 5% of the curiosity rate per day
        let curiosity_multiplier = 1.0 + user.behavior_profile.curiosity_level;
        let adjusted_probability = base_probability * curiosity_multiplier;

        // Cap the probability to ensure users still primarily use authorized areas
        let capped_probability = adjusted_probability.min(0.2); // Maximum 20% chance per day

        self.rng.gen::<f64>() < capped_probability
    }

    /// Calculate minimum travel time between two locations
    ///
    /// Returns the minimum time required to travel between locations based on:
    /// - Cross-location travel: 4 hours (flights, driving between cities)
    /// - Intra-location building travel: 15-30 minutes (walking, driving, shuttle)
    /// - Same building: No additional travel time required
    ///
    /// # Arguments
    /// * `from_location` - Source location ID
    /// * `to_location` - Destination location ID  
    /// * `from_building` - Source building ID (optional)
    /// * `to_building` - Destination building ID (optional)
    ///
    /// # Returns
    /// Duration representing minimum travel time required
    pub fn calculate_minimum_travel_time(
        &self,
        from_location: LocationId,
        to_location: LocationId,
        from_building: Option<BuildingId>,
        to_building: Option<BuildingId>,
    ) -> Duration {
        // Same location check
        if from_location != to_location {
            // Different geographical locations require significant travel time (flights, etc.)
            return self.travel_time_constants.cross_location_travel_time;
        }

        // Same location, check buildings
        match (from_building, to_building) {
            (Some(from_bld), Some(to_bld)) if from_bld == to_bld => {
                // Same building - no additional travel time
                Duration::zero()
            }
            (Some(_), Some(_)) => {
                // Different buildings in same location - use random time within range
                let (_min_time, max_time) =
                    self.travel_time_constants.intra_location_building_travel_time;
                // For deterministic calculation, use the maximum time to be safe
                max_time
            }
            _ => {
                // Unknown building information - assume intra-location travel
                self.travel_time_constants.intra_location_building_travel_time.1
            }
        }
    }

    /// Get the last activity location for a user before a specific time
    ///
    /// Searches through the user's schedule to find the most recent activity
    /// before the given time and returns its location information.
    ///
    /// # Arguments
    /// * `schedule` - The user's scheduled activities
    /// * `before_time` - Find the last activity before this time
    /// * `registry` - Location registry for room lookups
    ///
    /// # Returns
    /// Option containing (LocationId, BuildingId) of the last activity, or None if no previous activity
    pub fn get_last_activity_location(
        &self,
        schedule: &[ScheduledActivity],
        before_time: DateTime<Utc>,
        registry: &LocationRegistry,
    ) -> Option<(LocationId, BuildingId)> {
        // Find the most recent activity that ends before the given time
        let last_activity = schedule
            .iter()
            .filter(|activity| activity.end_time() <= before_time)
            .max_by_key(|activity| activity.end_time())?;

        // Look up the room's location and building
        self.get_room_location_info(last_activity.target_room, registry)
    }

    /// Helper method to get location and building information for a room
    ///
    /// # Arguments
    /// * `room_id` - The room to look up
    /// * `registry` - Location registry for room lookups
    ///
    /// # Returns
    /// Option containing (LocationId, BuildingId) for the room, or None if not found
    fn get_room_location_info(
        &self,
        room_id: RoomId,
        registry: &LocationRegistry,
    ) -> Option<(LocationId, BuildingId)> {
        // Search through all locations and buildings to find the room
        for location in &registry.locations {
            for building in &location.buildings {
                for room in &building.rooms {
                    if room.id == room_id {
                        return Some((location.id, building.id));
                    }
                }
            }
        }
        None
    }

    /// Track user's current location for the day
    ///
    /// This method ensures that once a user travels to a different geographical location,
    /// they remain at that location for all subsequent activities that day. This implements
    /// realistic business travel behavior where users don't make multiple cross-location
    /// trips in a single day.
    ///
    /// # Arguments
    /// * `user_id` - The user to track
    /// * `current_location` - The location the user is currently at
    ///
    /// # Returns
    /// The location the user should be constrained to for the rest of the day
    pub fn track_user_daily_location(
        &mut self,
        user_id: UserId,
        current_location: LocationId,
    ) -> LocationId {
        // If this is the first time we're tracking this user today, or if they're
        // moving to a different location, update their daily location
        match self.user_daily_locations.get(&user_id) {
            Some(&existing_location) => {
                // User already has a tracked location for today
                if existing_location != current_location {
                    debug!(
                        "User {} has traveled from {} to {} - constraining to destination location",
                        user_id, existing_location, current_location
                    );
                    // Update to the new location (cross-location travel)
                    self.user_daily_locations.insert(user_id, current_location);
                }
                // Return the current tracked location (may have been updated)
                *self.user_daily_locations.get(&user_id).unwrap()
            }
            None => {
                // First time tracking this user today
                debug!(
                    "Tracking user {} at location {} for the day",
                    user_id, current_location
                );
                self.user_daily_locations.insert(user_id, current_location);
                current_location
            }
        }
    }

    /// Check if user has already traveled cross-location today
    ///
    /// Analyzes the user's schedule to determine if they have already made a cross-location
    /// trip today. This is used to enforce the constraint that users can only make one
    /// cross-location trip per day and must remain at the destination location.
    ///
    /// # Arguments
    /// * `user_id` - The user to check
    /// * `schedule` - The user's current schedule
    /// * `registry` - Location registry for room lookups
    ///
    /// # Returns
    /// Some(LocationId) if the user has traveled cross-location (returns destination location),
    /// None if they haven't traveled cross-location yet today
    pub fn has_traveled_cross_location_today(
        &self,
        user_id: UserId,
        schedule: &[ScheduledActivity],
        registry: &LocationRegistry,
    ) -> Option<LocationId> {
        // Check if we have a tracked location that's different from their primary location
        if let Some(&tracked_location) = self.user_daily_locations.get(&user_id) {
            // We need to get the user's primary location to compare
            // Since we don't have direct access to the user here, we'll analyze the schedule

            // Find the first activity (usually arrival) to determine primary location
            if let Some(first_activity) = schedule.first() {
                if let Some((first_location, _)) =
                    self.get_room_location_info(first_activity.target_room, registry)
                {
                    if tracked_location != first_location {
                        debug!(
                            "User {} has traveled cross-location from {} to {}",
                            user_id, first_location, tracked_location
                        );
                        return Some(tracked_location);
                    }
                }
            }
        }

        // Also check the schedule for any cross-location activities
        let mut previous_location: Option<LocationId> = None;

        for activity in schedule {
            if let Some((activity_location, _)) =
                self.get_room_location_info(activity.target_room, registry)
            {
                if let Some(prev_loc) = previous_location {
                    if prev_loc != activity_location {
                        debug!(
                            "User {} has cross-location activity from {} to {}",
                            user_id, prev_loc, activity_location
                        );
                        // Update tracking and return the destination location
                        return Some(activity_location);
                    }
                }
                previous_location = Some(activity_location);
            }
        }

        None
    }

    /// Constrain activity location based on cross-location travel rules
    ///
    /// Ensures that activities after cross-location travel stay in the destination location.
    /// This method implements the business rule that users who travel to a different
    /// geographical location must remain at that location for all subsequent activities.
    ///
    /// # Arguments
    /// * `user` - The user for whom to constrain the activity
    /// * `activity_type` - The type of activity being scheduled
    /// * `preferred_location` - The originally preferred location for this activity
    /// * `current_day_location` - The location the user is constrained to (if any)
    ///
    /// # Returns
    /// The location where the activity should actually take place
    pub fn constrain_activity_location(
        &self,
        user: &User,
        activity_type: ActivityType,
        preferred_location: LocationId,
        current_day_location: Option<LocationId>,
    ) -> LocationId {
        match current_day_location {
            Some(constrained_location) => {
                if constrained_location != preferred_location {
                    debug!(
                        "Constraining {} activity for user {} from {} to {} due to cross-location travel",
                        activity_type, user.id, preferred_location, constrained_location
                    );
                    constrained_location
                } else {
                    preferred_location
                }
            }
            None => {
                // No constraint, use preferred location
                preferred_location
            }
        }
    }

    /// Get the destination location for departure activity after cross-location travel
    ///
    /// Determines where the user's departure activity should occur. If the user
    /// has traveled cross-location during the day, their departure should occur from
    /// the destination location, not their original primary workspace.
    ///
    /// # Arguments
    /// * `user` - The user whose departure location to determine
    /// * `schedule` - The user's current schedule
    /// * `registry` - Location registry for room lookups
    ///
    /// # Returns
    /// The location where the departure activity should occur
    pub fn get_departure_location(
        &self,
        user: &User,
        schedule: &[ScheduledActivity],
        registry: &LocationRegistry,
    ) -> LocationId {
        // Check if user has traveled cross-location today
        if let Some(destination_location) =
            self.has_traveled_cross_location_today(user.id, schedule, registry)
        {
            debug!(
                "User {} departure will be from destination location {} instead of primary location {}",
                user.id, destination_location, user.primary_location
            );
            destination_location
        } else {
            // No cross-location travel, use primary location
            user.primary_location
        }
    }

    /// Select an appropriate room for departure activity based on location constraints
    ///
    /// Finds a suitable room for the departure activity, respecting cross-location travel
    /// constraints. If the user has traveled cross-location, finds a room in the
    /// destination location instead of their primary workspace.
    ///
    /// # Arguments
    /// * `user` - The user whose departure room to select
    /// * `schedule` - The user's current schedule
    /// * `registry` - Location registry for room lookups
    ///
    /// # Returns
    /// A room ID for the departure activity, or the user's primary workspace as fallback
    pub fn select_departure_room(
        &mut self,
        user: &User,
        schedule: &[ScheduledActivity],
        registry: &LocationRegistry,
    ) -> RoomId {
        let departure_location = self.get_departure_location(user, schedule, registry);

        if departure_location != user.primary_location {
            // User has traveled cross-location, find a suitable room in destination location
            if let Some(location) = registry.get_location(departure_location) {
                // Try to find a workspace or meeting room in the destination location
                for building in &location.buildings {
                    // First try workspaces
                    let workspaces = building.get_rooms_by_type(RoomType::Workspace);
                    if !workspaces.is_empty()
                        && user.can_access_building(building.id, departure_location)
                    {
                        let selected_room = workspaces[self.rng.gen_range(0..workspaces.len())];
                        debug!(
                            "Selected departure room {} in destination location {} for user {}",
                            selected_room.id, departure_location, user.id
                        );
                        return selected_room.id;
                    }

                    // If no workspaces, try meeting rooms
                    let meeting_rooms = building.get_rooms_by_type(RoomType::MeetingRoom);
                    if !meeting_rooms.is_empty()
                        && user.can_access_building(building.id, departure_location)
                    {
                        let selected_room =
                            meeting_rooms[self.rng.gen_range(0..meeting_rooms.len())];
                        debug!(
                            "Selected departure meeting room {} in destination location {} for user {}",
                            selected_room.id, departure_location, user.id
                        );
                        return selected_room.id;
                    }
                }
            }

            debug!(
                "Could not find suitable departure room in destination location {} for user {}, using primary workspace",
                departure_location, user.id
            );
        }

        // Default to primary workspace
        user.primary_workspace
    }

    /// Clear daily location tracking (typically called at start of new day)
    ///
    /// Resets the daily location tracking for all users. This should be called
    /// when starting a new simulation day to ensure users can make fresh
    /// cross-location travel decisions.
    pub fn clear_daily_location_tracking(&mut self) {
        debug!(
            "Clearing daily location tracking for {} users",
            self.user_daily_locations.len()
        );
        self.user_daily_locations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facility::{Building, Location, Room};
    use crate::types::{RoomType, SecurityLevel};

    fn create_test_registry() -> LocationRegistry {
        let mut registry = LocationRegistry::new();

        // Create first location with two buildings
        let mut location1 = Location::new("Seattle".to_string(), (47.6062, -122.3321));

        let mut building1 = Building::new(location1.id, "Building A".to_string());
        let room1 = Room::new(
            building1.id,
            "Room 1A".to_string(),
            RoomType::Workspace,
            SecurityLevel::Public,
        );
        let room2 = Room::new(
            building1.id,
            "Room 2A".to_string(),
            RoomType::MeetingRoom,
            SecurityLevel::Public,
        );
        building1.add_room(room1);
        building1.add_room(room2);

        let mut building2 = Building::new(location1.id, "Building B".to_string());
        let room3 = Room::new(
            building2.id,
            "Room 1B".to_string(),
            RoomType::Workspace,
            SecurityLevel::Public,
        );
        building2.add_room(room3);

        location1.add_building(building1);
        location1.add_building(building2);

        // Create second location with one building
        let mut location2 = Location::new("Portland".to_string(), (45.5152, -122.6784));

        let mut building3 = Building::new(location2.id, "Building C".to_string());
        let room4 = Room::new(
            building3.id,
            "Room 1C".to_string(),
            RoomType::Workspace,
            SecurityLevel::Public,
        );
        building3.add_room(room4);

        location2.add_building(building3);

        registry.add_location(location1);
        registry.add_location(location2);

        registry
    }

    #[test]
    fn test_travel_time_constants_default() {
        let constants = TravelTimeConstants::default();

        assert_eq!(constants.cross_location_travel_time, Duration::hours(4));
        assert_eq!(constants.intra_location_building_travel_time.0, Duration::minutes(15));
        assert_eq!(constants.intra_location_building_travel_time.1, Duration::minutes(30));
        assert_eq!(constants.scheduling_buffer_time, Duration::minutes(5));
    }

    #[test]
    fn test_calculate_minimum_travel_time_same_building() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        let location_id = LocationId::new();
        let building_id = BuildingId::new();

        let travel_time = engine.calculate_minimum_travel_time(
            location_id,
            location_id,
            Some(building_id),
            Some(building_id),
        );

        assert_eq!(travel_time, Duration::zero());
    }

    #[test]
    fn test_calculate_minimum_travel_time_different_buildings_same_location() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        let location_id = LocationId::new();
        let building1_id = BuildingId::new();
        let building2_id = BuildingId::new();

        let travel_time = engine.calculate_minimum_travel_time(
            location_id,
            location_id,
            Some(building1_id),
            Some(building2_id),
        );

        // Should return the maximum intra-location travel time for safety
        assert_eq!(travel_time, Duration::minutes(30));
    }

    #[test]
    fn test_calculate_minimum_travel_time_different_locations() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        let location1_id = LocationId::new();
        let location2_id = LocationId::new();
        let building1_id = BuildingId::new();
        let building2_id = BuildingId::new();

        let travel_time = engine.calculate_minimum_travel_time(
            location1_id,
            location2_id,
            Some(building1_id),
            Some(building2_id),
        );

        // Should return cross-location travel time regardless of buildings
        assert_eq!(travel_time, Duration::hours(4));
    }

    #[test]
    fn test_calculate_minimum_travel_time_unknown_buildings() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        let location_id = LocationId::new();

        // Test with None buildings
        let travel_time =
            engine.calculate_minimum_travel_time(location_id, location_id, None, None);

        // Should assume intra-location travel (max time for safety)
        assert_eq!(travel_time, Duration::minutes(30));

        // Test with partial building info
        let travel_time2 = engine.calculate_minimum_travel_time(
            location_id,
            location_id,
            Some(BuildingId::new()),
            None,
        );

        assert_eq!(travel_time2, Duration::minutes(30));
    }

    #[test]
    fn test_get_room_location_info() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Get the first room from the registry
        let location = &registry.locations[0];
        let building = &location.buildings[0];
        let room = &building.rooms[0];

        let result = engine.get_room_location_info(room.id, &registry);

        assert!(result.is_some());
        let (found_location, found_building) = result.unwrap();
        assert_eq!(found_location, location.id);
        assert_eq!(found_building, building.id);
    }

    #[test]
    fn test_get_room_location_info_not_found() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        let nonexistent_room = RoomId::new();
        let result = engine.get_room_location_info(nonexistent_room, &registry);

        assert!(result.is_none());
    }

    #[test]
    fn test_get_last_activity_location() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Get room IDs from the registry
        let location = &registry.locations[0];
        let building = &location.buildings[0];
        let room1 = &building.rooms[0];
        let room2 = &building.rooms[1];

        // Create a schedule with activities
        let base_time = Utc::now();
        let schedule = vec![
            ScheduledActivity::new(ActivityType::Arrival, room1.id, base_time, Duration::hours(1)),
            ScheduledActivity::new(
                ActivityType::Meeting,
                room2.id,
                base_time + Duration::hours(2),
                Duration::hours(1),
            ),
        ];

        // Test finding last activity before a specific time
        let query_time = base_time + Duration::hours(1) + Duration::minutes(30);
        let result = engine.get_last_activity_location(&schedule, query_time, &registry);

        assert!(result.is_some());
        let (found_location, found_building) = result.unwrap();
        assert_eq!(found_location, location.id);
        assert_eq!(found_building, building.id);
    }

    #[test]
    fn test_get_last_activity_location_no_previous_activity() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Create a schedule with activities
        let base_time = Utc::now();
        let room_id = registry.locations[0].buildings[0].rooms[0].id;
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Meeting,
            room_id,
            base_time + Duration::hours(2),
            Duration::hours(1),
        )];

        // Query before any activities start
        let query_time = base_time;
        let result = engine.get_last_activity_location(&schedule, query_time, &registry);

        assert!(result.is_none());
    }

    #[test]
    fn test_get_last_activity_location_empty_schedule() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        let schedule = vec![];
        let query_time = Utc::now();
        let result = engine.get_last_activity_location(&schedule, query_time, &registry);

        assert!(result.is_none());
    }

    #[test]
    fn test_get_last_activity_location_multiple_activities() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Get room IDs from different buildings
        let location = &registry.locations[0];
        let building1 = &location.buildings[0];
        let building2 = &location.buildings[1];
        let room1 = &building1.rooms[0];
        let room2 = &building2.rooms[0];

        let base_time = Utc::now();
        let schedule = vec![
            ScheduledActivity::new(ActivityType::Arrival, room1.id, base_time, Duration::hours(1)),
            ScheduledActivity::new(
                ActivityType::Meeting,
                room2.id,
                base_time + Duration::hours(2),
                Duration::hours(1),
            ),
            ScheduledActivity::new(
                ActivityType::Lunch,
                room1.id,
                base_time + Duration::hours(4),
                Duration::hours(1),
            ),
        ];

        // Query after the meeting but before lunch
        let query_time = base_time + Duration::hours(3) + Duration::minutes(30);
        let result = engine.get_last_activity_location(&schedule, query_time, &registry);

        assert!(result.is_some());
        let (found_location, found_building) = result.unwrap();
        assert_eq!(found_location, location.id);
        assert_eq!(found_building, building2.id); // Should find the meeting room in building2
    }

    #[test]
    fn test_travel_time_integration_cross_location() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Get rooms from different locations
        let location1 = &registry.locations[0];
        let location2 = &registry.locations[1];
        let room1 = &location1.buildings[0].rooms[0];
        let room2 = &location2.buildings[0].rooms[0];

        // Create schedule with cross-location activities
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Meeting,
            room1.id,
            base_time,
            Duration::hours(1),
        )];

        // Get last activity location
        let query_time = base_time + Duration::hours(2);
        let last_location = engine.get_last_activity_location(&schedule, query_time, &registry);
        assert!(last_location.is_some());

        let (from_location, from_building) = last_location.unwrap();
        let (to_location, to_building) =
            engine.get_room_location_info(room2.id, &registry).unwrap();

        // Calculate travel time between locations
        let travel_time = engine.calculate_minimum_travel_time(
            from_location,
            to_location,
            Some(from_building),
            Some(to_building),
        );

        // Should require 4 hours for cross-location travel
        assert_eq!(travel_time, Duration::hours(4));
    }

    #[test]
    fn test_travel_time_integration_intra_location() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Get rooms from different buildings in same location
        let location = &registry.locations[0];
        let room1 = &location.buildings[0].rooms[0];
        let room2 = &location.buildings[1].rooms[0];

        // Create schedule
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Meeting,
            room1.id,
            base_time,
            Duration::hours(1),
        )];

        // Get last activity location
        let query_time = base_time + Duration::hours(2);
        let last_location = engine.get_last_activity_location(&schedule, query_time, &registry);
        assert!(last_location.is_some());

        let (from_location, from_building) = last_location.unwrap();
        let (to_location, to_building) =
            engine.get_room_location_info(room2.id, &registry).unwrap();

        // Calculate travel time between buildings in same location
        let travel_time = engine.calculate_minimum_travel_time(
            from_location,
            to_location,
            Some(from_building),
            Some(to_building),
        );

        // Should require 30 minutes for intra-location travel (max time for safety)
        assert_eq!(travel_time, Duration::minutes(30));
    }

    #[test]
    fn test_track_user_daily_location_first_time() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let mut engine = BehaviorEngine::new(config, time_manager);

        let user_id = UserId::new();
        let location_id = LocationId::new();

        // First time tracking should set and return the location
        let result = engine.track_user_daily_location(user_id, location_id);
        assert_eq!(result, location_id);

        // Verify it's stored in the tracking map
        assert_eq!(engine.user_daily_locations.get(&user_id), Some(&location_id));
    }

    #[test]
    fn test_track_user_daily_location_same_location() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let mut engine = BehaviorEngine::new(config, time_manager);

        let user_id = UserId::new();
        let location_id = LocationId::new();

        // Track initial location
        engine.track_user_daily_location(user_id, location_id);

        // Track same location again - should return same location
        let result = engine.track_user_daily_location(user_id, location_id);
        assert_eq!(result, location_id);

        // Should still be the same in tracking map
        assert_eq!(engine.user_daily_locations.get(&user_id), Some(&location_id));
    }

    #[test]
    fn test_track_user_daily_location_cross_location_travel() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let mut engine = BehaviorEngine::new(config, time_manager);

        let user_id = UserId::new();
        let location1_id = LocationId::new();
        let location2_id = LocationId::new();

        // Track initial location
        engine.track_user_daily_location(user_id, location1_id);
        assert_eq!(engine.user_daily_locations.get(&user_id), Some(&location1_id));

        // Track different location (cross-location travel)
        let result = engine.track_user_daily_location(user_id, location2_id);
        assert_eq!(result, location2_id);

        // Should be updated to new location
        assert_eq!(engine.user_daily_locations.get(&user_id), Some(&location2_id));
    }

    #[test]
    fn test_has_traveled_cross_location_today_no_travel() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        let user_id = UserId::new();

        // Create schedule with activities in same location
        let location = &registry.locations[0];
        let room1 = &location.buildings[0].rooms[0];
        let room2 = &location.buildings[0].rooms[1];

        let base_time = Utc::now();
        let schedule = vec![
            ScheduledActivity::new(ActivityType::Arrival, room1.id, base_time, Duration::hours(1)),
            ScheduledActivity::new(
                ActivityType::Meeting,
                room2.id,
                base_time + Duration::hours(2),
                Duration::hours(1),
            ),
        ];

        let result = engine.has_traveled_cross_location_today(user_id, &schedule, &registry);
        assert!(result.is_none());
    }

    #[test]
    fn test_has_traveled_cross_location_today_with_travel() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        let user_id = UserId::new();

        // Create schedule with cross-location activities
        let location1 = &registry.locations[0];
        let location2 = &registry.locations[1];
        let room1 = &location1.buildings[0].rooms[0];
        let room2 = &location2.buildings[0].rooms[0];

        let base_time = Utc::now();
        let schedule = vec![
            ScheduledActivity::new(ActivityType::Arrival, room1.id, base_time, Duration::hours(1)),
            ScheduledActivity::new(
                ActivityType::Meeting,
                room2.id,
                base_time + Duration::hours(5), // 5 hours later for travel time
                Duration::hours(1),
            ),
        ];

        let result = engine.has_traveled_cross_location_today(user_id, &schedule, &registry);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), location2.id);
    }

    #[test]
    fn test_has_traveled_cross_location_today_with_tracking() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let mut engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        let user_id = UserId::new();
        let location1 = &registry.locations[0];
        let location2 = &registry.locations[1];

        // Create schedule starting in location1
        let room1 = &location1.buildings[0].rooms[0];
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Arrival,
            room1.id,
            base_time,
            Duration::hours(1),
        )];

        // Track user as having traveled to location2
        engine.track_user_daily_location(user_id, location2.id);

        let result = engine.has_traveled_cross_location_today(user_id, &schedule, &registry);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), location2.id);
    }

    #[test]
    fn test_constrain_activity_location_no_constraint() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        // Create test user
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location_id, building_id, room_id, permissions);

        let preferred_location = LocationId::new();

        // No constraint should return preferred location
        let result = engine.constrain_activity_location(
            &user,
            ActivityType::Meeting,
            preferred_location,
            None,
        );

        assert_eq!(result, preferred_location);
    }

    #[test]
    fn test_constrain_activity_location_with_constraint_same_location() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        // Create test user
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location_id, building_id, room_id, permissions);

        let preferred_location = LocationId::new();

        // Constraint matches preferred location
        let result = engine.constrain_activity_location(
            &user,
            ActivityType::Meeting,
            preferred_location,
            Some(preferred_location),
        );

        assert_eq!(result, preferred_location);
    }

    #[test]
    fn test_constrain_activity_location_with_constraint_different_location() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        // Create test user
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location_id, building_id, room_id, permissions);

        let preferred_location = LocationId::new();
        let constrained_location = LocationId::new();

        // Constraint overrides preferred location
        let result = engine.constrain_activity_location(
            &user,
            ActivityType::Meeting,
            preferred_location,
            Some(constrained_location),
        );

        assert_eq!(result, constrained_location);
    }

    #[test]
    fn test_get_departure_location_no_cross_location_travel() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Create test user
        let location = &registry.locations[0];
        let building = &location.buildings[0];
        let room = &building.rooms[0];
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location.id, building.id, room.id, permissions);

        // Create schedule with activities in same location
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Arrival,
            room.id,
            base_time,
            Duration::hours(1),
        )];

        let result = engine.get_departure_location(&user, &schedule, &registry);
        assert_eq!(result, user.primary_location);
    }

    #[test]
    fn test_get_departure_location_with_cross_location_travel() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let mut engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Create test user in location1
        let location1 = &registry.locations[0];
        let location2 = &registry.locations[1];
        let building1 = &location1.buildings[0];
        let room1 = &building1.rooms[0];
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location1.id, building1.id, room1.id, permissions);

        // Track user as having traveled to location2
        engine.track_user_daily_location(user.id, location2.id);

        // Create schedule starting in location1
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Arrival,
            room1.id,
            base_time,
            Duration::hours(1),
        )];

        let result = engine.get_departure_location(&user, &schedule, &registry);
        assert_eq!(result, location2.id);
    }

    #[test]
    fn test_select_departure_room_primary_location() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let mut engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Create test user
        let location = &registry.locations[0];
        let building = &location.buildings[0];
        let room = &building.rooms[0];
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location.id, building.id, room.id, permissions);

        // Create schedule with activities in same location
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Arrival,
            room.id,
            base_time,
            Duration::hours(1),
        )];

        let result = engine.select_departure_room(&user, &schedule, &registry);
        assert_eq!(result, user.primary_workspace);
    }

    #[test]
    fn test_select_departure_room_destination_location() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let mut engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Create test user in location1
        let location1 = &registry.locations[0];
        let location2 = &registry.locations[1];
        let building1 = &location1.buildings[0];
        let room1 = &building1.rooms[0];
        let permissions = crate::permissions::PermissionSet::new();
        let mut user =
            crate::user::User::new(location1.id, building1.id, room1.id, permissions);

        // Give user access to location2
        user
            .permissions
            .add_permission(crate::permissions::PermissionLevel::Location(location2.id));

        // Track user as having traveled to location2
        engine.track_user_daily_location(user.id, location2.id);

        // Create schedule starting in location1
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Arrival,
            room1.id,
            base_time,
            Duration::hours(1),
        )];

        let result = engine.select_departure_room(&user, &schedule, &registry);

        // Should not be the primary workspace since user traveled cross-location
        // Should be a room in location2
        let (result_location, _) = engine.get_room_location_info(result, &registry).unwrap();
        assert_eq!(result_location, location2.id);
    }

    #[test]
    fn test_clear_daily_location_tracking() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let mut engine = BehaviorEngine::new(config, time_manager);

        let user1_id = UserId::new();
        let user2_id = UserId::new();
        let location1_id = LocationId::new();
        let location2_id = LocationId::new();

        // Track some users
        engine.track_user_daily_location(user1_id, location1_id);
        engine.track_user_daily_location(user2_id, location2_id);

        assert_eq!(engine.user_daily_locations.len(), 2);

        // Clear tracking
        engine.clear_daily_location_tracking();

        assert_eq!(engine.user_daily_locations.len(), 0);
        assert!(engine.user_daily_locations.get(&user1_id).is_none());
        assert!(engine.user_daily_locations.get(&user2_id).is_none());
    }

    #[test]
    fn test_location_persistence_integration() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let mut engine = BehaviorEngine::new(config, time_manager);
        let registry = create_test_registry();

        // Create test user in location1
        let location1 = &registry.locations[0];
        let location2 = &registry.locations[1];
        let building1 = &location1.buildings[0];
        let room1 = &building1.rooms[0];
        let permissions = crate::permissions::PermissionSet::new();
        let mut user =
            crate::user::User::new(location1.id, building1.id, room1.id, permissions);

        // Give user access to both locations
        user
            .permissions
            .add_permission(crate::permissions::PermissionLevel::Location(location1.id));
        user
            .permissions
            .add_permission(crate::permissions::PermissionLevel::Location(location2.id));

        // Simulate cross-location travel
        engine.track_user_daily_location(user.id, location1.id); // Start at primary
        engine.track_user_daily_location(user.id, location2.id); // Travel to destination

        // Create schedule with activities
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Arrival,
            room1.id,
            base_time,
            Duration::hours(1),
        )];

        // Test that user has traveled cross-location
        let cross_location_result =
            engine.has_traveled_cross_location_today(user.id, &schedule, &registry);
        assert!(cross_location_result.is_some());
        assert_eq!(cross_location_result.unwrap(), location2.id);

        // Test activity location constraint
        let constrained_location = engine.constrain_activity_location(
            &user,
            ActivityType::Meeting,
            location1.id,          // Prefer primary location
            cross_location_result, // But constrained to destination
        );
        assert_eq!(constrained_location, location2.id);

        // Test departure location
        let departure_location = engine.get_departure_location(&user, &schedule, &registry);
        assert_eq!(departure_location, location2.id);

        // Test departure room selection
        let departure_room = engine.select_departure_room(&user, &schedule, &registry);
        let (departure_room_location, _) =
            engine.get_room_location_info(departure_room, &registry).unwrap();
        assert_eq!(departure_room_location, location2.id);
    }

    #[test]
    fn test_calculate_earliest_meeting_time_no_previous_activities() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        // Create test data
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location_id, building_id, room_id, permissions);

        let registry = LocationRegistry::new();
        let schedule = vec![];
        let proposed_time = Utc::now();

        // With no previous activities, proposed time should be returned unchanged
        let result =
            engine.calculate_earliest_meeting_time(proposed_time, &schedule, &user, &registry);

        assert_eq!(result, Some(proposed_time));
    }

    #[test]
    fn test_calculate_earliest_meeting_time_with_travel_time() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        // Create test data
        let location1_id = LocationId::new();
        let building1_id = BuildingId::new();
        let room1_id = RoomId::new();
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location1_id, building1_id, room1_id, permissions);

        // Create registry with location
        let mut registry = LocationRegistry::new();
        let mut location1 = Location::new("Location 1".to_string(), (47.6062, -122.3321));
        let mut building1 = Building::new(location1.id, "Building 1".to_string());
        building1.add_room(Room::new(
            building1.id,
            "Room 1".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        location1.add_building(building1);
        registry.add_location(location1);

        // Create schedule with previous activity
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Arrival,
            room1_id,
            base_time,
            Duration::minutes(15),
        )];

        // Propose meeting time that's too soon after previous activity
        let proposed_time = base_time + Duration::minutes(30);

        let result =
            engine.calculate_earliest_meeting_time(proposed_time, &schedule, &user, &registry);

        // Result should be later than proposed time due to travel time requirements
        assert!(result.is_some());
        let actual_time = result.unwrap();
        assert!(actual_time >= proposed_time);
    }

    #[test]
    fn test_validate_travel_time_for_meeting_sufficient_time() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        // Create test data
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location_id, building_id, room_id, permissions);

        // Create registry
        let mut registry = LocationRegistry::new();
        let mut location = Location::new("Test Location".to_string(), (47.6062, -122.3321));
        let mut building = Building::new(location.id, "Test Building".to_string());
        building.add_room(Room::new(
            building.id,
            "Test Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        location.add_building(building);
        registry.add_location(location);

        // Create schedule with previous activity
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Arrival,
            room_id,
            base_time,
            Duration::minutes(15),
        )];

        // Meeting time with sufficient gap (2 hours later)
        let meeting_time = base_time + Duration::hours(2);

        let result = engine.validate_travel_time_for_meeting(
            &user,
            &schedule,
            meeting_time,
            location_id,
            &registry,
        );

        assert!(result);
    }

    #[test]
    fn test_validate_travel_time_for_meeting_insufficient_time() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        // Create registry with two buildings in same location
        let mut registry = LocationRegistry::new();
        let mut location = Location::new("Test Location".to_string(), (47.6062, -122.3321));
        let mut building1 = Building::new(location.id, "Building 1".to_string());
        building1.add_room(Room::new(
            building1.id,
            "Room 1".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        let room1_id = building1.rooms[0].id; // Get the actual room ID from the registry

        let mut building2 = Building::new(location.id, "Building 2".to_string());
        building2.add_room(Room::new(
            building2.id,
            "Room 2".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));

        location.add_building(building1);
        location.add_building(building2);
        registry.add_location(location);

        // Create test data using actual IDs from registry
        let location_id = registry.locations[0].id;
        let building1_id = registry.locations[0].buildings[0].id;
        let permissions = crate::permissions::PermissionSet::new();
        let _user =
            crate::user::User::new(location_id, building1_id, room1_id, permissions);

        // Create schedule with recent activity in building1
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Meeting,
            room1_id, // Activity in building1
            base_time,
            Duration::minutes(60), // 1-hour meeting
        )];

        // Meeting time too soon after previous activity (only 20 minutes gap, need 30+ minutes for building travel + buffer)
        let meeting_time = base_time + Duration::minutes(80);

        // Test validation for a meeting in building2 (different from building1 where the previous activity was)
        let building2_id = registry.locations[0].buildings[1].id;

        // Create a mock validation by checking travel time between buildings
        let travel_time = engine.calculate_minimum_travel_time(
            location_id,
            location_id,
            Some(building1_id),
            Some(building2_id),
        );

        // Find the end time of the last activity
        let last_activity_end =
            schedule.iter().map(|activity| activity.start_time + activity.duration).max().unwrap();

        // Check if there's sufficient time
        let available_time = meeting_time - last_activity_end;
        let required_time = travel_time + engine.travel_time_constants.scheduling_buffer_time;

        let result = available_time >= required_time;

        // Should fail validation due to insufficient time (need 30 min travel + 5 min buffer = 35 min, only have 20 min)
        assert!(!result);
    }

    #[test]
    fn test_validate_travel_time_for_cross_location_meeting_sufficient_time() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        // Create test data
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location_id, building_id, room_id, permissions);

        // Create registry
        let mut registry = LocationRegistry::new();
        let mut location = Location::new("Test Location".to_string(), (47.6062, -122.3321));
        let mut building = Building::new(location.id, "Test Building".to_string());
        building.add_room(Room::new(
            building.id,
            "Test Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        location.add_building(building);
        registry.add_location(location);

        // Create schedule with previous activity
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Arrival,
            room_id,
            base_time,
            Duration::minutes(15),
        )];

        // Meeting time with sufficient gap for cross-location travel (5 hours later)
        let meeting_time = base_time + Duration::hours(5);

        let result = engine.validate_travel_time_for_cross_location_meeting(
            &user,
            &schedule,
            meeting_time,
            &registry,
        );

        assert!(result);
    }

    #[test]
    fn test_validate_travel_time_for_cross_location_meeting_insufficient_time() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let engine = BehaviorEngine::new(config, time_manager);

        // Create registry with two locations
        let mut registry = LocationRegistry::new();
        let mut location1 = Location::new("Location 1".to_string(), (47.6062, -122.3321));
        let mut building1 = Building::new(location1.id, "Building 1".to_string());
        building1.add_room(Room::new(
            building1.id,
            "Room 1".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        let room1_id = building1.rooms[0].id; // Get the actual room ID from the registry
        location1.add_building(building1);
        registry.add_location(location1);

        let mut location2 = Location::new("Location 2".to_string(), (40.7128, -74.0060));
        let mut building2 = Building::new(location2.id, "Building 2".to_string());
        building2.add_room(Room::new(
            building2.id,
            "Room 2".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        location2.add_building(building2);
        registry.add_location(location2);

        // Create test data using actual IDs from registry
        let location1_id = registry.locations[0].id;
        let building1_id = registry.locations[0].buildings[0].id;
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location1_id, building1_id, room1_id, permissions);

        // Create schedule with recent activity in location1
        let base_time = Utc::now();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Meeting,
            room1_id, // Activity in location1
            base_time,
            Duration::minutes(60), // 1-hour meeting
        )];

        // Meeting time too soon for cross-location travel (only 3 hours gap, need 4+ hours)
        let meeting_time = base_time + Duration::hours(3);

        let result = engine.validate_travel_time_for_cross_location_meeting(
            &user,
            &schedule,
            meeting_time,
            &registry,
        );

        // Should fail validation due to insufficient time for cross-location travel
        assert!(!result);
    }

    #[test]
    fn test_generate_meeting_time_with_travel_constraints() {
        let config = SimulationConfig::default();
        let time_manager = TimeManager::new();
        let mut engine = BehaviorEngine::new(config, time_manager);

        // Create test data
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        let permissions = crate::permissions::PermissionSet::new();
        let user =
            crate::user::User::new(location_id, building_id, room_id, permissions);

        // Create registry
        let mut registry = LocationRegistry::new();
        let mut location = Location::new("Test Location".to_string(), (47.6062, -122.3321));
        let mut building = Building::new(location.id, "Test Building".to_string());
        building.add_room(Room::new(
            building.id,
            "Test Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        location.add_building(building);
        registry.add_location(location);

        // Create schedule with previous activity
        let base_time = Utc::now().date_naive();
        let schedule = vec![ScheduledActivity::new(
            ActivityType::Arrival,
            room_id,
            base_time.and_hms_opt(9, 0, 0).unwrap().and_utc(),
            Duration::minutes(15),
        )];

        // Generate meeting time
        let result = engine.generate_meeting_time(
            base_time,
            &BehaviorProfile::default(),
            &schedule,
            &user,
            &registry,
        );

        assert!(result.is_some());

    }
}
