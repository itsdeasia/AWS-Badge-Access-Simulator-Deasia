//! Unique identifier types for the badge access simulator
//!
//! This module contains UUID-based identifier types for users, locations,
//! buildings, and rooms used throughout the simulation system.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

impl UserId {
    /// Create a new random user ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "USER_{}", self.0.simple())
    }
}

impl Serialize for UserId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("USER_{}", self.0.simple()))
    }
}

impl<'de> Deserialize<'de> for UserId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some(uuid_str) = s.strip_prefix("USER_") {
            let uuid = Uuid::parse_str(uuid_str).map_err(serde::de::Error::custom)?;
            Ok(UserId(uuid))
        } else {
            // Fallback: try to parse as raw UUID for backward compatibility
            let uuid = Uuid::parse_str(&s).map_err(serde::de::Error::custom)?;
            Ok(UserId(uuid))
        }
    }
}

/// Unique identifier for a geographical location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocationId(pub Uuid);

impl LocationId {
    /// Create a new random location ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LocationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for LocationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LOC_{}", self.0.simple())
    }
}

impl Serialize for LocationId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("LOC_{}", self.0.simple()))
    }
}

impl<'de> Deserialize<'de> for LocationId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some(uuid_str) = s.strip_prefix("LOC_") {
            let uuid = Uuid::parse_str(uuid_str).map_err(serde::de::Error::custom)?;
            Ok(LocationId(uuid))
        } else {
            // Fallback: try to parse as raw UUID for backward compatibility
            let uuid = Uuid::parse_str(&s).map_err(serde::de::Error::custom)?;
            Ok(LocationId(uuid))
        }
    }
}

/// Unique identifier for a building within a location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BuildingId(pub Uuid);

impl BuildingId {
    /// Create a new random building ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for BuildingId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BuildingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BLD_{}", self.0.simple())
    }
}

impl Serialize for BuildingId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("BLD_{}", self.0.simple()))
    }
}

impl<'de> Deserialize<'de> for BuildingId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some(uuid_str) = s.strip_prefix("BLD_") {
            let uuid = Uuid::parse_str(uuid_str).map_err(serde::de::Error::custom)?;
            Ok(BuildingId(uuid))
        } else {
            // Fallback: try to parse as raw UUID for backward compatibility
            let uuid = Uuid::parse_str(&s).map_err(serde::de::Error::custom)?;
            Ok(BuildingId(uuid))
        }
    }
}

/// Unique identifier for a room within a building
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoomId(pub Uuid);

impl RoomId {
    /// Create a new random room ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RoomId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ROOM_{}", self.0.simple())
    }
}

impl Serialize for RoomId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("ROOM_{}", self.0.simple()))
    }
}

