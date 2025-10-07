//! Location registry and facility lookup system
//!
//! This module contains the LocationRegistry struct and related functionality for
//! managing collections of locations with efficient lookup capabilities and
//! cross-facility access flow calculations.

use crate::facility::{building::Building, location::Location, room::Room};
use crate::permissions::access_flow::AccessFlow;
use crate::simulation::time_manager::TimeManager;
use crate::types::{BuildingId, LocationId, RoomId};
use chrono::Duration;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A collection of locations with lookup capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRegistry {
    /// All locations in the simulation
    pub locations: Vec<Location>,
    /// Quick lookup map from location ID to index
    location_index: HashMap<LocationId, usize>,
    /// Quick lookup map from building ID to (location_index, building_index)
    building_index: HashMap<BuildingId, (usize, usize)>,
    /// Quick lookup map from room ID to (location_index, building_index, room_index)
    room_index: HashMap<RoomId, (usize, usize, usize)>,
}

impl LocationRegistry {
    /// Create a new empty location registry
    pub fn new() -> Self {
        Self {
            locations: Vec::new(),
            location_index: HashMap::new(),
            building_index: HashMap::new(),
            room_index: HashMap::new(),
        }
    }

    /// Add a location to the registry
    pub fn add_location(&mut self, location: Location) {
        let location_idx = self.locations.len();
        let location_id = location.id;

        // Index all buildings and rooms in this location
        for (building_idx, building) in location.buildings.iter().enumerate() {
            let building_id = building.id;
            self.building_index.insert(building_id, (location_idx, building_idx));

            for (room_idx, room) in building.rooms.iter().enumerate() {
                let room_id = room.id;
                self.room_index.insert(room_id, (location_idx, building_idx, room_idx));
            }
        }

        self.locations.push(location);
        self.location_index.insert(location_id, location_idx);
    }

    /// Rebuild the internal indices (call after modifying locations directly)
    pub fn rebuild_indices(&mut self) {
        self.location_index.clear();
        self.building_index.clear();
        self.room_index.clear();

        for (location_idx, location) in self.locations.iter().enumerate() {
            self.location_index.insert(location.id, location_idx);

            for (building_idx, building) in location.buildings.iter().enumerate() {
                self.building_index.insert(building.id, (location_idx, building_idx));

                for (room_idx, room) in building.rooms.iter().enumerate() {
                    self.room_index.insert(room.id, (location_idx, building_idx, room_idx));
                }
            }
        }
    }

    /// Get a location by ID
    pub fn get_location(&self, location_id: LocationId) -> Option<&Location> {
        self.location_index.get(&location_id).and_then(|&idx| self.locations.get(idx))
    }

    /// Get a building by ID
    pub fn get_building(&self, building_id: BuildingId) -> Option<&Building> {
        self.building_index.get(&building_id).and_then(|&(loc_idx, bld_idx)| {
            self.locations.get(loc_idx).and_then(|loc| loc.buildings.get(bld_idx))
        })
    }

    /// Get a room by ID
    pub fn get_room(&self, room_id: RoomId) -> Option<&Room> {
        self.room_index.get(&room_id).and_then(|&(loc_idx, bld_idx, room_idx)| {
            self.locations
                .get(loc_idx)
                .and_then(|loc| loc.buildings.get(bld_idx))
                .and_then(|bld| bld.rooms.get(room_idx))
        })
    }

    /// Get the location that contains a specific building
    pub fn get_location_for_building(&self, building_id: BuildingId) -> Option<&Location> {
        self.building_index.get(&building_id).and_then(|&(loc_idx, _)| self.locations.get(loc_idx))
    }

    /// Get the building that contains a specific room
    pub fn get_building_for_room(&self, room_id: RoomId) -> Option<&Building> {
        self.room_index.get(&room_id).and_then(|&(loc_idx, bld_idx, _)| {
            self.locations.get(loc_idx).and_then(|loc| loc.buildings.get(bld_idx))
        })
    }

    /// Get the location that contains a specific room
    pub fn get_location_for_room(&self, room_id: RoomId) -> Option<&Location> {
        self.room_index.get(&room_id).and_then(|&(loc_idx, _, _)| self.locations.get(loc_idx))
    }

    /// Get all locations
    pub fn get_all_locations(&self) -> &[Location] {
        &self.locations
    }

    /// Get all rooms across all locations
    pub fn get_all_rooms(&self) -> Vec<&Room> {
        self.locations
            .iter()
            .flat_map(|location| location.buildings.iter())
            .flat_map(|building| building.rooms.iter())
            .collect()
    }

    /// Get total number of locations
    pub fn location_count(&self) -> usize {
        self.locations.len()
    }

    /// Get total number of buildings across all locations
    pub fn total_building_count(&self) -> usize {
        self.locations.iter().map(|l| l.building_count()).sum()
    }

