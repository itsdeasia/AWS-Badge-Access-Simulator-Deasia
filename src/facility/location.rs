//! Location management and building organization
//!
//! This module contains the Location struct and related functionality for managing
//! geographical locations containing multiple buildings, including cross-building
//! access flows and distance calculations.

use crate::facility::{building::Building, room::Room};
use crate::permissions::access_flow::AccessFlow;
use crate::simulation::time_manager::TimeManager;
use crate::types::{BuildingId, LocationId, RoomId, RoomType, SecurityLevel};
use chrono::Duration;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Represents a geographical location containing multiple buildings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Unique identifier for the location
    pub id: LocationId,
    /// Human-readable name of the location
    pub name: String,
    /// Collection of buildings within this location
    pub buildings: Vec<Building>,
    /// Geographical coordinates (latitude, longitude) for impossible traveler calculations
    pub coordinates: (f64, f64),
}

impl Location {
    /// Create a new location
    pub fn new(name: String, coordinates: (f64, f64)) -> Self {
        Self { id: LocationId::new(), name, buildings: Vec::new(), coordinates }
    }

    /// Add a building to the location
    pub fn add_building(&mut self, mut building: Building) {
        // Ensure the building belongs to this location
        building.location_id = self.id;
        self.buildings.push(building);
    }

    /// Remove a building from the location
    pub fn remove_building(&mut self, building_id: BuildingId) -> Option<Building> {
        if let Some(pos) = self.buildings.iter().position(|b| b.id == building_id) {
            Some(self.buildings.remove(pos))
        } else {
            None
        }
    }

    /// Get a building by ID
    pub fn get_building(&self, building_id: BuildingId) -> Option<&Building> {
        self.buildings.iter().find(|b| b.id == building_id)
    }

    /// Get a mutable reference to a building by ID
    pub fn get_building_mut(&mut self, building_id: BuildingId) -> Option<&mut Building> {
        self.buildings.iter_mut().find(|b| b.id == building_id)
    }

    /// Get a room by ID (searches all buildings)
    pub fn get_room(&self, room_id: RoomId) -> Option<&Room> {
        for building in &self.buildings {
            if let Some(room) = building.get_room(room_id) {
                return Some(room);
            }
        }
        None
    }

    /// Get a mutable reference to a room by ID (searches all buildings)
    pub fn get_room_mut(&mut self, room_id: RoomId) -> Option<&mut Room> {
        for building in &mut self.buildings {
            if let Some(room) = building.get_room_mut(room_id) {
                return Some(room);
            }
        }
        None
    }

    /// Find which building contains a specific room
    pub fn find_building_for_room(&self, room_id: RoomId) -> Option<&Building> {
        self.buildings.iter().find(|b| b.contains_room(room_id))
    }

    /// Get all rooms of a specific type across all buildings
    pub fn get_rooms_by_type(&self, room_type: RoomType) -> Vec<&Room> {
        self.buildings.iter().flat_map(|b| b.get_rooms_by_type(room_type)).collect()
    }

    /// Get all rooms with a specific security level across all buildings
    pub fn get_rooms_by_security_level(&self, security_level: SecurityLevel) -> Vec<&Room> {
        self.buildings.iter().flat_map(|b| b.get_rooms_by_security_level(security_level)).collect()
    }

    /// Get the number of buildings in the location
    pub fn building_count(&self) -> usize {
        self.buildings.len()
    }

    /// Get the total number of rooms across all buildings
    pub fn total_room_count(&self) -> usize {
        self.buildings.iter().map(|b| b.room_count()).sum()
    }

    /// Validate that the location has valid buildings and rooms
    pub fn validate(&self) -> Result<(), String> {
        if self.buildings.is_empty() {
            return Err("Location must have at least one building".to_string());
        }

        // Validate each building
        for building in &self.buildings {
            if let Err(e) = building.validate() {
                return Err(format!("Building {} validation failed: {}", building.id, e));
            }

            // Ensure building belongs to this location
            if building.location_id != self.id {
                return Err(format!(
                    "Building {} does not belong to location {}",
                    building.id, self.id
                ));
            }
        }

        Ok(())
    }