impl<'de> Deserialize<'de> for RoomId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some(uuid_str) = s.strip_prefix("ROOM_") {
            let uuid = Uuid::parse_str(uuid_str).map_err(serde::de::Error::custom)?;
            Ok(RoomId(uuid))
        } else {
            // Fallback: try to parse as raw UUID for backward compatibility
            let uuid = Uuid::parse_str(&s).map_err(serde::de::Error::custom)?;
            Ok(RoomId(uuid))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_creation() {
        let id1 = UserId::new();
        let id2 = UserId::new();

        // IDs should be unique
        assert_ne!(id1, id2);

        // Default should create a new ID
        let id3 = UserId::default();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_user_id_display() {
        let id = UserId::new();
        let display_str = format!("{}", id);

        // Should start with USER_ prefix
        assert!(display_str.starts_with("USER_"));

        // Should be 37 characters total (USER_ + 32 hex chars)
        assert_eq!(display_str.len(), 37);
    }

    #[test]
    fn test_location_id_creation() {
        let id1 = LocationId::new();
        let id2 = LocationId::new();

        // IDs should be unique
        assert_ne!(id1, id2);

        // Default should create a new ID
        let id3 = LocationId::default();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_location_id_display() {
        let id = LocationId::new();
        let display_str = format!("{}", id);

        // Should start with LOC_ prefix
        assert!(display_str.starts_with("LOC_"));

        // Should be 36 characters total (LOC_ + 32 hex chars)
        assert_eq!(display_str.len(), 36);
    }

    #[test]
    fn test_building_id_creation() {
        let id1 = BuildingId::new();
        let id2 = BuildingId::new();

        // IDs should be unique
        assert_ne!(id1, id2);

        // Default should create a new ID
        let id3 = BuildingId::default();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_building_id_display() {
        let id = BuildingId::new();
        let display_str = format!("{}", id);

        // Should start with BLD_ prefix
        assert!(display_str.starts_with("BLD_"));

        // Should be 36 characters total (BLD_ + 32 hex chars)
        assert_eq!(display_str.len(), 36);
    }

    #[test]
    fn test_room_id_creation() {
        let id1 = RoomId::new();
        let id2 = RoomId::new();

        // IDs should be unique
        assert_ne!(id1, id2);

        // Default should create a new ID
        let id3 = RoomId::default();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_room_id_display() {
        let id = RoomId::new();
        let display_str = format!("{}", id);

        // Should start with ROOM_ prefix
        assert!(display_str.starts_with("ROOM_"));

        // Should be 37 characters total (ROOM_ + 32 hex chars)
        assert_eq!(display_str.len(), 37);
    }

    #[test]
    fn test_id_serialization() {
        let user_id = UserId::new();
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();

        // Test that IDs can be serialized and deserialized with prefixes
        let user_json = serde_json::to_string(&user_id).unwrap();
        assert!(user_json.contains("USER_"));
        let deserialized_user: UserId = serde_json::from_str(&user_json).unwrap();
        assert_eq!(user_id, deserialized_user);

        let location_json = serde_json::to_string(&location_id).unwrap();
        assert!(location_json.contains("LOC_"));
        let deserialized_location: LocationId = serde_json::from_str(&location_json).unwrap();
        assert_eq!(location_id, deserialized_location);

        let building_json = serde_json::to_string(&building_id).unwrap();
        assert!(building_json.contains("BLD_"));
        let deserialized_building: BuildingId = serde_json::from_str(&building_json).unwrap();
        assert_eq!(building_id, deserialized_building);

        let room_json = serde_json::to_string(&room_id).unwrap();
        assert!(room_json.contains("ROOM_"));
        let deserialized_room: RoomId = serde_json::from_str(&room_json).unwrap();
        assert_eq!(room_id, deserialized_room);
    }

    #[test]
    fn test_id_deserialization_backward_compatibility() {
        // Test that we can still deserialize raw UUIDs (backward compatibility)
        let raw_uuid = Uuid::new_v4();
        let raw_uuid_str = format!("\"{}\"", raw_uuid);

        let user_id: UserId = serde_json::from_str(&raw_uuid_str).unwrap();
        assert_eq!(user_id.0, raw_uuid);

        let location_id: LocationId = serde_json::from_str(&raw_uuid_str).unwrap();
        assert_eq!(location_id.0, raw_uuid);

        let building_id: BuildingId = serde_json::from_str(&raw_uuid_str).unwrap();
        assert_eq!(building_id.0, raw_uuid);

        let room_id: RoomId = serde_json::from_str(&raw_uuid_str).unwrap();
        assert_eq!(room_id.0, raw_uuid);
    }

    #[test]
    fn test_id_deserialization_with_prefixes() {
        // Test that we can deserialize prefixed IDs
        let raw_uuid = Uuid::new_v4();

        let user_json = format!("\"USER_{}\"", raw_uuid.simple());
        let user_id: UserId = serde_json::from_str(&user_json).unwrap();
        assert_eq!(user_id.0, raw_uuid);

        let location_json = format!("\"LOC_{}\"", raw_uuid.simple());
        let location_id: LocationId = serde_json::from_str(&location_json).unwrap();
        assert_eq!(location_id.0, raw_uuid);

        let building_json = format!("\"BLD_{}\"", raw_uuid.simple());
        let building_id: BuildingId = serde_json::from_str(&building_json).unwrap();
        assert_eq!(building_id.0, raw_uuid);

        let room_json = format!("\"ROOM_{}\"", raw_uuid.simple());
        let room_id: RoomId = serde_json::from_str(&room_json).unwrap();
        assert_eq!(room_id.0, raw_uuid);
    }

    #[test]
    fn test_id_hash_and_equality() {
        use std::collections::HashSet;

        let id1 = UserId::new();
        let id2 = UserId::new();
        let id1_copy = UserId(id1.0);

        // Same ID should be equal
        assert_eq!(id1, id1_copy);

        // Different IDs should not be equal
        assert_ne!(id1, id2);

        // IDs should work in hash collections
        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        set.insert(id1_copy); // Should not increase size

        assert_eq!(set.len(), 2);
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
        assert!(set.contains(&id1_copy));
    }
}
