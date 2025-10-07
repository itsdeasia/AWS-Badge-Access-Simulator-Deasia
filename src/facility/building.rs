//! Building management and room organization
//!
//! This module contains the Building struct and related functionality for managing
//! buildings within locations, including room management and access flow calculations.

use crate::facility::room::Room;
use crate::permissions::access_flow::AccessFlow;
use crate::simulation::time_manager::TimeManager;
use crate::types::{BuildingId, LocationId, RoomId, RoomType, SecurityLevel};
use chrono::Duration;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Represents a building within a location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    /// Unique identifier for the building
    pub id: BuildingId,
    /// ID of the location this building belongs to
    pub location_id: LocationId,
    /// Human-readable name of the building
    pub name: String,
    /// Collection of rooms within this building
    pub rooms: Vec<Room>,
    /// ID of the lobby room (required for building access)
    pub lobby_room_id: Option<RoomId>,
}

impl Building {
    /// Create a new building
    pub fn new(location_id: LocationId, name: String) -> Self {
        Self { id: BuildingId::new(), location_id, name, rooms: Vec::new(), lobby_room_id: None }
    }

    /// Add a room to the building
    pub fn add_room(&mut self, mut room: Room) {
        // Ensure the room belongs to this building
        room.building_id = self.id;

        // If this is a lobby room, set it as the building's lobby
        if room.is_lobby() {
            self.lobby_room_id = Some(room.id);
        }

        self.rooms.push(room);
    }

    /// Remove a room from the building
    pub fn remove_room(&mut self, room_id: RoomId) -> Option<Room> {
        if let Some(pos) = self.rooms.iter().position(|r| r.id == room_id) {
            let room = self.rooms.remove(pos);

            // If we removed the lobby, clear the lobby_room_id
            if Some(room_id) == self.lobby_room_id {
                self.lobby_room_id = None;
            }

            Some(room)
        } else {
            None
        }
    }

    /// Get a room by ID
    pub fn get_room(&self, room_id: RoomId) -> Option<&Room> {
        self.rooms.iter().find(|r| r.id == room_id)
    }

    /// Get a mutable reference to a room by ID
    pub fn get_room_mut(&mut self, room_id: RoomId) -> Option<&mut Room> {
        self.rooms.iter_mut().find(|r| r.id == room_id)
    }

    /// Get the lobby room
    pub fn get_lobby_room(&self) -> Option<&Room> {
        self.lobby_room_id.and_then(|id| self.get_room(id))
    }

    /// Get all rooms of a specific type
    pub fn get_rooms_by_type(&self, room_type: RoomType) -> Vec<&Room> {
        self.rooms.iter().filter(|r| r.room_type == room_type).collect()
    }

    /// Get all rooms with a specific security level
    pub fn get_rooms_by_security_level(&self, security_level: SecurityLevel) -> Vec<&Room> {
        self.rooms.iter().filter(|r| r.security_level == security_level).collect()
    }

    /// Check if the building has a lobby
    pub fn has_lobby(&self) -> bool {
        self.lobby_room_id.is_some()
    }

    /// Get the number of rooms in the building
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Validate that the building has required rooms (at least one lobby)
    pub fn validate(&self) -> Result<(), String> {
        if !self.has_lobby() {
            return Err("Building must have at least one lobby room".to_string());
        }

        // Validate that all rooms belong to this building
        for room in &self.rooms {
            if room.building_id != self.id {
                return Err(format!("Room {} does not belong to building {}", room.id, self.id));
            }
        }

        Ok(())
    }

    /// Get all room IDs in the building
    pub fn get_all_room_ids(&self) -> Vec<RoomId> {
        self.rooms.iter().map(|r| r.id).collect()
    }

    /// Check if a room exists in this building
    pub fn contains_room(&self, room_id: RoomId) -> bool {
        self.rooms.iter().any(|r| r.id == room_id)
    }

    /// Get the access flow required to reach a target room from a starting room
    ///
    /// This method calculates the sequence of rooms that must be accessed to reach
    /// the target room, including lobby access and any intermediate security checkpoints.
    ///
    /// # Arguments
    /// * `from_room` - Starting room (None if entering from outside the building)
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
        let target_room = self
            .get_room(to_room)
            .ok_or_else(|| format!("Target room {} not found in building {}", to_room, self.id))?;

        let mut sequence = Vec::new();
        let mut requires_lobby_access = false;
        let mut involves_high_security = target_room.is_high_security();

