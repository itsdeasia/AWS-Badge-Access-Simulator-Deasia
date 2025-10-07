//! User state management and activity scheduling
//!
//! This module contains user state tracking and scheduled activity management.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{ActivityType, BuildingId, LocationId, RoomId};

/// Scheduled activity for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledActivity {
    /// Type of activity
    pub activity_type: ActivityType,
    /// Target room for the activity
    pub target_room: RoomId,
    /// When the activity starts
    pub start_time: DateTime<Utc>,
    /// How long the activity lasts
    pub duration: Duration,
}

impl ScheduledActivity {
    /// Create a new scheduled activity
    pub fn new(
        activity_type: ActivityType,
        target_room: RoomId,
        start_time: DateTime<Utc>,
        duration: Duration,
    ) -> Self {
        Self { activity_type, target_room, start_time, duration }
    }

    /// Get the end time of the activity
    pub fn end_time(&self) -> DateTime<Utc> {
        self.start_time + self.duration
    }

    /// Check if the activity is currently active
    pub fn is_active(&self, current_time: DateTime<Utc>) -> bool {
        current_time >= self.start_time && current_time <= self.end_time()
    }

    /// Check if the activity has finished
    pub fn is_finished(&self, current_time: DateTime<Utc>) -> bool {
        current_time > self.end_time()
    }

    /// Check if the activity is in the future
    pub fn is_future(&self, current_time: DateTime<Utc>) -> bool {
        current_time < self.start_time
    }
}

/// Current state of a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    /// Current room the user is in (None if outside all buildings)
    pub current_room: Option<RoomId>,
    /// Current building the user is in
    pub current_building: BuildingId,
    /// Current location the user is in
    pub current_location: LocationId,
    /// When the user last moved or performed an activity
    pub last_activity_time: DateTime<Utc>,
    /// The user's daily schedule
    pub daily_schedule: Vec<ScheduledActivity>,
    /// Index of the current activity in the daily schedule
    pub current_activity_index: usize,
}

impl UserState {
    /// Create a new user state
    pub fn new(
        current_building: BuildingId,
        current_location: LocationId,
        last_activity_time: DateTime<Utc>,
    ) -> Self {
        Self {
            current_room: None,
            current_building,
            current_location,
            last_activity_time,
            daily_schedule: Vec::new(),
            current_activity_index: 0,
        }
    }

    /// Update the user's current room
    pub fn move_to_room(&mut self, room_id: RoomId, timestamp: DateTime<Utc>) {
        self.current_room = Some(room_id);
        self.last_activity_time = timestamp;
    }

    /// Update the user's current building
    pub fn move_to_building(&mut self, building_id: BuildingId, timestamp: DateTime<Utc>) {
        self.current_building = building_id;
        self.current_room = None; // Clear room when changing buildings
        self.last_activity_time = timestamp;
    }

    /// Update the user's current location
    pub fn move_to_location(&mut self, location_id: LocationId, timestamp: DateTime<Utc>) {
        self.current_location = location_id;
        self.current_room = None; // Clear room when changing locations
        self.last_activity_time = timestamp;
    }

    /// Set the daily schedule for the user
    pub fn set_daily_schedule(&mut self, schedule: Vec<ScheduledActivity>) {
        self.daily_schedule = schedule;
        self.current_activity_index = 0;
    }

    /// Get the current activity if any
    pub fn get_current_activity(&self, current_time: DateTime<Utc>) -> Option<&ScheduledActivity> {
        self.daily_schedule.iter().find(|activity| activity.is_active(current_time))
    }

    /// Get the next scheduled activity
    pub fn get_next_activity(&self, current_time: DateTime<Utc>) -> Option<&ScheduledActivity> {
        self.daily_schedule.iter().find(|activity| activity.is_future(current_time))
    }

    /// Advance to the next activity in the schedule
    pub fn advance_to_next_activity(&mut self) {
        if self.current_activity_index < self.daily_schedule.len() {
            self.current_activity_index += 1;
        }
    }

    /// Check if the user is currently in a specific room
    pub fn is_in_room(&self, room_id: RoomId) -> bool {
        self.current_room == Some(room_id)
    }

    /// Check if the user is currently in a specific building
    pub fn is_in_building(&self, building_id: BuildingId) -> bool {
        self.current_building == building_id
    }

    /// Check if the user is currently in a specific location
    pub fn is_in_location(&self, location_id: LocationId) -> bool {
        self.current_location == location_id
    }

    /// Get time since last activity
    pub fn time_since_last_activity(&self, current_time: DateTime<Utc>) -> Duration {
        current_time - self.last_activity_time
    }