    /// Get total number of rooms across all locations
    pub fn total_room_count(&self) -> usize {
        self.locations.iter().map(|l| l.total_room_count()).sum()
    }

    /// Validate all locations in the registry
    pub fn validate(&self) -> Result<(), String> {
        if self.locations.is_empty() {
            return Err("Registry must have at least one location".to_string());
        }

        for location in &self.locations {
            if let Err(e) = location.validate() {
                return Err(format!("Location {} validation failed: {}", location.id, e));
            }
        }

        Ok(())
    }

    /// Get the complete access flow required to reach a target room from a starting room
    ///
    /// This is the main method for calculating access flows across the entire facility system.
    /// It handles same-room, same-building, same-location, and cross-location access patterns.
    ///
    /// # Arguments
    /// * `from_room` - Starting room (None if starting from outside all facilities)
    /// * `to_room` - Target room to access
    /// * `time_manager` - Time manager for calculating travel times
    /// * `rng` - Random number generator for realistic variance
    pub fn get_access_flow<R: Rng>(
        &self,
        from_room: Option<RoomId>,
        to_room: RoomId,
        time_manager: &TimeManager,
        rng: &mut R,
    ) -> Result<AccessFlow, String> {
        let target_location = self
            .get_location_for_room(to_room)
            .ok_or_else(|| format!("Target room {} not found in any location", to_room))?;

        // Check if we're staying in the same location
        if let Some(from_room_id) = from_room {
            if let Some(from_location) = self.get_location_for_room(from_room_id) {
                if from_location.id == target_location.id {
                    // Same location - delegate to location's access flow
                    return target_location.get_access_flow(from_room, to_room, time_manager, rng);
                }
            }
        }

        // Cross-location access - this involves significant travel time
        let target_location_flow =
            target_location.get_access_flow(None, to_room, time_manager, rng)?;

        // Calculate inter-location travel time (4-12 hours)
        let inter_location_travel_time = if from_room.is_some() {
            let hours = rng.gen_range(4..=12);
            Duration::hours(hours)
        } else {
            // Starting from outside all facilities
            Duration::minutes(rng.gen_range(10..=30))
        };

        let total_travel_time =
            target_location_flow.estimated_travel_time + inter_location_travel_time;

        Ok(AccessFlow::new(
            target_location_flow.required_sequence,
            total_travel_time,
            target_location_flow.requires_lobby_access,
            target_location_flow.involves_high_security,
        ))
    }

    /// Calculate travel time between two rooms anywhere in the facility system
    pub fn calculate_travel_time<R: Rng>(
        &self,
        from_room: Option<RoomId>,
        to_room: RoomId,
        time_manager: &TimeManager,
        rng: &mut R,
    ) -> Duration {
        let from_location = from_room.and_then(|room_id| self.get_location_for_room(room_id));
        let to_location = self.get_location_for_room(to_room);

        if let Some(to_loc) = to_location {
            if let Some(from_loc) = from_location {
                if from_loc.id == to_loc.id {
                    // Same location
                    to_loc.calculate_travel_time(from_room, to_room, time_manager, rng)
                } else {
                    // Different locations - use time manager's cross-location calculation
                    let from_building =
                        from_room.and_then(|room_id| self.get_building_for_room(room_id));
                    let to_building = self.get_building_for_room(to_room);

                    if let (Some(from_bldg), Some(to_bldg)) = (from_building, to_building) {
                        time_manager.calculate_travel_time(
                            from_room,
                            to_room,
                            from_bldg.id,
                            to_bldg.id,
                            from_loc.id,
                            to_loc.id,
                            rng,
                        )
                    } else {
                        // Fallback to default cross-location time
                        let hours = rng.gen_range(4..=12);
                        Duration::hours(hours)
                    }
                }
            } else {
                // Entering from outside
                to_loc.calculate_travel_time(from_room, to_room, time_manager, rng)
            }
        } else {
            // Room not found
            Duration::minutes(5)
        }
    }

    /// Get all rooms that require lobby access across all facilities
    pub fn get_all_rooms_requiring_lobby_access(&self) -> Vec<&Room> {
        self.locations
            .iter()
            .flat_map(|location| location.buildings.iter())
            .flat_map(|building| building.get_rooms_requiring_lobby_access())
            .collect()
    }

    /// Get all high-security rooms across all facilities
    pub fn get_all_high_security_rooms(&self) -> Vec<&Room> {
        self.locations
            .iter()
            .flat_map(|location| location.buildings.iter())
            .flat_map(|building| building.get_high_security_rooms())
            .collect()
    }

