//! User profiles for validation and analysis
//!
//! This module contains user profile structures for ground truth validation.

use serde::{Deserialize, Serialize};

use crate::user::{ActivityPreferences, BehaviorProfile, User};
use crate::types::{BuildingId, UserId, LocationId, RoomId, SimulationConfig};

/// User profile containing the "answer key" information for validation and analysis
/// This represents the ground truth about a user's intended behavior and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Unique identifier for the user
    pub user_id: UserId,
    /// User's primary location where they normally work
    pub primary_location: LocationId,
    /// User's primary building within their location
    pub primary_building: BuildingId,
    /// User's assigned workspace (desk/office)
    pub primary_workspace: RoomId,
    /// Complete set of authorized rooms (room-level permissions)
    pub authorized_rooms: Vec<RoomId>,
    /// Complete set of authorized buildings (building-level permissions)
    pub authorized_buildings: Vec<BuildingId>,
    /// Complete set of authorized locations (location-level permissions)
    pub authorized_locations: Vec<LocationId>,
    /// Whether this user is marked as curious (attempts unauthorized access)
    pub is_curious: bool,
    /// Whether this user has a cloned badge (generates impossible traveler scenarios)
    pub has_cloned_badge: bool,
    /// is user a night shift user
    pub is_night_shift: bool,
    /// Behavioral characteristics that influence activity patterns
    pub behavior_profile: BehaviorProfile,
    /// Expected activity patterns and preferences
    pub activity_preferences: ActivityPreferences,
    /// Travel patterns and location affinity
    pub travel_patterns: TravelPatterns,
}

/// Travel patterns that define how a user moves between locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelPatterns {
    /// Percentage of time spent in primary building (0.0-1.0)
    pub primary_building_affinity: f64,
    /// Percentage of time spent in other buildings within same location (0.0-1.0)
    pub same_location_travel_frequency: f64,
    /// Percentage of time spent in different geographical locations (0.0-1.0)
    pub cross_location_travel_frequency: f64,
    /// Typical travel time between locations (in hours)
    pub typical_cross_location_travel_time: f64,
    /// Locations this user is likely to visit (in order of frequency)
    pub frequent_locations: Vec<LocationId>,
    /// Buildings this user is likely to visit (in order of frequency)
    pub frequent_buildings: Vec<BuildingId>,
}

impl UserProfile {
    /// Create a new user profile from a user
    pub fn from_user(user: &User, config: &SimulationConfig) -> Self {
        Self {
            user_id: user.id,
            primary_location: user.primary_location,
            primary_building: user.primary_building,
            primary_workspace: user.primary_workspace,
            authorized_rooms: user.permissions.get_authorized_rooms(),
            authorized_buildings: user.permissions.get_authorized_buildings(),
            authorized_locations: user.permissions.get_authorized_locations(),
            is_curious: user.is_curious,
            has_cloned_badge: user.has_cloned_badge,
            is_night_shift: user.is_night_shift,
            behavior_profile: user.behavior_profile.clone(),
            activity_preferences: ActivityPreferences::from_behavior_profile(
                &user.behavior_profile,
            ),
            travel_patterns: TravelPatterns::from_config_and_user(config, user),
        }
    }

    /// Check if this user should have access to a specific room
    pub fn should_have_access_to_room(
        &self,
        room_id: RoomId,
        building_id: BuildingId,
        location_id: LocationId,
    ) -> bool {
        // Check room-level permissions
        if self.authorized_rooms.contains(&room_id) {
            return true;
        }

        // Check building-level permissions
        if self.authorized_buildings.contains(&building_id) {
            return true;
        }

        // Check location-level permissions
        if self.authorized_locations.contains(&location_id) {
            return true;
        }

        false
    }

    /// Get the expected behavior classification for this user
    pub fn get_behavior_classification(&self) -> String {
        if self.has_cloned_badge && self.is_curious {
            "cloned_badge_curious".to_string()
        } else if self.has_cloned_badge {
            "cloned_badge".to_string()
        } else if self.is_curious {
            "curious".to_string()
        } else {
            "normal".to_string()
        }
    }

