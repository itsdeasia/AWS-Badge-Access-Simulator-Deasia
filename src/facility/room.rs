//! Room management and access control
//!
//! This module contains the Room struct and related functionality for managing
//! individual rooms within buildings, including access requirements and validation.

use crate::types::{BuildingId, RoomId, RoomType, SecurityLevel};
use serde::{Deserialize, Serialize};

/// Represents a room within a building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// Unique identifier for the room
    pub id: RoomId,
    /// ID of the building this room belongs to
    pub building_id: BuildingId,
    /// Human-readable name of the room
    pub name: String,
    /// Type of room (lobby, workspace, meeting room, etc.)
    pub room_type: RoomType,
    /// Security level required to access this room
    pub security_level: SecurityLevel,
    /// Rooms that must be accessed before this room (for sequential access)
    pub required_intermediate_access: Vec<RoomId>,
}

impl Room {
    /// Create a new room
    pub fn new(
        building_id: BuildingId,
        name: String,
        room_type: RoomType,
        security_level: SecurityLevel,
    ) -> Self {
        Self {
            id: RoomId::new(),
            building_id,
            name,
            room_type,
            security_level,
            required_intermediate_access: Vec::new(),
        }
    }

    /// Create a new room with intermediate access requirements
    pub fn new_with_intermediate_access(
        building_id: BuildingId,
        name: String,
        room_type: RoomType,
        security_level: SecurityLevel,
        required_intermediate_access: Vec<RoomId>,
    ) -> Self {
        Self {
            id: RoomId::new(),
            building_id,
            name,
            room_type,
            security_level,
            required_intermediate_access,
        }
    }

    /// Check if this room is a lobby
    pub fn is_lobby(&self) -> bool {
        self.room_type == RoomType::Lobby
    }

    /// Check if this room requires intermediate access
    pub fn requires_intermediate_access(&self) -> bool {
        !self.required_intermediate_access.is_empty()
    }

    /// Get the required intermediate access rooms
    pub fn get_intermediate_access_rooms(&self) -> &[RoomId] {
        &self.required_intermediate_access
    }

    /// Add an intermediate access requirement
    pub fn add_intermediate_access(&mut self, room_id: RoomId) {
        if !self.required_intermediate_access.contains(&room_id) {
            self.required_intermediate_access.push(room_id);
        }
    }

    /// Remove an intermediate access requirement
    pub fn remove_intermediate_access(&mut self, room_id: RoomId) {
        self.required_intermediate_access.retain(|&id| id != room_id);
    }

    /// Check if this room has high security requirements
    pub fn is_high_security(&self) -> bool {
        matches!(self.security_level, SecurityLevel::HighSecurity | SecurityLevel::MaxSecurity)
    }

    /// Check if this room is publicly accessible
    pub fn is_public(&self) -> bool {
        self.security_level == SecurityLevel::Public
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_creation() {
        let building_id = BuildingId::new();
        let room = Room::new(
            building_id,
            "Test Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );

        assert_eq!(room.building_id, building_id);
        assert_eq!(room.name, "Test Room");
        assert_eq!(room.room_type, RoomType::Workspace);
        assert_eq!(room.security_level, SecurityLevel::Standard);
        assert!(room.required_intermediate_access.is_empty());
    }

    #[test]
    fn test_room_with_intermediate_access() {
        let building_id = BuildingId::new();
        let intermediate_room = RoomId::new();
        let room = Room::new_with_intermediate_access(
            building_id,
            "Secure Room".to_string(),
            RoomType::ServerRoom,
            SecurityLevel::HighSecurity,
            vec![intermediate_room],
        );

        assert!(room.requires_intermediate_access());
        assert_eq!(room.get_intermediate_access_rooms(), &[intermediate_room]);
    }

    #[test]
    fn test_lobby_identification() {
        let building_id = BuildingId::new();
        let lobby = Room::new(
            building_id,
            "Main Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        );

        assert!(lobby.is_lobby());
        assert!(lobby.is_public());
        assert!(!lobby.is_high_security());
    }

    #[test]
    fn test_high_security_room() {
        let building_id = BuildingId::new();
        let server_room = Room::new(
            building_id,
            "Server Room".to_string(),
            RoomType::ServerRoom,
            SecurityLevel::HighSecurity,
        );

        assert!(server_room.is_high_security());
        assert!(!server_room.is_public());
        assert!(!server_room.is_lobby());
    }

    #[test]
    fn test_intermediate_access_management() {
        let building_id = BuildingId::new();
        let mut room = Room::new(
            building_id,
            "Test Room".to_string(),
            RoomType::Laboratory,
            SecurityLevel::MaxSecurity,
        );

        let checkpoint1 = RoomId::new();
        let checkpoint2 = RoomId::new();

        // Initially no intermediate access required
        assert!(!room.requires_intermediate_access());

        // Add intermediate access requirements
        room.add_intermediate_access(checkpoint1);
        room.add_intermediate_access(checkpoint2);

        assert!(room.requires_intermediate_access());
        assert_eq!(room.get_intermediate_access_rooms().len(), 2);
        assert!(room.get_intermediate_access_rooms().contains(&checkpoint1));
        assert!(room.get_intermediate_access_rooms().contains(&checkpoint2));

        // Adding the same room again should not duplicate
        room.add_intermediate_access(checkpoint1);
        assert_eq!(room.get_intermediate_access_rooms().len(), 2);

        // Remove intermediate access requirement
        room.remove_intermediate_access(checkpoint1);
        assert_eq!(room.get_intermediate_access_rooms().len(), 1);
        assert!(!room.get_intermediate_access_rooms().contains(&checkpoint1));
        assert!(room.get_intermediate_access_rooms().contains(&checkpoint2));

        // Remove last requirement
        room.remove_intermediate_access(checkpoint2);
        assert!(!room.requires_intermediate_access());
        assert!(room.get_intermediate_access_rooms().is_empty());
    }

    #[test]
    fn test_security_level_checks() {
        let building_id = BuildingId::new();

        let public_room = Room::new(
            building_id,
            "Public Room".to_string(),
            RoomType::Cafeteria,
            SecurityLevel::Public,
        );
        assert!(public_room.is_public());
        assert!(!public_room.is_high_security());

        let standard_room = Room::new(
            building_id,
            "Standard Room".to_string(),
            RoomType::Workspace,
            SecurityLevel::Standard,
        );
        assert!(!standard_room.is_public());
        assert!(!standard_room.is_high_security());

        let restricted_room = Room::new(
            building_id,
            "Restricted Room".to_string(),
            RoomType::ExecutiveOffice,
            SecurityLevel::Restricted,
        );
        assert!(!restricted_room.is_public());
        assert!(!restricted_room.is_high_security());

        let high_security_room = Room::new(
            building_id,
            "High Security Room".to_string(),
            RoomType::ServerRoom,
            SecurityLevel::HighSecurity,
        );
        assert!(!high_security_room.is_public());
        assert!(high_security_room.is_high_security());

        let max_security_room = Room::new(
            building_id,
            "Max Security Room".to_string(),
            RoomType::Laboratory,
            SecurityLevel::MaxSecurity,
        );
        assert!(!max_security_room.is_public());
        assert!(max_security_room.is_high_security());
    }
}