        // Check if we need lobby access
        if from_room.is_none() || !self.is_same_building_room(from_room.unwrap()) {
            // Entering from outside or from a different building - must go through lobby
            if let Some(lobby_id) = self.lobby_room_id {
                sequence.push(lobby_id);
                requires_lobby_access = true;
            } else {
                return Err(format!("Building {} has no lobby room", self.id));
            }
        }

        // Add any required intermediate access rooms
        for &intermediate_room_id in &target_room.required_intermediate_access {
            if !sequence.contains(&intermediate_room_id) {
                sequence.push(intermediate_room_id);

                // Check if intermediate room is high security
                if let Some(intermediate_room) = self.get_room(intermediate_room_id) {
                    if intermediate_room.is_high_security() {
                        involves_high_security = true;
                    }
                }
            }
        }

        // Add the target room
        sequence.push(to_room);

        // Calculate total travel time
        let estimated_travel_time =
            self.calculate_sequence_travel_time(from_room, &sequence, time_manager, rng);

        Ok(AccessFlow::new(
            sequence,
            estimated_travel_time,
            requires_lobby_access,
            involves_high_security,
        ))
    }

    /// Calculate the total travel time for a sequence of room accesses
    fn calculate_sequence_travel_time<R: Rng>(
        &self,
        from_room: Option<RoomId>,
        sequence: &[RoomId],
        _time_manager: &TimeManager,
        rng: &mut R,
    ) -> Duration {
        if sequence.is_empty() {
            return Duration::seconds(0);
        }

        let mut total_time = Duration::seconds(0);
        let mut current_room = from_room;

        for &next_room in sequence {
            // Calculate travel time to next room
            let travel_time = if let Some(current) = current_room {
                if current == next_room {
                    Duration::seconds(0) // Already in the room
                } else {
                    // Same building travel time (30 seconds to 3 minutes)
                    let seconds = rng.gen_range(30..=180);
                    Duration::seconds(seconds)
                }
            } else {
                // Entering building from outside (1-2 minutes to reach first room)
                let seconds = rng.gen_range(60..=120);
                Duration::seconds(seconds)
            };

            total_time += travel_time;

            // Add time for badge swipe and door opening (5-15 seconds)
            let badge_time = rng.gen_range(5..=15);
            total_time += Duration::seconds(badge_time);

            current_room = Some(next_room);
        }

        total_time
    }

    /// Check if a room belongs to this building
    fn is_same_building_room(&self, room_id: RoomId) -> bool {
        self.contains_room(room_id)
    }

    /// Get all rooms that require lobby access (i.e., all non-lobby rooms)
    pub fn get_rooms_requiring_lobby_access(&self) -> Vec<&Room> {
        self.rooms.iter().filter(|room| !room.is_lobby()).collect()
    }

    /// Get all high-security rooms in the building
    pub fn get_high_security_rooms(&self) -> Vec<&Room> {
        self.rooms.iter().filter(|room| room.is_high_security()).collect()
    }

    /// Check if accessing a room requires going through specific intermediate rooms
    pub fn requires_intermediate_access(&self, room_id: RoomId) -> bool {
        if let Some(room) = self.get_room(room_id) {
            room.requires_intermediate_access()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RoomType;

    #[test]
    fn test_building_creation() {
        let location_id = LocationId::new();
        let building = Building::new(location_id, "Test Building".to_string());

        assert_eq!(building.location_id, location_id);
        assert_eq!(building.name, "Test Building");
        assert!(building.rooms.is_empty());
        assert!(!building.has_lobby());
    }

    #[test]
    fn test_room_management() {
        let location_id = LocationId::new();
        let mut building = Building::new(location_id, "Test Building".to_string());

        // Add a lobby room
        let lobby = Room::new(
            building.id,
            "Main Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        );
        let lobby_id = lobby.id;
        building.add_room(lobby);

        assert!(building.has_lobby());
        assert_eq!(building.lobby_room_id, Some(lobby_id));
        assert_eq!(building.room_count(), 1);

        // Add a workspace
        let workspace = Room::new(
            building.id,
            "Workspace 1".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        let workspace_id = workspace.id;
        building.add_room(workspace);

        assert_eq!(building.room_count(), 2);
        assert!(building.contains_room(workspace_id));

        // Get room by ID
        let retrieved_room = building.get_room(workspace_id);
        assert!(retrieved_room.is_some());
        assert_eq!(retrieved_room.unwrap().name, "Workspace 1");

        // Remove room
        let removed_room = building.remove_room(workspace_id);
        assert!(removed_room.is_some());
        assert_eq!(building.room_count(), 1);
        assert!(!building.contains_room(workspace_id));
    }

    #[test]
    fn test_lobby_management() {
        let location_id = LocationId::new();
        let mut building = Building::new(location_id, "Test Building".to_string());

        // Add a lobby room
        let lobby = Room::new(
            building.id,
            "Main Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        );
        let lobby_id = lobby.id;
        building.add_room(lobby);

        // Check lobby access
        assert!(building.has_lobby());
        let lobby_room = building.get_lobby_room();
        assert!(lobby_room.is_some());
        assert_eq!(lobby_room.unwrap().id, lobby_id);

        // Remove lobby
        building.remove_room(lobby_id);
        assert!(!building.has_lobby());
        assert!(building.get_lobby_room().is_none());
    }

    #[test]
    fn test_rooms_by_type() {
        let location_id = LocationId::new();
        let mut building = Building::new(location_id, "Test Building".to_string());

        // Add various room types
        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));
        building.add_room(Room::new(
            building.id,
            "Workspace 1".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        building.add_room(Room::new(
            building.id,
            "Workspace 2".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        building.add_room(Room::new(
            building.id,
            "Meeting Room".to_string(),
            RoomType::MeetingRoom,
            SecurityLevel::Standard,
        ));

        // Test filtering by type
        let workspaces = building.get_rooms_by_type(RoomType::Workspace);
        assert_eq!(workspaces.len(), 2);

        let lobbies = building.get_rooms_by_type(RoomType::Lobby);
        assert_eq!(lobbies.len(), 1);

        let meeting_rooms = building.get_rooms_by_type(RoomType::MeetingRoom);
        assert_eq!(meeting_rooms.len(), 1);
    }

    #[test]
    fn test_rooms_by_security_level() {
        let location_id = LocationId::new();
        let mut building = Building::new(location_id, "Test Building".to_string());

        // Add rooms with different security levels
        building.add_room(Room::new(
            building.id,
            "Public Room".to_string(),
            RoomType::Cafeteria,
            SecurityLevel::Public,
        ));
        building.add_room(Room::new(
            building.id,
            "Standard Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        building.add_room(Room::new(
            building.id,
            "High Security Room".to_string(),
            RoomType::ServerRoom,
            SecurityLevel::HighSecurity,
        ));

        // Test filtering by security level
        let public_rooms = building.get_rooms_by_security_level(SecurityLevel::Public);
        assert_eq!(public_rooms.len(), 1);

        let standard_rooms = building.get_rooms_by_security_level(SecurityLevel::Standard);
        assert_eq!(standard_rooms.len(), 1);

        let high_security_rooms = building.get_rooms_by_security_level(SecurityLevel::HighSecurity);
        assert_eq!(high_security_rooms.len(), 1);
    }

    #[test]
    fn test_building_validation() {
        let location_id = LocationId::new();
        let mut building = Building::new(location_id, "Test Building".to_string());

        // Building without lobby should fail validation
        assert!(building.validate().is_err());

        // Add lobby
        building.add_room(Room::new(
            building.id,
            "Main Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));

        // Now validation should pass
        assert!(building.validate().is_ok());
    }

    #[test]
    fn test_high_security_rooms() {
        let location_id = LocationId::new();
        let mut building = Building::new(location_id, "Test Building".to_string());

        // Add lobby first
        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));

        // Add various security level rooms
        building.add_room(Room::new(
            building.id,
            "Standard Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        building.add_room(Room::new(
            building.id,
            "High Security Room".to_string(),
            RoomType::ServerRoom,
            SecurityLevel::HighSecurity,
        ));
        building.add_room(Room::new(
            building.id,
            "Max Security Room".to_string(),
            RoomType::Laboratory,
            SecurityLevel::MaxSecurity,
        ));

        let high_security_rooms = building.get_high_security_rooms();
        assert_eq!(high_security_rooms.len(), 2); // HighSecurity and MaxSecurity
    }

    #[test]
    fn test_rooms_requiring_lobby_access() {
        let location_id = LocationId::new();
        let mut building = Building::new(location_id, "Test Building".to_string());

        // Add lobby
        building.add_room(Room::new(
            building.id,
            "Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        ));

        // Add other rooms
        building.add_room(Room::new(
            building.id,
            "Workspace".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        ));
        building.add_room(Room::new(
            building.id,
            "Meeting Room".to_string(),
            RoomType::MeetingRoom,
            SecurityLevel::Standard,
        ));

        let rooms_requiring_lobby = building.get_rooms_requiring_lobby_access();
        assert_eq!(rooms_requiring_lobby.len(), 2); // All non-lobby rooms
    }
}
