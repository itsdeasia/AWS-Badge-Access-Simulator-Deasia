//! Permission set management and access validation
//!
//! This module contains the PermissionSet struct and related access validation logic.

use crate::types::{BuildingId, LocationId, RoomId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Permission levels for access control
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// Access to a specific room
    Room(RoomId),
    /// Access to all rooms in a building
    Building(BuildingId),
    /// Access to all rooms in all buildings at a location
    Location(LocationId),
}

impl fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionLevel::Room(id) => write!(f, "Room({})", id),
            PermissionLevel::Building(id) => write!(f, "Building({})", id),
            PermissionLevel::Location(id) => write!(f, "Location({})", id),
        }
    }
}

/// Set of permissions for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    /// List of permissions granted to the user
    pub permissions: Vec<PermissionLevel>,
}

impl PermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        Self { permissions: Vec::new() }
    }

    /// Create a permission set with the given permissions
    pub fn with_permissions(permissions: Vec<PermissionLevel>) -> Self {
        Self { permissions }
    }

    /// Add a permission to the set
    pub fn add_permission(&mut self, permission: PermissionLevel) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    /// Remove a permission from the set
    pub fn remove_permission(&mut self, permission: &PermissionLevel) {
        self.permissions.retain(|p| p != permission);
    }

    /// Check if the user can access a specific room
    pub fn can_access_room(
        &self,
        room_id: RoomId,
        building_id: BuildingId,
        location_id: LocationId,
    ) -> bool {
        self.permissions.iter().any(|perm| match perm {
            PermissionLevel::Room(id) => *id == room_id,
            PermissionLevel::Building(id) => *id == building_id,
            PermissionLevel::Location(id) => *id == location_id,
        })
    }

    /// Check if the user can access any room in a building
    pub fn can_access_building(&self, building_id: BuildingId, location_id: LocationId) -> bool {
        self.permissions.iter().any(|perm| match perm {
            PermissionLevel::Building(id) => *id == building_id,
            PermissionLevel::Location(id) => *id == location_id,
            PermissionLevel::Room(_) => false, // Room-level permissions don't grant building access
        })
    }

    /// Check if the user can access any room in a location
    pub fn can_access_location(&self, location_id: LocationId) -> bool {
        self.permissions.iter().any(|perm| match perm {
            PermissionLevel::Location(id) => *id == location_id,
            PermissionLevel::Building(_) | PermissionLevel::Room(_) => false,
        })
    }

    /// Get all room IDs the user has direct access to
    pub fn get_authorized_rooms(&self) -> Vec<RoomId> {
        self.permissions
            .iter()
            .filter_map(|perm| match perm {
                PermissionLevel::Room(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    /// Get all building IDs the user has access to
    pub fn get_authorized_buildings(&self) -> Vec<BuildingId> {
        self.permissions
            .iter()
            .filter_map(|perm| match perm {
                PermissionLevel::Building(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    /// Get all location IDs the user has access to
    pub fn get_authorized_locations(&self) -> Vec<LocationId> {
        self.permissions
            .iter()
            .filter_map(|perm| match perm {
                PermissionLevel::Location(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    /// Check if the permission set is empty
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    /// Get the number of permissions
    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    /// Check if the user has any permissions for a specific location
    pub fn has_any_location_permissions(&self, location_id: LocationId) -> bool {
        self.permissions.iter().any(|perm| match perm {
            PermissionLevel::Location(id) => *id == location_id,
            PermissionLevel::Building(_) | PermissionLevel::Room(_) => {
                // Would need location registry to check, but this is a basic check
                false
            }
        })
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set_creation() {
        let permission_set = PermissionSet::new();
        assert!(permission_set.is_empty());
        assert_eq!(permission_set.len(), 0);
    }

    #[test]
    fn test_permission_set_with_permissions() {
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let permissions =
            vec![PermissionLevel::Room(room_id), PermissionLevel::Building(building_id)];
        let permission_set = PermissionSet::with_permissions(permissions.clone());
        assert_eq!(permission_set.permissions, permissions);
        assert_eq!(permission_set.len(), 2);
    }

    #[test]
    fn test_add_permission() {
        let mut permission_set = PermissionSet::new();
        let room_id = RoomId::new();
        let permission = PermissionLevel::Room(room_id);

        permission_set.add_permission(permission.clone());
        assert_eq!(permission_set.len(), 1);
        assert!(permission_set.permissions.contains(&permission));

        // Adding the same permission again should not duplicate it
        permission_set.add_permission(permission.clone());
        assert_eq!(permission_set.len(), 1);
    }

    #[test]
    fn test_remove_permission() {
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let room_permission = PermissionLevel::Room(room_id);
        let building_permission = PermissionLevel::Building(building_id);

        let mut permission_set = PermissionSet::with_permissions(vec![
            room_permission.clone(),
            building_permission.clone(),
        ]);

        permission_set.remove_permission(&room_permission);
        assert_eq!(permission_set.len(), 1);
        assert!(!permission_set.permissions.contains(&room_permission));
        assert!(permission_set.permissions.contains(&building_permission));
    }

    #[test]
    fn test_can_access_room() {
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        // Test room-level permission
        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Room(room_id));
        assert!(permission_set.can_access_room(room_id, building_id, location_id));

        // Test building-level permission
        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Building(building_id));
        assert!(permission_set.can_access_room(room_id, building_id, location_id));

        // Test location-level permission
        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Location(location_id));
        assert!(permission_set.can_access_room(room_id, building_id, location_id));

        // Test no permission
        let permission_set = PermissionSet::new();
        assert!(!permission_set.can_access_room(room_id, building_id, location_id));
    }

    #[test]
    fn test_can_access_building() {
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let room_id = RoomId::new();

        // Test building-level permission
        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Building(building_id));
        assert!(permission_set.can_access_building(building_id, location_id));

        // Test location-level permission
        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Location(location_id));
        assert!(permission_set.can_access_building(building_id, location_id));

        // Test room-level permission (should not grant building access)
        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Room(room_id));
        assert!(!permission_set.can_access_building(building_id, location_id));
    }

    #[test]
    fn test_can_access_location() {
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();

        // Test location-level permission
        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Location(location_id));
        assert!(permission_set.can_access_location(location_id));

        // Test building-level permission (should not grant location access)
        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Building(building_id));
        assert!(!permission_set.can_access_location(location_id));

        // Test room-level permission (should not grant location access)
        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Room(room_id));
        assert!(!permission_set.can_access_location(location_id));
    }

    #[test]
    fn test_get_authorized_rooms() {
        let room1 = RoomId::new();
        let room2 = RoomId::new();
        let building_id = BuildingId::new();

        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Room(room1));
        permission_set.add_permission(PermissionLevel::Room(room2));
        permission_set.add_permission(PermissionLevel::Building(building_id));

        let authorized_rooms = permission_set.get_authorized_rooms();
        assert_eq!(authorized_rooms.len(), 2);
        assert!(authorized_rooms.contains(&room1));
        assert!(authorized_rooms.contains(&room2));
    }

    #[test]
    fn test_get_authorized_buildings() {
        let building1 = BuildingId::new();
        let building2 = BuildingId::new();
        let room_id = RoomId::new();

        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Building(building1));
        permission_set.add_permission(PermissionLevel::Building(building2));
        permission_set.add_permission(PermissionLevel::Room(room_id));

        let authorized_buildings = permission_set.get_authorized_buildings();
        assert_eq!(authorized_buildings.len(), 2);
        assert!(authorized_buildings.contains(&building1));
        assert!(authorized_buildings.contains(&building2));
    }

    #[test]
    fn test_get_authorized_locations() {
        let location1 = LocationId::new();
        let location2 = LocationId::new();
        let building_id = BuildingId::new();

        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Location(location1));
        permission_set.add_permission(PermissionLevel::Location(location2));
        permission_set.add_permission(PermissionLevel::Building(building_id));

        let authorized_locations = permission_set.get_authorized_locations();
        assert_eq!(authorized_locations.len(), 2);
        assert!(authorized_locations.contains(&location1));
        assert!(authorized_locations.contains(&location2));
    }

    #[test]
    fn test_has_any_location_permissions() {
        let location_id = LocationId::new();
        let other_location_id = LocationId::new();

        let mut permission_set = PermissionSet::new();
        permission_set.add_permission(PermissionLevel::Location(location_id));

        assert!(permission_set.has_any_location_permissions(location_id));
        assert!(!permission_set.has_any_location_permissions(other_location_id));
    }

    #[test]
    fn test_permission_level_display() {
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let room_permission = PermissionLevel::Room(room_id);
        let building_permission = PermissionLevel::Building(building_id);
        let location_permission = PermissionLevel::Location(location_id);

        assert_eq!(format!("{}", room_permission), format!("Room({})", room_id));
        assert_eq!(format!("{}", building_permission), format!("Building({})", building_id));
        assert_eq!(format!("{}", location_permission), format!("Location({})", location_id));
    }

    #[test]
    fn test_default_permission_set() {
        let permission_set = PermissionSet::default();
        assert!(permission_set.is_empty());
        assert_eq!(permission_set.len(), 0);
    }
}
