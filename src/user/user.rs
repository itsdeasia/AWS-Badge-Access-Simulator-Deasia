//! Core user struct and methods
//!
//! This module contains the User struct and all its core functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::user::{BehaviorProfile, UserState, ScheduledActivity};
use crate::permissions::{PermissionLevel, PermissionSet};
use crate::types::{BuildingId, UserId, LocationId, RoomId};

/// Represents a user in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for the user
    pub id: UserId,
    /// User's primary location (where they usually work)
    pub primary_location: LocationId,
    /// User's primary building within their location
    pub primary_building: BuildingId,
    /// User's assigned workspace (desk/office)
    pub primary_workspace: RoomId,
    /// Set of permissions for room/building/location access
    pub permissions: PermissionSet,
    /// Whether this user exhibits curious behavior (attempts unauthorized access)
    pub is_curious: bool,
    /// Whether this user has a cloned badge (for impossible traveler scenarios)
    pub has_cloned_badge: bool,
    /// Whether this user works night shifts
    pub is_night_shift: bool,
    /// Building assigned for night-shift work (only relevant for night-shift users)
    pub assigned_night_building: Option<BuildingId>,
    /// Behavioral characteristics of the user
    pub behavior_profile: BehaviorProfile,
    /// Current state and location of the user
    pub current_state: UserState,
}

impl User {
    /// Create a new user
    pub fn new(
        primary_location: LocationId,
        primary_building: BuildingId,
        primary_workspace: RoomId,
        permissions: PermissionSet,
    ) -> Self {
        let user_id = UserId::new();
        let current_time = Utc::now();

        Self {
            id: user_id,
            primary_location,
            primary_building,
            primary_workspace,
            permissions,
            is_curious: false,
            has_cloned_badge: false,
            is_night_shift: false,
            assigned_night_building: None,
            behavior_profile: BehaviorProfile::default(),
            current_state: UserState::new(primary_building, primary_location, current_time),
        }
    }

    /// Create a new user with curious behavior
    pub fn new_curious(
        primary_location: LocationId,
        primary_building: BuildingId,
        primary_workspace: RoomId,
        permissions: PermissionSet,
    ) -> Self {
        let mut user =
            Self::new(primary_location, primary_building, primary_workspace, permissions);
        user.is_curious = true;
        user.behavior_profile = BehaviorProfile::curious();
        user
    }

    /// Create a new user with a cloned badge
    pub fn new_with_cloned_badge(
        primary_location: LocationId,
        primary_building: BuildingId,
        primary_workspace: RoomId,
        permissions: PermissionSet,
    ) -> Self {
        let mut user =
            Self::new(primary_location, primary_building, primary_workspace, permissions);
        user.has_cloned_badge = true;
        user
    }

    /// Create a new night-shift user
    pub fn new_night_shift(
        primary_location: LocationId,
        primary_building: BuildingId,
        primary_workspace: RoomId,
        permissions: PermissionSet,
        assigned_night_building: BuildingId,
    ) -> Self {
        let mut user =
            Self::new(primary_location, primary_building, primary_workspace, permissions);
        user.is_night_shift = true;
        user.assigned_night_building = Some(assigned_night_building);
        user
    }

    /// Check if the user can access a specific room
    pub fn can_access_room(
        &self,
        room_id: RoomId,
        building_id: BuildingId,
        location_id: LocationId,
    ) -> bool {
        self.permissions.can_access_room(room_id, building_id, location_id)
    }

    /// Check if the user can access a building
    pub fn can_access_building(&self, building_id: BuildingId, location_id: LocationId) -> bool {
        self.permissions.can_access_building(building_id, location_id)
    }

    /// Check if the user can access a location
    pub fn can_access_location(&self, location_id: LocationId) -> bool {
        self.permissions.can_access_location(location_id)
    }

    /// Add a permission to the user
    pub fn add_permission(&mut self, permission: PermissionLevel) {
        self.permissions.add_permission(permission);
    }

    /// Remove a permission from the user
    pub fn remove_permission(&mut self, permission: &PermissionLevel) {
        self.permissions.remove_permission(permission);
    }

    /// Check if the user is currently at their primary workspace
    pub fn is_at_primary_workspace(&self) -> bool {
        self.current_state.is_in_room(self.primary_workspace)
    }