    /// Get statistics about access complexity across the facility system
    pub fn get_access_complexity_stats(&self) -> AccessComplexityStats {
        let mut total_rooms = 0;
        let mut rooms_requiring_lobby = 0;
        let mut high_security_rooms = 0;
        let mut rooms_with_intermediate_access = 0;

        for location in &self.locations {
            for building in &location.buildings {
                for room in &building.rooms {
                    total_rooms += 1;

                    if !room.is_lobby() {
                        rooms_requiring_lobby += 1;
                    }

                    if room.is_high_security() {
                        high_security_rooms += 1;
                    }

                    if room.requires_intermediate_access() {
                        rooms_with_intermediate_access += 1;
                    }
                }
            }
        }

        AccessComplexityStats {
            total_rooms,
            rooms_requiring_lobby,
            high_security_rooms,
            rooms_with_intermediate_access,
            locations: self.locations.len(),
            buildings: self.locations.iter().map(|l| l.buildings.len()).sum(),
        }
    }

    /// Check if a room exists in the registry
    pub fn room_exists(&self, room_id: RoomId) -> bool {
        self.room_index.contains_key(&room_id)
    }

    /// Check if a building exists in the registry
    pub fn building_exists(&self, building_id: BuildingId) -> bool {
        self.building_index.contains_key(&building_id)
    }

    /// Check if a location exists in the registry
    pub fn location_exists(&self, location_id: LocationId) -> bool {
        self.location_index.contains_key(&location_id)
    }

    /// Check if a user can access a specific room
    pub fn can_user_access_room(
        &self,
        user: &crate::user::User,
        room_id: RoomId,
    ) -> bool {
        if let Some(_room) = self.get_room(room_id) {
            if let Some(building) = self.get_building_for_room(room_id) {
                if let Some(location) = self.get_location_for_building(building.id) {
                    return user.can_access_room(room_id, building.id, location.id);
                }
            }
        }
        false
    }
}

impl Default for LocationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about access complexity in the facility system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessComplexityStats {
    /// Total number of rooms across all facilities
    pub total_rooms: usize,
    /// Number of rooms that require lobby access
    pub rooms_requiring_lobby: usize,
    /// Number of high-security rooms
    pub high_security_rooms: usize,
    /// Number of rooms with intermediate access requirements
    pub rooms_with_intermediate_access: usize,
    /// Total number of locations
    pub locations: usize,
    /// Total number of buildings
    pub buildings: usize,
}

impl AccessComplexityStats {
    /// Calculate the percentage of rooms requiring lobby access
    pub fn lobby_access_percentage(&self) -> f64 {
        if self.total_rooms == 0 {
            0.0
        } else {
            (self.rooms_requiring_lobby as f64 / self.total_rooms as f64) * 100.0
        }
    }

    /// Calculate the percentage of high-security rooms
    pub fn high_security_percentage(&self) -> f64 {
        if self.total_rooms == 0 {
            0.0
        } else {
            (self.high_security_rooms as f64 / self.total_rooms as f64) * 100.0
        }
    }

    /// Calculate the percentage of rooms with intermediate access requirements
    pub fn intermediate_access_percentage(&self) -> f64 {
        if self.total_rooms == 0 {
            0.0
        } else {
            (self.rooms_with_intermediate_access as f64 / self.total_rooms as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facility::room::Room;
    use crate::types::{RoomType, SecurityLevel};

    #[test]
    fn test_registry_creation() {
        let registry = LocationRegistry::new();
        assert_eq!(registry.location_count(), 0);
        assert_eq!(registry.total_building_count(), 0);
        assert_eq!(registry.total_room_count(), 0);
    }

    #[test]
    fn test_location_management() {
        let mut registry = LocationRegistry::new();

        // Create a location with buildings and rooms
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));
        let mut building = Building::new(location.id, "Test Building".to_string());
        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        let room = Room::new(
            building.id,
            "Workspace".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room_id = room.id;
        building.add_room(room);
        let building_id = building.id;
        location.add_building(building);
        let location_id = location.id;
        registry.add_location(location);

        // Test lookups
        assert_eq!(registry.location_count(), 1);
        assert_eq!(registry.total_building_count(), 1);
        assert_eq!(registry.total_room_count(), 2);

        assert!(registry.location_exists(location_id));
        assert!(registry.building_exists(building_id));
        assert!(registry.room_exists(room_id));

        // Test getting entities by ID
        let retrieved_location = registry.get_location(location_id);
        assert!(retrieved_location.is_some());
        assert_eq!(retrieved_location.unwrap().name, "Test Location");

        let retrieved_building = registry.get_building(building_id);
        assert!(retrieved_building.is_some());
        assert_eq!(retrieved_building.unwrap().name, "Test Building");

        let retrieved_room = registry.get_room(room_id);
        assert!(retrieved_room.is_some());
        assert_eq!(retrieved_room.unwrap().name, "Workspace");
    }

    #[test]
    fn test_cross_reference_lookups() {
        let mut registry = LocationRegistry::new();

        // Create a location with buildings and rooms
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));
        let mut building = Building::new(location.id, "Test Building".to_string());
        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        let room = Room::new(
            building.id,
            "Workspace".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room_id = room.id;
        building.add_room(room);
        let building_id = building.id;
        location.add_building(building);
        let location_id = location.id;
        registry.add_location(location);

        // Test cross-reference lookups
        let location_for_building = registry.get_location_for_building(building_id);
        assert!(location_for_building.is_some());
        assert_eq!(location_for_building.unwrap().id, location_id);

        let building_for_room = registry.get_building_for_room(room_id);
        assert!(building_for_room.is_some());
        assert_eq!(building_for_room.unwrap().id, building_id);

        let location_for_room = registry.get_location_for_room(room_id);
        assert!(location_for_room.is_some());
        assert_eq!(location_for_room.unwrap().id, location_id);
    }

    #[test]
    fn test_registry_validation() {
        let mut registry = LocationRegistry::new();

        // Empty registry should fail validation
        assert!(registry.validate().is_err());

        // Add a valid location
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));
        let mut building = Building::new(location.id, "Test Building".to_string());
        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        location.add_building(building);
        registry.add_location(location);

