//! Access flow and sequence validation
//!
//! This module contains access flow logic for complex access sequences.

use crate::permissions::PermissionSet;
use crate::types::RoomId;
use chrono::Duration;
use serde::{Deserialize, Serialize};

/// Represents the required sequence of room accesses to reach a target room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessFlow {
    /// Sequence of rooms that must be accessed in order to reach the target
    pub required_sequence: Vec<RoomId>,
    /// Estimated total travel time for the entire sequence
    pub estimated_travel_time: Duration,
    /// Whether this flow requires building lobby access
    pub requires_lobby_access: bool,
    /// Whether this flow involves high-security areas
    pub involves_high_security: bool,
}

impl AccessFlow {
    /// Create a new access flow
    pub fn new(
        required_sequence: Vec<RoomId>,
        estimated_travel_time: Duration,
        requires_lobby_access: bool,
        involves_high_security: bool,
    ) -> Self {
        Self {
            required_sequence,
            estimated_travel_time,
            requires_lobby_access,
            involves_high_security,
        }
    }

    /// Create a simple access flow with just the target room
    pub fn direct(target_room: RoomId, travel_time: Duration) -> Self {
        Self {
            required_sequence: vec![target_room],
            estimated_travel_time: travel_time,
            requires_lobby_access: false,
            involves_high_security: false,
        }
    }

    /// Get the target room (last room in the sequence)
    pub fn target_room(&self) -> Option<RoomId> {
        self.required_sequence.last().copied()
    }

    /// Get the number of rooms in the access sequence
    pub fn sequence_length(&self) -> usize {
        self.required_sequence.len()
    }

    /// Check if this is a direct access (no intermediate rooms)
    pub fn is_direct_access(&self) -> bool {
        self.required_sequence.len() == 1
    }

    /// Get all intermediate rooms (excluding the target)
    pub fn intermediate_rooms(&self) -> Vec<RoomId> {
        if self.required_sequence.len() <= 1 {
            Vec::new()
        } else {
            self.required_sequence[..self.required_sequence.len() - 1].to_vec()
        }
    }

    /// Validate that a user has permissions for all rooms in the sequence
    pub fn validate_permissions(&self, permissions: &PermissionSet) -> ValidationResult {
        let mut unauthorized_rooms = Vec::new();
        let mut missing_intermediate_access = Vec::new();

        for (index, &room_id) in self.required_sequence.iter().enumerate() {
            // For this validation, we need to check if the user has any permission
            // that could grant access to this room. Since we don't have the full
            // facility registry context here, we'll do a basic check.
            let has_room_permission = permissions.get_authorized_rooms().contains(&room_id);

            if !has_room_permission {
                if index == self.required_sequence.len() - 1 {
                    // This is the target room
                    unauthorized_rooms.push(room_id);
                } else {
                    // This is an intermediate room
                    missing_intermediate_access.push(room_id);
                }
            }
        }

        ValidationResult {
            is_valid: unauthorized_rooms.is_empty() && missing_intermediate_access.is_empty(),
            unauthorized_rooms,
            missing_intermediate_access,
            requires_lobby_access: self.requires_lobby_access,
            involves_high_security: self.involves_high_security,
        }
    }

    /// Check if the access flow requires specific security clearance
    pub fn requires_security_clearance(&self) -> bool {
        self.involves_high_security
    }

    /// Get estimated time for a specific segment of the flow
    pub fn get_segment_time(&self, from_index: usize, to_index: usize) -> Option<Duration> {
        if from_index >= to_index || to_index > self.required_sequence.len() {
            return None;
        }

        // Simple estimation: divide total time by number of segments
        let total_segments = self.required_sequence.len().saturating_sub(1).max(1);
        let segment_count = to_index - from_index;
        let segment_duration =
            self.estimated_travel_time.num_milliseconds() / total_segments as i64;

        Some(Duration::milliseconds(segment_duration * segment_count as i64))
    }
}

/// Result of validating an access flow against user permissions
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the user has all required permissions
    pub is_valid: bool,
    /// Rooms the user cannot access (target rooms)
    pub unauthorized_rooms: Vec<RoomId>,
    /// Intermediate rooms the user cannot access
    pub missing_intermediate_access: Vec<RoomId>,
    /// Whether lobby access is required
    pub requires_lobby_access: bool,
    /// Whether high-security areas are involved
    pub involves_high_security: bool,
}

impl ValidationResult {
    /// Check if the validation passed completely
    pub fn is_fully_authorized(&self) -> bool {
        self.is_valid
    }

    /// Get all rooms that the user cannot access
    pub fn get_all_unauthorized_rooms(&self) -> Vec<RoomId> {
        let mut all_unauthorized = self.unauthorized_rooms.clone();
        all_unauthorized.extend(self.missing_intermediate_access.clone());
        all_unauthorized
    }