    /// Check if the user is currently in their primary building
    pub fn is_in_primary_building(&self) -> bool {
        self.current_state.is_in_building(self.primary_building)
    }

    /// Check if the user is currently in their primary location
    pub fn is_in_primary_location(&self) -> bool {
        self.current_state.is_in_location(self.primary_location)
    }

    /// Move the user to a specific room
    pub fn move_to_room(&mut self, room_id: RoomId, timestamp: DateTime<Utc>) {
        self.current_state.move_to_room(room_id, timestamp);
    }

    /// Move the user to a specific building
    pub fn move_to_building(&mut self, building_id: BuildingId, timestamp: DateTime<Utc>) {
        self.current_state.move_to_building(building_id, timestamp);
    }

    /// Move the user to a specific location
    pub fn move_to_location(&mut self, location_id: LocationId, timestamp: DateTime<Utc>) {
        self.current_state.move_to_location(location_id, timestamp);
    }

    /// Set the user's daily schedule
    pub fn set_daily_schedule(&mut self, schedule: Vec<ScheduledActivity>) {
        self.current_state.set_daily_schedule(schedule);
    }

    /// Get the user's current activity
    pub fn get_current_activity(&self, current_time: DateTime<Utc>) -> Option<&ScheduledActivity> {
        self.current_state.get_current_activity(current_time)
    }

    /// Get the user's next scheduled activity
    pub fn get_next_activity(&self, current_time: DateTime<Utc>) -> Option<&ScheduledActivity> {
        self.current_state.get_next_activity(current_time)
    }

    /// Check if the user should attempt unauthorized access based on curiosity
    pub fn should_attempt_unauthorized_access(&self, base_probability: f64) -> bool {
        if !self.is_curious {
            return false;
        }

        // Curious users have higher probability of unauthorized access attempts
        let curiosity_multiplier = 1.0 + self.behavior_profile.curiosity_level;
        let adjusted_probability = base_probability * curiosity_multiplier;

        // Use a simple probability check (in real implementation, would use proper RNG)
        adjusted_probability > 0.5
    }

    /// Check if the user is likely to travel to a different location
    pub fn should_travel_to_different_location(&self, base_probability: f64) -> bool {
        let travel_multiplier = 1.0 + self.behavior_profile.travel_frequency;
        let adjusted_probability = base_probability * travel_multiplier;

        adjusted_probability > 0.5
    }

    /// Get all rooms the user is authorized to access
    pub fn get_authorized_rooms(&self) -> Vec<RoomId> {
        self.permissions.get_authorized_rooms()
    }

    /// Get all buildings the user is authorized to access
    pub fn get_authorized_buildings(&self) -> Vec<BuildingId> {
        self.permissions.get_authorized_buildings()
    }

    /// Get all locations the user is authorized to access
    pub fn get_authorized_locations(&self) -> Vec<LocationId> {
        self.permissions.get_authorized_locations()
    }

    /// Validate that the user has consistent permissions and assignments
    pub fn validate(&self) -> Result<(), String> {
        // Check that primary workspace is accessible
        if !self.permissions.get_authorized_rooms().contains(&self.primary_workspace) {
            // Check if user has building or location level access
            if !self.can_access_building(self.primary_building, self.primary_location) {
                return Err("User does not have access to their primary workspace".to_string());
            }
        }

        // Check that behavior profile values are in valid ranges
        if !(0.0..=1.0).contains(&self.behavior_profile.travel_frequency) {
            return Err("Travel frequency must be between 0.0 and 1.0".to_string());
        }
        if !(0.0..=1.0).contains(&self.behavior_profile.curiosity_level) {
            return Err("Curiosity level must be between 0.0 and 1.0".to_string());
        }
        if !(0.0..=1.0).contains(&self.behavior_profile.schedule_adherence) {
            return Err("Schedule adherence must be between 0.0 and 1.0".to_string());
        }
        if !(0.0..=1.0).contains(&self.behavior_profile.social_level) {
            return Err("Social level must be between 0.0 and 1.0".to_string());
        }

        Ok(())
    }

    /// Mark this user as having a cloned badge for impossible traveler scenarios
    pub fn mark_badge_as_cloned(&mut self) {
        self.has_cloned_badge = true;
    }

    /// Remove the cloned badge marking from this user
    pub fn unmark_cloned_badge(&mut self) {
        self.has_cloned_badge = false;
    }