        // Now validation should pass
        assert!(registry.validate().is_ok());
    }

    #[test]
    fn test_access_complexity_stats() {
        let mut registry = LocationRegistry::new();

        // Create a location with various room types
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));
        let mut building = Building::new(location.id, "Test Building".to_string());

        // Add lobby (public, no intermediate access)
        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));

        // Add standard workspace (requires lobby access)
        building.add_room(Room::new(
            building.id,
            "Workspace".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));

        // Add high-security room (requires lobby access, high security)
        building.add_room(Room::new(
            building.id,
            "Server Room".to_string(),
            RoomType::ServerRoom,
            SecurityLevel::HighSecurity,
        ));

        // Add room with intermediate access
        let mut secure_room = Room::new(
            building.id,
            "Secure Lab".to_string(),
            RoomType::Laboratory,
            SecurityLevel::MaxSecurity,
        );
        secure_room.add_intermediate_access(RoomId::new()); // Add dummy intermediate access
        building.add_room(secure_room);

        location.add_building(building);
        registry.add_location(location);

        let stats = registry.get_access_complexity_stats();
        assert_eq!(stats.total_rooms, 4);
        assert_eq!(stats.rooms_requiring_lobby, 3); // All except lobby
        assert_eq!(stats.high_security_rooms, 2); // Server room and lab
        assert_eq!(stats.rooms_with_intermediate_access, 1); // Just the lab
        assert_eq!(stats.locations, 1);
        assert_eq!(stats.buildings, 1);

        // Test percentage calculations
        assert_eq!(stats.lobby_access_percentage(), 75.0); // 3/4 * 100
        assert_eq!(stats.high_security_percentage(), 50.0); // 2/4 * 100
        assert_eq!(stats.intermediate_access_percentage(), 25.0); // 1/4 * 100
    }

    #[test]
    fn test_high_security_rooms_collection() {
        let mut registry = LocationRegistry::new();

        // Create locations with various security levels
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));
        let mut building = Building::new(location.id, "Test Building".to_string());

        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        building.add_room(Room::new(
            building.id,
            "Workspace".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        building.add_room(Room::new(
            building.id,
            "Server Room".to_string(),
            RoomType::ServerRoom,
            SecurityLevel::HighSecurity,
        ));
        building.add_room(Room::new(
            building.id,
            "Lab".to_string(),
            RoomType::Laboratory,
            SecurityLevel::MaxSecurity,
        ));

        location.add_building(building);
        registry.add_location(location);

        let high_security_rooms = registry.get_all_high_security_rooms();
        assert_eq!(high_security_rooms.len(), 2); // Server room and lab

        let lobby_access_rooms = registry.get_all_rooms_requiring_lobby_access();
        assert_eq!(lobby_access_rooms.len(), 3); // All except lobby
    }

    #[test]
    fn test_rebuild_indices() {
        let mut registry = LocationRegistry::new();

        // Create a location
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));
        let mut building = Building::new(location.id, "Test Building".to_string());
        let room = Room::new(
            building.id,
            "Test Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room_id = room.id;
        building.add_room(room);
        location.add_building(building);
        registry.add_location(location);

        // Verify room exists
        assert!(registry.room_exists(room_id));

        // Modify locations directly (bypassing add_location)
        registry.locations.clear();

        // Room should still exist in index
        assert!(registry.room_exists(room_id));

        // Rebuild indices
        registry.rebuild_indices();

        // Now room should not exist
        assert!(!registry.room_exists(room_id));
    }
}