    /// Check if only intermediate access is missing (target is accessible)
    pub fn has_target_access_only(&self) -> bool {
        self.unauthorized_rooms.is_empty() && !self.missing_intermediate_access.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_flow_creation() {
        let room1 = RoomId::new();
        let room2 = RoomId::new();
        let sequence = vec![room1, room2];
        let travel_time = Duration::minutes(5);

        let flow = AccessFlow::new(sequence.clone(), travel_time, true, false);

        assert_eq!(flow.required_sequence, sequence);
        assert_eq!(flow.estimated_travel_time, travel_time);
        assert!(flow.requires_lobby_access);
        assert!(!flow.involves_high_security);
    }

    #[test]
    fn test_direct_access_flow() {
        let room = RoomId::new();
        let travel_time = Duration::minutes(2);

        let flow = AccessFlow::direct(room, travel_time);

        assert_eq!(flow.required_sequence, vec![room]);
        assert_eq!(flow.estimated_travel_time, travel_time);
        assert!(!flow.requires_lobby_access);
        assert!(!flow.involves_high_security);
    }

    #[test]
    fn test_target_room() {
        let room1 = RoomId::new();
        let room2 = RoomId::new();
        let flow = AccessFlow::new(vec![room1, room2], Duration::minutes(5), false, false);

        assert_eq!(flow.target_room(), Some(room2));

        let empty_flow = AccessFlow::new(vec![], Duration::minutes(0), false, false);
        assert_eq!(empty_flow.target_room(), None);
    }

    #[test]
    fn test_sequence_length() {
        let room1 = RoomId::new();
        let room2 = RoomId::new();
        let flow = AccessFlow::new(vec![room1, room2], Duration::minutes(5), false, false);

        assert_eq!(flow.sequence_length(), 2);
    }

    #[test]
    fn test_is_direct_access() {
        let room = RoomId::new();
        let direct_flow = AccessFlow::direct(room, Duration::minutes(2));
        assert!(direct_flow.is_direct_access());

        let room1 = RoomId::new();
        let room2 = RoomId::new();
        let multi_flow = AccessFlow::new(vec![room1, room2], Duration::minutes(5), false, false);
        assert!(!multi_flow.is_direct_access());
    }

    #[test]
    fn test_intermediate_rooms() {
        let room1 = RoomId::new();
        let room2 = RoomId::new();
        let room3 = RoomId::new();
        let flow = AccessFlow::new(vec![room1, room2, room3], Duration::minutes(10), false, false);

        let intermediate = flow.intermediate_rooms();
        assert_eq!(intermediate, vec![room1, room2]);

        let direct_flow = AccessFlow::direct(room1, Duration::minutes(2));
        assert!(direct_flow.intermediate_rooms().is_empty());
    }

    #[test]
    fn test_validate_permissions() {
        let room1 = RoomId::new();
        let room2 = RoomId::new();
        let room3 = RoomId::new();

        let mut permissions = PermissionSet::new();
        permissions.add_permission(crate::permissions::PermissionLevel::Room(room1));
        permissions.add_permission(crate::permissions::PermissionLevel::Room(room2));

        let flow = AccessFlow::new(vec![room1, room2], Duration::minutes(5), false, false);
        let result = flow.validate_permissions(&permissions);
        assert!(result.is_fully_authorized());

        // Test with unauthorized room
        let flow_with_unauthorized =
            AccessFlow::new(vec![room1, room3], Duration::minutes(5), false, false);
        let result = flow_with_unauthorized.validate_permissions(&permissions);
        assert!(!result.is_fully_authorized());
        assert!(result.unauthorized_rooms.contains(&room3));
    }

    #[test]
    fn test_requires_security_clearance() {
        let room = RoomId::new();
        let high_security_flow = AccessFlow::new(vec![room], Duration::minutes(5), false, true);
        assert!(high_security_flow.requires_security_clearance());

        let normal_flow = AccessFlow::new(vec![room], Duration::minutes(5), false, false);
        assert!(!normal_flow.requires_security_clearance());
    }

    #[test]
    fn test_get_segment_time() {
        let room1 = RoomId::new();
        let room2 = RoomId::new();
        let room3 = RoomId::new();
        let flow = AccessFlow::new(vec![room1, room2, room3], Duration::minutes(6), false, false);

        let segment_time = flow.get_segment_time(0, 2);
        assert!(segment_time.is_some());

        // Invalid segment should return None
        let invalid_segment = flow.get_segment_time(2, 1);
        assert!(invalid_segment.is_none());
    }

    #[test]
    fn test_validation_result() {
        let room1 = RoomId::new();
        let room2 = RoomId::new();

        let result = ValidationResult {
            is_valid: false,
            unauthorized_rooms: vec![room1],
            missing_intermediate_access: vec![room2],
            requires_lobby_access: true,
            involves_high_security: false,
        };

        assert!(!result.is_fully_authorized());
        assert!(!result.has_target_access_only()); // Has both target and intermediate issues

        let all_unauthorized = result.get_all_unauthorized_rooms();
        assert_eq!(all_unauthorized.len(), 2);
        assert!(all_unauthorized.contains(&room1));
        assert!(all_unauthorized.contains(&room2));
    }

    #[test]
    fn test_validation_result_target_access_only() {
        let room1 = RoomId::new();

        let result = ValidationResult {
            is_valid: false,
            unauthorized_rooms: vec![],
            missing_intermediate_access: vec![room1],
            requires_lobby_access: false,
            involves_high_security: false,
        };

        assert!(result.has_target_access_only());
    }
}