    /// Get all building IDs in the location
    pub fn get_all_building_ids(&self) -> Vec<BuildingId> {
        self.buildings.iter().map(|b| b.id).collect()
    }

    /// Get all room IDs across all buildings in the location
    pub fn get_all_room_ids(&self) -> Vec<RoomId> {
        self.buildings.iter().flat_map(|b| b.get_all_room_ids()).collect()
    }

    /// Get the access flow required to reach a target room from a starting room
    ///
    /// This method handles cross-building access flows within the same location,
    /// including the need to exit one building and enter another through its lobby.
    ///
    /// # Arguments
    /// * `from_room` - Starting room (None if starting from outside all buildings)
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
        let target_building = self
            .find_building_for_room(to_room)
            .ok_or_else(|| format!("Target room {} not found in location {}", to_room, self.id))?;

        // Check if we're staying in the same building
        if let Some(from_room_id) = from_room {
            if let Some(from_building) = self.find_building_for_room(from_room_id) {
                if from_building.id == target_building.id {
                    // Same building - delegate to building's access flow
                    return target_building.get_access_flow(from_room, to_room, time_manager, rng);
                }
            }
        }

        // Cross-building access or entering from outside
        // Must go through target building's lobby
        let target_building_flow =
            target_building.get_access_flow(None, to_room, time_manager, rng)?;

        // Calculate additional travel time between buildings
        let inter_building_travel_time = if from_room.is_some() {
            // Walking between buildings in same location (2-10 minutes)
            let minutes = rng.gen_range(2..=10);
            Duration::minutes(minutes)
        } else {
            // Arriving at location from outside (5-15 minutes to reach building)
            let minutes = rng.gen_range(5..=15);
            Duration::minutes(minutes)
        };

        let total_travel_time =
            target_building_flow.estimated_travel_time + inter_building_travel_time;