    /// Check if this user is eligible for badge cloning based on their profile
    ///
    /// Users with higher travel frequency are more likely candidates for
    /// badge cloning scenarios as they would have more opportunities for their
    /// badges to be compromised during travel.
    pub fn is_eligible_for_badge_cloning(&self) -> bool {
        // Users who travel frequently are more likely to have their badges cloned
        self.behavior_profile.travel_frequency > 0.1 ||
        // Users with location-level permissions are also good candidates
        !self.permissions.get_authorized_locations().is_empty()
    }

    /// Create a new user for testing purposes with minimal setup
    #[cfg(test)]
    pub fn new_for_testing(
        id: UserId,
        primary_location: LocationId,
        primary_building: BuildingId,
        is_curious: bool,
        has_cloned_badge: bool,
    ) -> Self {
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();
        let current_time = Utc::now();

        Self {
            id,
            primary_location,
            primary_building,
            primary_workspace: workspace_id,
            permissions,
            is_curious,
            has_cloned_badge,
            is_night_shift: false,
            assigned_night_building: None,
            behavior_profile: if is_curious { BehaviorProfile::curious() } else { BehaviorProfile::default() },
            current_state: UserState::new(primary_building, primary_location, current_time),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_user_creation() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let user = User::new(location_id, building_id, workspace_id, permissions);

        assert_eq!(user.primary_location, location_id);
        assert_eq!(user.primary_building, building_id);
        assert_eq!(user.primary_workspace, workspace_id);
        assert!(!user.is_curious);
        assert!(!user.has_cloned_badge);
        assert_eq!(user.current_state.current_building, building_id);
        assert_eq!(user.current_state.current_location, location_id);
    }

    #[test]
    fn test_curious_user_creation() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let user = User::new_curious(location_id, building_id, workspace_id, permissions);

        assert!(user.is_curious);
        assert!(user.behavior_profile.is_curious());
        assert!(!user.has_cloned_badge);
    }

    #[test]
    fn test_cloned_badge_user_creation() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let user =
            User::new_with_cloned_badge(location_id, building_id, workspace_id, permissions);