    /// Get the risk level associated with this user
    pub fn get_risk_level(&self) -> String {
        if self.has_cloned_badge {
            "high".to_string()
        } else if self.is_curious {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }
}

impl TravelPatterns {
    /// Create travel patterns from configuration and user data
    pub fn from_config_and_user(config: &SimulationConfig, user: &User) -> Self {
        Self {
            primary_building_affinity: config.primary_building_affinity,
            same_location_travel_frequency: config.same_location_travel,
            cross_location_travel_frequency: config.different_location_travel,
            typical_cross_location_travel_time: 6.0, // 6 hours typical travel time
            frequent_locations: vec![user.primary_location], // Start with primary location
            frequent_buildings: vec![user.primary_building], // Start with primary building
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::PermissionSet;

    #[test]
    fn test_user_profile_creation() {
        let _user_id = UserId::new();
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let user = User::new(location_id, building_id, workspace_id, permissions);
        let config = SimulationConfig::default();

        let profile = UserProfile::from_user(&user, &config);

        assert_eq!(profile.user_id, user.id);
        assert_eq!(profile.primary_location, location_id);
        assert_eq!(profile.primary_building, building_id);
        assert_eq!(profile.primary_workspace, workspace_id);
        assert!(!profile.is_curious);
        assert!(!profile.has_cloned_badge);
    }

    #[test]
    fn test_user_profile_access_validation() {
        let user_id = UserId::new();
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let room_id = RoomId::new();

        let mut profile = UserProfile {
            user_id,
            primary_location: location_id,
            primary_building: building_id,
            primary_workspace: workspace_id,
            authorized_rooms: vec![room_id],
            authorized_buildings: vec![],
            authorized_locations: vec![],
            is_curious: false,
            has_cloned_badge: false,
            is_night_shift: false,
            behavior_profile: BehaviorProfile::default(),
            activity_preferences: ActivityPreferences::from_behavior_profile(
                &BehaviorProfile::default(),
            ),
            travel_patterns: TravelPatterns {
                primary_building_affinity: 0.8,
                same_location_travel_frequency: 0.1,
                cross_location_travel_frequency: 0.05,
                typical_cross_location_travel_time: 6.0,
                frequent_locations: vec![location_id],
                frequent_buildings: vec![building_id],
            },
        };

        // Test room-level access
        assert!(profile.should_have_access_to_room(room_id, building_id, location_id));

        // Test building-level access
        profile.authorized_rooms.clear();
        profile.authorized_buildings.push(building_id);
        assert!(profile.should_have_access_to_room(room_id, building_id, location_id));

        // Test location-level access
        profile.authorized_buildings.clear();
        profile.authorized_locations.push(location_id);
        assert!(profile.should_have_access_to_room(room_id, building_id, location_id));

        // Test no access
        profile.authorized_locations.clear();
        assert!(!profile.should_have_access_to_room(room_id, building_id, location_id));
    }

    #[test]
    fn test_behavior_classification() {
        let user_id = UserId::new();
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();

        let mut profile = UserProfile {
            user_id,
            primary_location: location_id,
            primary_building: building_id,
            primary_workspace: workspace_id,
            authorized_rooms: vec![],
            authorized_buildings: vec![],
            authorized_locations: vec![],
            is_curious: false,
            has_cloned_badge: false,
            is_night_shift: false,
            behavior_profile: BehaviorProfile::default(),
            activity_preferences: ActivityPreferences::from_behavior_profile(
                &BehaviorProfile::default(),
            ),
            travel_patterns: TravelPatterns {
                primary_building_affinity: 0.8,
                same_location_travel_frequency: 0.1,
                cross_location_travel_frequency: 0.05,
                typical_cross_location_travel_time: 6.0,
                frequent_locations: vec![location_id],
                frequent_buildings: vec![building_id],
            },
        };

        // Test normal user
        assert_eq!(profile.get_behavior_classification(), "normal");
        assert_eq!(profile.get_risk_level(), "low");

        // Test curious user
        profile.is_curious = true;
        assert_eq!(profile.get_behavior_classification(), "curious");
        assert_eq!(profile.get_risk_level(), "medium");

        // Test cloned badge user
        profile.is_curious = false;
        profile.has_cloned_badge = true;
        assert_eq!(profile.get_behavior_classification(), "cloned_badge");
        assert_eq!(profile.get_risk_level(), "high");

        // Test curious user with cloned badge
        profile.is_curious = true;
        profile.has_cloned_badge = true;
        assert_eq!(profile.get_behavior_classification(), "cloned_badge_curious");
        assert_eq!(profile.get_risk_level(), "high");
    }

    #[test]
    fn test_travel_patterns_creation() {
        let _user_id = UserId::new();
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();
        let permissions = PermissionSet::new();

        let user = User::new(location_id, building_id, workspace_id, permissions);
        let config = SimulationConfig::default();

        let travel_patterns = TravelPatterns::from_config_and_user(&config, &user);

        assert_eq!(travel_patterns.primary_building_affinity, config.primary_building_affinity);
        assert_eq!(travel_patterns.same_location_travel_frequency, config.same_location_travel);
        assert_eq!(
            travel_patterns.cross_location_travel_frequency,
            config.different_location_travel
        );
        assert_eq!(travel_patterns.typical_cross_location_travel_time, 6.0);
        assert_eq!(travel_patterns.frequent_locations, vec![location_id]);
        assert_eq!(travel_patterns.frequent_buildings, vec![building_id]);
    }
}