    /// Check if the user has been idle for too long
    pub fn is_idle(&self, current_time: DateTime<Utc>, idle_threshold: Duration) -> bool {
        self.time_since_last_activity(current_time) > idle_threshold
    }

    /// Generate a night-shift schedule for a user
    /// Creates an inverted schedule where the user starts inside the building,
    /// exits in the morning, and returns in the late afternoon
    pub fn generate_night_shift_schedule(&mut self, building_rooms: &[RoomId]) {
        let mut schedule = Vec::new();
        let base_time = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

        // Night-shift users start inside the building at the beginning of the day
        // They patrol rooms throughout the night (using existing rooms for patrol activities)

        // Early morning patrol (2 AM - 4 AM) - Visit accessible rooms
        if !building_rooms.is_empty() {
            let patrol_start = base_time + Duration::hours(2);
            let patrol_duration = Duration::minutes(30);

            for (i, &room_id) in building_rooms.iter().enumerate() {
                let activity_start = patrol_start + Duration::minutes(i as i64 * 45);
                schedule.push(ScheduledActivity::new(
                    ActivityType::NightPatrol,
                    room_id,
                    activity_start,
                    patrol_duration,
                ));
            }
        }

        // Break time (4:30 AM) - Use bathroom/break room if available
        if let Some(&first_room) = building_rooms.first() {
            let break_time = base_time + Duration::hours(4) + Duration::minutes(30);
            schedule.push(ScheduledActivity::new(
                ActivityType::Bathroom,
                first_room,
                break_time,
                Duration::minutes(15),
            ));
        }

        // Second patrol round (5 AM - 7 AM) - Visit rooms again using NightPatrol activity
        if !building_rooms.is_empty() {
            let second_patrol_start = base_time + Duration::hours(5);
            let patrol_duration = Duration::minutes(20);

            for (i, &room_id) in building_rooms.iter().enumerate() {
                let activity_start = second_patrol_start + Duration::minutes(i as i64 * 30);
                schedule.push(ScheduledActivity::new(
                    ActivityType::NightPatrol, // Use NightPatrol for all patrol activities
                    room_id,
                    activity_start,
                    patrol_duration,
                ));
            }
        }

        // Morning departure (8 AM) - Exit building
        if let Some(&exit_room) = building_rooms.first() {
            let departure_time = base_time + Duration::hours(8);
            schedule.push(ScheduledActivity::new(
                ActivityType::Departure,
                exit_room,
                departure_time,
                Duration::minutes(10),
            ));
        }

        // Late afternoon arrival (5 PM) - Return to building
        if let Some(&entry_room) = building_rooms.first() {
            let arrival_time = base_time + Duration::hours(17);
            schedule.push(ScheduledActivity::new(
                ActivityType::Arrival,
                entry_room,
                arrival_time,
                Duration::minutes(10),
            ));
        }

        // Evening patrol setup (6 PM onwards) - Prepare for night shift
        if !building_rooms.is_empty() {
            let evening_start = base_time + Duration::hours(18);
            let setup_duration = Duration::minutes(30);

            for (i, &room_id) in building_rooms.iter().enumerate().take(3) {
                let activity_start = evening_start + Duration::minutes(i as i64 * 45);
                schedule.push(ScheduledActivity::new(
                    ActivityType::NightPatrol, // Use NightPatrol for evening setup activities
                    room_id,
                    activity_start,
                    setup_duration,
                ));
            }
        }

        // Set the generated schedule using existing method
        self.set_daily_schedule(schedule);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_scheduled_activity_creation() {
        let room_id = RoomId::new();
        let start_time = Utc::now();
        let duration = Duration::hours(1);

        let activity = ScheduledActivity::new(ActivityType::Meeting, room_id, start_time, duration);

        assert_eq!(activity.activity_type, ActivityType::Meeting);
        assert_eq!(activity.target_room, room_id);
        assert_eq!(activity.start_time, start_time);
        assert_eq!(activity.duration, duration);
        assert_eq!(activity.end_time(), start_time + duration);
    }

    #[test]
    fn test_scheduled_activity_timing() {
        let room_id = RoomId::new();
        let start_time = Utc::now();
        let duration = Duration::hours(1);

        let activity = ScheduledActivity::new(ActivityType::Meeting, room_id, start_time, duration);

        // Test future activity
        let before_start = start_time - Duration::minutes(30);
        assert!(activity.is_future(before_start));
        assert!(!activity.is_active(before_start));
        assert!(!activity.is_finished(before_start));

        // Test active activity
        let during_activity = start_time + Duration::minutes(30);
        assert!(!activity.is_future(during_activity));
        assert!(activity.is_active(during_activity));
        assert!(!activity.is_finished(during_activity));

        // Test finished activity
        let after_end = start_time + Duration::hours(2);
        assert!(!activity.is_future(after_end));
        assert!(!activity.is_active(after_end));
        assert!(activity.is_finished(after_end));
    }

    #[test]
    fn test_user_state_creation() {
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let state = UserState::new(building_id, location_id, timestamp);

        assert_eq!(state.current_building, building_id);
        assert_eq!(state.current_location, location_id);
        assert_eq!(state.last_activity_time, timestamp);
        assert_eq!(state.current_room, None);
        assert!(state.daily_schedule.is_empty());
        assert_eq!(state.current_activity_index, 0);
    }

    #[test]
    fn test_user_state_movement() {
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let mut state = UserState::new(building_id, location_id, timestamp);

        // Test room movement
        let room_id = RoomId::new();
        let new_timestamp = timestamp + Duration::minutes(30);
        state.move_to_room(room_id, new_timestamp);

        assert_eq!(state.current_room, Some(room_id));
        assert_eq!(state.last_activity_time, new_timestamp);
        assert!(state.is_in_room(room_id));

        // Test building movement (should clear room)
        let new_building_id = BuildingId::new();
        let building_timestamp = new_timestamp + Duration::minutes(30);
        state.move_to_building(new_building_id, building_timestamp);

        assert_eq!(state.current_building, new_building_id);
        assert_eq!(state.current_room, None);
        assert_eq!(state.last_activity_time, building_timestamp);
        assert!(state.is_in_building(new_building_id));

        // Test location movement (should clear room)
        let new_location_id = LocationId::new();
        let location_timestamp = building_timestamp + Duration::minutes(30);
        state.move_to_location(new_location_id, location_timestamp);

        assert_eq!(state.current_location, new_location_id);
        assert_eq!(state.current_room, None);
        assert_eq!(state.last_activity_time, location_timestamp);
        assert!(state.is_in_location(new_location_id));
    }

    #[test]
    fn test_user_state_schedule_management() {
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let mut state = UserState::new(building_id, location_id, timestamp);

        // Create a schedule with multiple activities
        let room1 = RoomId::new();
        let room2 = RoomId::new();
        let start_time = Utc::now();

        let activities = vec![
            ScheduledActivity::new(
                ActivityType::Meeting,
                room1,
                start_time + Duration::hours(1),
                Duration::hours(1),
            ),
            ScheduledActivity::new(
                ActivityType::Lunch,
                room2,
                start_time + Duration::hours(3),
                Duration::minutes(30),
            ),
        ];

        state.set_daily_schedule(activities);

        assert_eq!(state.daily_schedule.len(), 2);
        assert_eq!(state.current_activity_index, 0);

        // Test getting current activity (none should be active yet)
        assert!(state.get_current_activity(start_time).is_none());

        // Test getting next activity
        let next_activity = state.get_next_activity(start_time);
        assert!(next_activity.is_some());
        assert_eq!(next_activity.unwrap().activity_type, ActivityType::Meeting);

        // Test advancing activity index
        state.advance_to_next_activity();
        assert_eq!(state.current_activity_index, 1);
    }

    #[test]
    fn test_user_state_idle_detection() {
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let state = UserState::new(building_id, location_id, timestamp);

        // Test not idle
        let current_time = timestamp + Duration::minutes(30);
        let idle_threshold = Duration::hours(1);
        assert!(!state.is_idle(current_time, idle_threshold));

        // Test idle
        let later_time = timestamp + Duration::hours(2);
        assert!(state.is_idle(later_time, idle_threshold));

        // Test time since last activity
        let time_diff = state.time_since_last_activity(later_time);
        assert_eq!(time_diff, Duration::hours(2));
    }

    #[test]
    fn test_generate_night_shift_schedule_empty_rooms() {
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let mut state = UserState::new(building_id, location_id, timestamp);
        let empty_rooms: Vec<RoomId> = vec![];

        state.generate_night_shift_schedule(&empty_rooms);

        // Should have empty schedule when no rooms provided
        assert!(state.daily_schedule.is_empty());
        assert_eq!(state.current_activity_index, 0);
    }

    #[test]
    fn test_generate_night_shift_schedule_single_room() {
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let mut state = UserState::new(building_id, location_id, timestamp);
        let room_id = RoomId::new();
        let rooms = vec![room_id];

        state.generate_night_shift_schedule(&rooms);

        // Should have activities scheduled
        assert!(!state.daily_schedule.is_empty());
        assert_eq!(state.current_activity_index, 0);

        // Check that we have the expected activity types
        let activity_types: Vec<ActivityType> =
            state.daily_schedule.iter().map(|activity| activity.activity_type).collect();

        // Should include NightPatrol, Bathroom, Departure, and Arrival (no meetings for night-shift)
        assert!(activity_types.contains(&ActivityType::NightPatrol));
        assert!(activity_types.contains(&ActivityType::Bathroom));
        assert!(activity_types.contains(&ActivityType::Departure));
        assert!(activity_types.contains(&ActivityType::Arrival));

        // All activities should target the provided room
        for activity in &state.daily_schedule {
            assert_eq!(activity.target_room, room_id);
        }
    }

    #[test]
    fn test_generate_night_shift_schedule_multiple_rooms() {
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let mut state = UserState::new(building_id, location_id, timestamp);
        let room1 = RoomId::new();
        let room2 = RoomId::new();
        let room3 = RoomId::new();
        let rooms = vec![room1, room2, room3];

        state.generate_night_shift_schedule(&rooms);

        // Should have activities scheduled
        assert!(!state.daily_schedule.is_empty());

        // Should have patrol activities for each room (multiple patrol rounds)
        let patrol_activities: Vec<&ScheduledActivity> = state
            .daily_schedule
            .iter()
            .filter(|activity| activity.activity_type == ActivityType::NightPatrol)
            .collect();

        // Should have multiple patrol activities (first patrol round + second patrol round + evening patrol)
        // 3 rooms * 3 patrol rounds = 9 patrol activities
        assert_eq!(patrol_activities.len(), 9);

        // Should not have any meeting activities (night-shift users don't attend meetings)
        let meeting_activities: Vec<&ScheduledActivity> = state
            .daily_schedule
            .iter()
            .filter(|activity| activity.activity_type == ActivityType::Meeting)
            .collect();

        assert!(meeting_activities.is_empty());

        // Check that all provided rooms are used in activities
        let used_rooms: std::collections::HashSet<RoomId> =
            state.daily_schedule.iter().map(|activity| activity.target_room).collect();

        assert!(used_rooms.contains(&room1));
        assert!(used_rooms.contains(&room2));
        assert!(used_rooms.contains(&room3));
    }

    #[test]
    fn test_generate_night_shift_schedule_timing() {
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let mut state = UserState::new(building_id, location_id, timestamp);
        let room_id = RoomId::new();
        let rooms = vec![room_id];

        state.generate_night_shift_schedule(&rooms);

        // Check that activities are scheduled at appropriate times
        let base_time = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

        // Find departure activity (should be around 8 AM)
        let departure_activity = state
            .daily_schedule
            .iter()
            .find(|activity| activity.activity_type == ActivityType::Departure);

        assert!(departure_activity.is_some());
        let departure = departure_activity.unwrap();
        let expected_departure = base_time + Duration::hours(8);
        assert_eq!(departure.start_time, expected_departure);

        // Find arrival activity (should be around 5 PM)
        let arrival_activity = state
            .daily_schedule
            .iter()
            .find(|activity| activity.activity_type == ActivityType::Arrival);

        assert!(arrival_activity.is_some());
        let arrival = arrival_activity.unwrap();
        let expected_arrival = base_time + Duration::hours(17);
        assert_eq!(arrival.start_time, expected_arrival);

        // Find night patrol activity (should be around 2 AM)
        let patrol_activity = state
            .daily_schedule
            .iter()
            .find(|activity| activity.activity_type == ActivityType::NightPatrol);

        assert!(patrol_activity.is_some());
        let patrol = patrol_activity.unwrap();
        let expected_patrol = base_time + Duration::hours(2);
        assert_eq!(patrol.start_time, expected_patrol);
    }

    #[test]
    fn test_generate_night_shift_schedule_reuses_existing_method() {
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let mut state = UserState::new(building_id, location_id, timestamp);
        let room_id = RoomId::new();
        let rooms = vec![room_id];

        // Verify initial state
        assert!(state.daily_schedule.is_empty());
        assert_eq!(state.current_activity_index, 0);

        // Generate night shift schedule
        state.generate_night_shift_schedule(&rooms);

        // Verify that set_daily_schedule was used (schedule is set and index reset)
        assert!(!state.daily_schedule.is_empty());
        assert_eq!(state.current_activity_index, 0);

        // Test that we can still use existing schedule methods
        let base_time = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let current_time = base_time + Duration::hours(2) + Duration::minutes(15);

        // Should be able to get current activity during patrol time
        let current_activity = state.get_current_activity(current_time);
        assert!(current_activity.is_some());

        // Should be able to get next activity
        let next_activity = state.get_next_activity(base_time);
        assert!(next_activity.is_some());
    }
}