        assert!(!user.is_curious);
        assert!(user.has_cloned_badge);
    }

    #[test]
    fn test_user_permission_management() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let room_id = RoomId::new();
        let permissions = PermissionSet::new();

        let mut user = User::new(location_id, building_id, workspace_id, permissions);

        // Add room permission
        user.add_permission(PermissionLevel::Room(room_id));
        assert!(user.can_access_room(room_id, building_id, location_id));
        assert!(user.get_authorized_rooms().contains(&room_id));

        // Remove room permission
        user.remove_permission(&PermissionLevel::Room(room_id));
        assert!(!user.get_authorized_rooms().contains(&room_id));
    }

    #[test]
    fn test_user_movement() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let mut user = User::new(location_id, building_id, workspace_id, permissions);

        // Test room movement
        let room_id = RoomId::new();
        let timestamp = Utc::now();
        user.move_to_room(room_id, timestamp);
        assert_eq!(user.current_state.current_room, Some(room_id));

        // Test building movement
        let new_building_id = BuildingId::new();
        user.move_to_building(new_building_id, timestamp);
        assert_eq!(user.current_state.current_building, new_building_id);
        assert!(!user.is_in_primary_building());

        // Test location movement
        let new_location_id = LocationId::new();
        user.move_to_location(new_location_id, timestamp);
        assert_eq!(user.current_state.current_location, new_location_id);
        assert!(!user.is_in_primary_location());
    }

    #[test]
    fn test_user_primary_location_checks() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let mut user = User::new(location_id, building_id, workspace_id, permissions);

        // Initially should be in primary location and building
        assert!(user.is_in_primary_location());
        assert!(user.is_in_primary_building());
        assert!(!user.is_at_primary_workspace()); // Not in specific room yet

        // Move to primary workspace
        user.move_to_room(workspace_id, Utc::now());
        assert!(user.is_at_primary_workspace());
    }

    #[test]
    fn test_user_schedule_management() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let mut user = User::new(location_id, building_id, workspace_id, permissions);

        // Create a schedule
        let room_id = RoomId::new();
        let start_time = Utc::now() + Duration::hours(1);
        let activities = vec![ScheduledActivity::new(
            crate::types::ActivityType::Meeting,
            room_id,
            start_time,
            Duration::hours(1),
        )];

        user.set_daily_schedule(activities);

        // Test getting activities
        let current_time = Utc::now();
        assert!(user.get_current_activity(current_time).is_none());

        let next_activity = user.get_next_activity(current_time);
        assert!(next_activity.is_some());
        assert_eq!(next_activity.unwrap().target_room, room_id);
    }

    #[test]
    fn test_user_behavior_decisions() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let mut user =
            User::new_curious(location_id, building_id, workspace_id, permissions);

        // Curious user should be more likely to attempt unauthorized access
        assert!(user.should_attempt_unauthorized_access(0.3));

        // Non-curious user should not attempt unauthorized access
        user.is_curious = false;
        assert!(!user.should_attempt_unauthorized_access(0.3));

        // Test travel behavior
        user.behavior_profile.travel_frequency = 0.8;
        assert!(user.should_travel_to_different_location(0.3)); // 0.3 * 1.8 = 0.54 > 0.5
    }

    #[test]
    fn test_user_validation() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let mut permissions = PermissionSet::new();

        // Add workspace permission
        permissions.add_permission(PermissionLevel::Room(workspace_id));

        let user = User::new(location_id, building_id, workspace_id, permissions);

        // Should validate successfully
        assert!(user.validate().is_ok());

        // Test invalid behavior profile
        let mut invalid_user = user.clone();
        invalid_user.behavior_profile.travel_frequency = 1.5; // Invalid range
        assert!(invalid_user.validate().is_err());
    }

    #[test]
    fn test_user_badge_cloning() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let mut user = User::new(location_id, building_id, workspace_id, permissions);

        // Initially not cloned
        assert!(!user.has_cloned_badge);

        // Mark as cloned
        user.mark_badge_as_cloned();
        assert!(user.has_cloned_badge);

        // Unmark cloned badge
        user.unmark_cloned_badge();
        assert!(!user.has_cloned_badge);

        // Test eligibility for badge cloning
        user.behavior_profile.travel_frequency = 0.2;
        assert!(user.is_eligible_for_badge_cloning());

        user.behavior_profile.travel_frequency = 0.05;
        assert!(!user.is_eligible_for_badge_cloning());
    }

    #[test]
    fn test_night_shift_user_creation() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let night_building_id = BuildingId::new();
        let permissions = PermissionSet::new();

        let user = User::new_night_shift(
            location_id,
            building_id,
            workspace_id,
            permissions,
            night_building_id,
        );

        // Verify night-shift designation
        assert!(user.is_night_shift);
        assert_eq!(user.assigned_night_building, Some(night_building_id));

        // Verify other fields are set correctly
        assert_eq!(user.primary_location, location_id);
        assert_eq!(user.primary_building, building_id);
        assert_eq!(user.primary_workspace, workspace_id);
        assert!(!user.is_curious);
        assert!(!user.has_cloned_badge);
        assert_eq!(user.current_state.current_building, building_id);
        assert_eq!(user.current_state.current_location, location_id);
    }

    #[test]
    fn test_regular_user_not_night_shift() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let user = User::new(location_id, building_id, workspace_id, permissions);

        // Verify regular user is not night-shift
        assert!(!user.is_night_shift);
        assert_eq!(user.assigned_night_building, None);
    }

    #[test]
    fn test_night_shift_user_with_different_buildings() {
        let location_id = LocationId::new();
        let primary_building_id = BuildingId::new();
        let night_building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let user = User::new_night_shift(
            location_id,
            primary_building_id,
            workspace_id,
            permissions,
            night_building_id,
        );

        // Verify night-shift user can have different primary and night buildings
        assert!(user.is_night_shift);
        assert_eq!(user.primary_building, primary_building_id);
        assert_eq!(user.assigned_night_building, Some(night_building_id));
        assert_ne!(user.primary_building, night_building_id);
    }

    #[test]
    fn test_night_shift_user_with_same_building() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let user = User::new_night_shift(
            location_id,
            building_id,
            workspace_id,
            permissions,
            building_id, // Same building for primary and night shift
        );

        // Verify night-shift user can have same primary and night building
        assert!(user.is_night_shift);
        assert_eq!(user.primary_building, building_id);
        assert_eq!(user.assigned_night_building, Some(building_id));
    }
}