        Ok(AccessFlow::new(
            target_building_flow.required_sequence,
            total_travel_time,
            target_building_flow.requires_lobby_access,
            target_building_flow.involves_high_security,
        ))
    }

    /// Calculate travel time between two rooms in this location
    ///
    /// # Arguments
    /// * `from_room` - Starting room (None if starting from outside)
    /// * `to_room` - Destination room
    /// * `time_manager` - Time manager for calculations
    /// * `rng` - Random number generator
    pub fn calculate_travel_time<R: Rng>(
        &self,
        from_room: Option<RoomId>,
        to_room: RoomId,
        time_manager: &TimeManager,
        rng: &mut R,
    ) -> Duration {
        let from_building = from_room.and_then(|room_id| self.find_building_for_room(room_id));
        let to_building = self.find_building_for_room(to_room);

        if let Some(to_bldg) = to_building {
            if let Some(from_bldg) = from_building {
                // Calculate travel time using time manager
                time_manager.calculate_travel_time(
                    from_room,
                    to_room,
                    from_bldg.id,
                    to_bldg.id,
                    self.id,
                    self.id,
                    rng,
                )
            } else {
                // Entering location from outside
                let minutes = rng.gen_range(5..=15);
                Duration::minutes(minutes)
            }
        } else {
            // Room not found, return default
            Duration::minutes(5)
        }
    }

    /// Check if two rooms are in the same building
    pub fn are_rooms_in_same_building(&self, room1: RoomId, room2: RoomId) -> bool {
        if let (Some(building1), Some(building2)) =
            (self.find_building_for_room(room1), self.find_building_for_room(room2))
        {
            building1.id == building2.id
        } else {
            false
        }
    }

    /// Get all buildings that contain high-security rooms
    pub fn get_buildings_with_high_security(&self) -> Vec<&Building> {
        self.buildings
            .iter()
            .filter(|building| !building.get_high_security_rooms().is_empty())
            .collect()
    }

    /// Get all rooms across all buildings that require intermediate access
    pub fn get_rooms_requiring_intermediate_access(&self) -> Vec<&Room> {
        self.buildings
            .iter()
            .flat_map(|building| building.rooms.iter())
            .filter(|room| room.requires_intermediate_access())
            .collect()
    }

    /// Check if a building exists in this location
    pub fn contains_building(&self, building_id: BuildingId) -> bool {
        self.buildings.iter().any(|b| b.id == building_id)
    }

    /// Check if a room exists in this location (searches all buildings)
    pub fn contains_room(&self, room_id: RoomId) -> bool {
        self.buildings.iter().any(|b| b.contains_room(room_id))
    }

    /// Calculate distance to another location in kilometers (using Haversine formula)
    pub fn distance_to(&self, other: &Location) -> f64 {
        let (lat1, lon1) = self.coordinates;
        let (lat2, lon2) = other.coordinates;

        let r = 6371.0; // Earth's radius in kilometers
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facility::room::Room;
    use crate::types::RoomType;

    #[test]
    fn test_location_creation() {
        let coordinates = (47.6062, -122.3321); // Seattle coordinates
        let location = Location::new("Seattle Office".to_string(), coordinates);

        assert_eq!(location.name, "Seattle Office");
        assert_eq!(location.coordinates, coordinates);
        assert!(location.buildings.is_empty());
    }

    #[test]
    fn test_building_management() {
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));

        // Create and add a building
        let mut building = Building::new(location.id, "Test Building".to_string());
        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        let building_id = building.id;
        location.add_building(building);

        assert_eq!(location.building_count(), 1);
        assert!(location.contains_building(building_id));

        // Get building by ID
        let retrieved_building = location.get_building(building_id);
        assert!(retrieved_building.is_some());
        assert_eq!(retrieved_building.unwrap().name, "Test Building");

        // Remove building
        let removed_building = location.remove_building(building_id);
        assert!(removed_building.is_some());
        assert_eq!(location.building_count(), 0);
        assert!(!location.contains_building(building_id));
    }

    #[test]
    fn test_room_search_across_buildings() {
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));

        // Create first building with rooms
        let mut building1 = Building::new(location.id, "Building 1".to_string());
        building1.add_room(Room::new(
            building1.id,
            "Lobby 1".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        let workspace1 = Room::new(
            building1.id,
            "Workspace 1".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let workspace1_id = workspace1.id;
        building1.add_room(workspace1);
        location.add_building(building1);

        // Create second building with rooms
        let mut building2 = Building::new(location.id, "Building 2".to_string());
        building2.add_room(Room::new(
            building2.id,
            "Lobby 2".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        let workspace2 = Room::new(
            building2.id,
            "Workspace 2".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let workspace2_id = workspace2.id;
        building2.add_room(workspace2);
        location.add_building(building2);

        // Test room search across buildings
        assert!(location.contains_room(workspace1_id));
        assert!(location.contains_room(workspace2_id));

        let room1 = location.get_room(workspace1_id);
        assert!(room1.is_some());
        assert_eq!(room1.unwrap().name, "Workspace 1");

        let room2 = location.get_room(workspace2_id);
        assert!(room2.is_some());
        assert_eq!(room2.unwrap().name, "Workspace 2");

        // Test finding building for room
        let building_for_room1 = location.find_building_for_room(workspace1_id);
        assert!(building_for_room1.is_some());
        assert_eq!(building_for_room1.unwrap().name, "Building 1");

        let building_for_room2 = location.find_building_for_room(workspace2_id);
        assert!(building_for_room2.is_some());
        assert_eq!(building_for_room2.unwrap().name, "Building 2");
    }

    #[test]
    fn test_rooms_by_type_across_buildings() {
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));

        // Create buildings with different room types
        let mut building1 = Building::new(location.id, "Building 1".to_string());
        building1.add_room(Room::new(
            building1.id,
            "Lobby 1".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        building1.add_room(Room::new(
            building1.id,
            "Workspace 1".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        location.add_building(building1);

        let mut building2 = Building::new(location.id, "Building 2".to_string());
        building2.add_room(Room::new(
            building2.id,
            "Lobby 2".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        building2.add_room(Room::new(
            building2.id,
            "Workspace 2".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        building2.add_room(Room::new(
            building2.id,
            "Meeting Room".to_string(),
            RoomType::MeetingRoom,
            SecurityLevel::Standard,
        ));
        location.add_building(building2);

        // Test filtering by type across all buildings
        let lobbies = location.get_rooms_by_type(RoomType::Lobby);
        assert_eq!(lobbies.len(), 2);

        let workspaces = location.get_rooms_by_type(RoomType::Workspace);
        assert_eq!(workspaces.len(), 2);

        let meeting_rooms = location.get_rooms_by_type(RoomType::MeetingRoom);
        assert_eq!(meeting_rooms.len(), 1);
    }

    #[test]
    fn test_location_validation() {
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));

        // Location without buildings should fail validation
        assert!(location.validate().is_err());

        // Add a building without lobby (should fail)
        let mut building = Building::new(location.id, "Test Building".to_string());
        building.add_room(Room::new(
            building.id,
            "Workspace".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        location.add_building(building);

        assert!(location.validate().is_err());

        // Add lobby to make building valid
        let building = location.get_building_mut(location.buildings[0].id).unwrap();
        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));

        // Now validation should pass
        assert!(location.validate().is_ok());
    }

    #[test]
    fn test_distance_calculation() {
        let seattle = Location::new("Seattle".to_string(), (47.6062, -122.3321));
        let portland = Location::new("Portland".to_string(), (45.5152, -122.6784));

        let distance = seattle.distance_to(&portland);

        // Distance between Seattle and Portland is approximately 233 km
        assert!(distance > 200.0 && distance < 300.0);
    }

    #[test]
    fn test_same_building_check() {
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));

        // Create building with multiple rooms
        let mut building = Building::new(location.id, "Test Building".to_string());
        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        let room1 = Room::new(
            building.id,
            "Room 1".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room1_id = room1.id;
        building.add_room(room1);
        let room2 = Room::new(
            building.id,
            "Room 2".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room2_id = room2.id;
        building.add_room(room2);
        location.add_building(building);

        // Create second building
        let mut building2 = Building::new(location.id, "Building 2".to_string());
        building2.add_room(Room::new(
            building2.id,
            "Lobby 2".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        let room3 = Room::new(
            building2.id,
            "Room 3".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let room3_id = room3.id;
        building2.add_room(room3);
        location.add_building(building2);

        // Test same building check
        assert!(location.are_rooms_in_same_building(room1_id, room2_id));
        assert!(!location.are_rooms_in_same_building(room1_id, room3_id));
        assert!(!location.are_rooms_in_same_building(room2_id, room3_id));
    }

    #[test]
    fn test_high_security_buildings() {
        let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));

        // Building with high security rooms
        let mut building1 = Building::new(location.id, "Secure Building".to_string());
        building1.add_room(Room::new(
            building1.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        building1.add_room(Room::new(
            building1.id,
            "Server Room".to_string(),
            RoomType::ServerRoom,
            SecurityLevel::HighSecurity,
        ));
        location.add_building(building1);

        // Building without high security rooms
        let mut building2 = Building::new(location.id, "Regular Building".to_string());
        building2.add_room(Room::new(
            building2.id,
            "Lobby 2".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        building2.add_room(Room::new(
            building2.id,
            "Workspace".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        location.add_building(building2);

        let high_security_buildings = location.get_buildings_with_high_security();
        assert_eq!(high_security_buildings.len(), 1);
        assert_eq!(high_security_buildings[0].name, "Secure Building");
    }
}
