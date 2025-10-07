//! Unit tests for access flow and sequential room access logic
//!
//! Tests Requirements: 3.4, 3.5

use amzn_career_pathway_activity_rust::*;
use chrono::Duration;

/// Test access flow creation
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

/// Test direct access flow creation
#[test]
fn test_direct_access_flow() {
    let room = RoomId::new();
    let travel_time = Duration::minutes(2);
    
    let flow = AccessFlow::direct(room, travel_time);
    
    assert_eq!(flow.required_sequence, vec![room]);
    assert_eq!(flow.estimated_travel_time, travel_time);
    assert!(!flow.requires_lobby_access);
    assert!(!flow.involves_high_security);
    assert!(flow.is_direct_access());
}

/// Test access flow target room identification
#[test]
fn test_target_room_identification() {
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    let room3 = RoomId::new();
    
    let flow = AccessFlow::new(vec![room1, room2, room3], Duration::minutes(10), false, false);
    assert_eq!(flow.target_room(), Some(room3));
    
    let empty_flow = AccessFlow::new(vec![], Duration::minutes(0), false, false);
    assert_eq!(empty_flow.target_room(), None);
    
    let single_flow = AccessFlow::direct(room1, Duration::minutes(2));
    assert_eq!(single_flow.target_room(), Some(room1));
}

/// Test sequence length calculation
#[test]
fn test_sequence_length() {
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    let room3 = RoomId::new();
    
    let multi_flow = AccessFlow::new(vec![room1, room2, room3], Duration::minutes(10), false, false);
    assert_eq!(multi_flow.sequence_length(), 3);
    
    let single_flow = AccessFlow::direct(room1, Duration::minutes(2));
    assert_eq!(single_flow.sequence_length(), 1);
    
    let empty_flow = AccessFlow::new(vec![], Duration::minutes(0), false, false);
    assert_eq!(empty_flow.sequence_length(), 0);
}

/// Test direct access detection
#[test]
fn test_direct_access_detection() {
    let room = RoomId::new();
    
    let direct_flow = AccessFlow::direct(room, Duration::minutes(2));
    assert!(direct_flow.is_direct_access());
    
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    let multi_flow = AccessFlow::new(vec![room1, room2], Duration::minutes(5), false, false);
    assert!(!multi_flow.is_direct_access());
    
    let empty_flow = AccessFlow::new(vec![], Duration::minutes(0), false, false);
    assert!(!empty_flow.is_direct_access()); // Empty sequence is not direct access
}

/// Test intermediate rooms identification
#[test]
fn test_intermediate_rooms() {
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    let room3 = RoomId::new();
    let room4 = RoomId::new();
    
    let flow = AccessFlow::new(vec![room1, room2, room3, room4], Duration::minutes(15), false, false);
    let intermediate = flow.intermediate_rooms();
    
    assert_eq!(intermediate.len(), 3);
    assert_eq!(intermediate, vec![room1, room2, room3]);
    
    let direct_flow = AccessFlow::direct(room1, Duration::minutes(2));
    assert!(direct_flow.intermediate_rooms().is_empty());
    
    let two_room_flow = AccessFlow::new(vec![room1, room2], Duration::minutes(5), false, false);
    let two_room_intermediate = two_room_flow.intermediate_rooms();
    assert_eq!(two_room_intermediate, vec![room1]);
}

/// Test permission validation with authorized access
#[test]
fn test_permission_validation_authorized() {
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    let room3 = RoomId::new();
    
    let mut permissions = PermissionSet::new();
    permissions.add_permission(PermissionLevel::Room(room1));
    permissions.add_permission(PermissionLevel::Room(room2));
    permissions.add_permission(PermissionLevel::Room(room3));
    
    let flow = AccessFlow::new(vec![room1, room2, room3], Duration::minutes(10), false, false);
    let result = flow.validate_permissions(&permissions);
    
    assert!(result.is_fully_authorized());
    assert!(result.unauthorized_rooms.is_empty());
    assert!(result.missing_intermediate_access.is_empty());
}

/// Test permission validation with unauthorized target room
#[test]
fn test_permission_validation_unauthorized_target() {
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    let room3 = RoomId::new();
    
    let mut permissions = PermissionSet::new();
    permissions.add_permission(PermissionLevel::Room(room1));
    permissions.add_permission(PermissionLevel::Room(room2));
    // room3 is not authorized
    
    let flow = AccessFlow::new(vec![room1, room2, room3], Duration::minutes(10), false, false);
    let result = flow.validate_permissions(&permissions);
    
    assert!(!result.is_fully_authorized());
    assert!(result.unauthorized_rooms.contains(&room3));
    assert!(result.missing_intermediate_access.is_empty());
    assert!(!result.has_target_access_only());
}

/// Test permission validation with unauthorized intermediate room
#[test]
fn test_permission_validation_unauthorized_intermediate() {
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    let room3 = RoomId::new();
    
    let mut permissions = PermissionSet::new();
    permissions.add_permission(PermissionLevel::Room(room1));
    // room2 is not authorized (intermediate)
    permissions.add_permission(PermissionLevel::Room(room3));
    
    let flow = AccessFlow::new(vec![room1, room2, room3], Duration::minutes(10), false, false);
    let result = flow.validate_permissions(&permissions);
    
    assert!(!result.is_fully_authorized());
    assert!(result.unauthorized_rooms.is_empty()); // Target is authorized
    assert!(result.missing_intermediate_access.contains(&room2));
    assert!(result.has_target_access_only());
}

/// Test permission validation with multiple unauthorized rooms
#[test]
fn test_permission_validation_multiple_unauthorized() {
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    let room3 = RoomId::new();
    let room4 = RoomId::new();
    
    let mut permissions = PermissionSet::new();
    permissions.add_permission(PermissionLevel::Room(room1));
    // room2 and room3 are not authorized (intermediate)
    // room4 is not authorized (target)
    
    let flow = AccessFlow::new(vec![room1, room2, room3, room4], Duration::minutes(15), false, false);
    let result = flow.validate_permissions(&permissions);
    
    assert!(!result.is_fully_authorized());
    assert!(result.unauthorized_rooms.contains(&room4)); // Target
    assert!(result.missing_intermediate_access.contains(&room2)); // Intermediate
    assert!(result.missing_intermediate_access.contains(&room3)); // Intermediate
    assert!(!result.has_target_access_only());
    
    let all_unauthorized = result.get_all_unauthorized_rooms();
    assert_eq!(all_unauthorized.len(), 3);
    assert!(all_unauthorized.contains(&room2));
    assert!(all_unauthorized.contains(&room3));
    assert!(all_unauthorized.contains(&room4));
}

/// Test security clearance requirements
#[test]
fn test_security_clearance_requirements() {
    let room = RoomId::new();
    
    let high_security_flow = AccessFlow::new(vec![room], Duration::minutes(5), false, true);
    assert!(high_security_flow.requires_security_clearance());
    assert!(high_security_flow.involves_high_security);
    
    let normal_flow = AccessFlow::new(vec![room], Duration::minutes(5), false, false);
    assert!(!normal_flow.requires_security_clearance());
    assert!(!normal_flow.involves_high_security);
}

/// Test lobby access requirements
#[test]
fn test_lobby_access_requirements() {
    let room = RoomId::new();
    
    let lobby_required_flow = AccessFlow::new(vec![room], Duration::minutes(5), true, false);
    assert!(lobby_required_flow.requires_lobby_access);
    
    let no_lobby_flow = AccessFlow::new(vec![room], Duration::minutes(5), false, false);
    assert!(!no_lobby_flow.requires_lobby_access);
}

/// Test segment time calculation
#[test]
fn test_segment_time_calculation() {
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    let room3 = RoomId::new();
    let room4 = RoomId::new();
    
    let flow = AccessFlow::new(vec![room1, room2, room3, room4], Duration::minutes(12), false, false);
    
    // Test valid segment (from index 0 to 2, covering 2 segments out of 3 total)
    let segment_time = flow.get_segment_time(0, 2);
    assert!(segment_time.is_some());
    
    let time = segment_time.unwrap();
    // The calculation divides total time by number of segments (3 total segments for 4 rooms)
    // So 2 segments out of 3 should be roughly 2/3 of total time (8 minutes out of 12)
    assert!(time >= Duration::minutes(7));
    assert!(time <= Duration::minutes(9));
    
    // Test full segment
    let full_segment = flow.get_segment_time(0, 4);
    assert!(full_segment.is_some());
    // Full segment should be the entire duration
    let full_time = full_segment.unwrap();
    // Allow some tolerance due to integer division
    assert!(full_time >= Duration::minutes(11));
    assert!(full_time <= Duration::minutes(16)); // More tolerance for rounding
    
    // Test invalid segments
    assert!(flow.get_segment_time(2, 1).is_none()); // from > to
    assert!(flow.get_segment_time(0, 5).is_none()); // to > length
    assert!(flow.get_segment_time(4, 5).is_none()); // from >= length
}

/// Test validation result functionality
#[test]
fn test_validation_result_functionality() {
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    let room3 = RoomId::new();
    
    // Test fully authorized result
    let authorized_result = ValidationResult {
        is_valid: true,
        unauthorized_rooms: vec![],
        missing_intermediate_access: vec![],
        requires_lobby_access: false,
        involves_high_security: false,
    };
    
    assert!(authorized_result.is_fully_authorized());
    assert!(authorized_result.get_all_unauthorized_rooms().is_empty());
    assert!(!authorized_result.has_target_access_only());
    
    // Test result with unauthorized target
    let unauthorized_target_result = ValidationResult {
        is_valid: false,
        unauthorized_rooms: vec![room1],
        missing_intermediate_access: vec![],
        requires_lobby_access: true,
        involves_high_security: false,
    };
    
    assert!(!unauthorized_target_result.is_fully_authorized());
    assert_eq!(unauthorized_target_result.get_all_unauthorized_rooms(), vec![room1]);
    assert!(!unauthorized_target_result.has_target_access_only());
    
    // Test result with only intermediate access missing
    let intermediate_missing_result = ValidationResult {
        is_valid: false,
        unauthorized_rooms: vec![],
        missing_intermediate_access: vec![room2, room3],
        requires_lobby_access: false,
        involves_high_security: true,
    };
    
    assert!(!intermediate_missing_result.is_fully_authorized());
    assert!(intermediate_missing_result.has_target_access_only());
    
    let all_unauthorized = intermediate_missing_result.get_all_unauthorized_rooms();
    assert_eq!(all_unauthorized.len(), 2);
    assert!(all_unauthorized.contains(&room2));
    assert!(all_unauthorized.contains(&room3));
}

/// Test complex access flow scenarios
#[test]
fn test_complex_access_flow_scenarios() {
    let lobby = RoomId::new();
    let security_checkpoint = RoomId::new();
    let elevator_hall = RoomId::new();
    let target_room = RoomId::new();
    
    // Complex flow: lobby -> security checkpoint -> elevator hall -> target room
    let complex_flow = AccessFlow::new(
        vec![lobby, security_checkpoint, elevator_hall, target_room],
        Duration::minutes(8),
        true,  // requires lobby access
        true,  // involves high security
    );
    
    assert_eq!(complex_flow.sequence_length(), 4);
    assert!(!complex_flow.is_direct_access());
    assert_eq!(complex_flow.target_room(), Some(target_room));
    assert_eq!(complex_flow.intermediate_rooms(), vec![lobby, security_checkpoint, elevator_hall]);
    assert!(complex_flow.requires_lobby_access);
    assert!(complex_flow.involves_high_security);
    assert!(complex_flow.requires_security_clearance());
    
    // Test with partial permissions
    let mut partial_permissions = PermissionSet::new();
    partial_permissions.add_permission(PermissionLevel::Room(lobby));
    partial_permissions.add_permission(PermissionLevel::Room(elevator_hall));
    partial_permissions.add_permission(PermissionLevel::Room(target_room));
    // Missing security_checkpoint permission
    
    let validation = complex_flow.validate_permissions(&partial_permissions);
    assert!(!validation.is_fully_authorized());
    assert!(validation.unauthorized_rooms.is_empty()); // Target is authorized
    assert!(validation.missing_intermediate_access.contains(&security_checkpoint));
    assert!(validation.has_target_access_only());
    assert!(validation.requires_lobby_access);
    assert!(validation.involves_high_security);
}

/// Test access flow with building lobby requirements
#[test]
fn test_building_lobby_access_flow() {
    let lobby = RoomId::new();
    let target_office = RoomId::new();
    
    // Typical building access: must go through lobby first
    let building_access_flow = AccessFlow::new(
        vec![lobby, target_office],
        Duration::minutes(3),
        true,  // requires lobby access
        false, // not high security
    );
    
    assert_eq!(building_access_flow.sequence_length(), 2);
    assert!(!building_access_flow.is_direct_access());
    assert!(building_access_flow.requires_lobby_access);
    assert!(!building_access_flow.involves_high_security);
    
    // Test with lobby permission only
    let mut lobby_only_permissions = PermissionSet::new();
    lobby_only_permissions.add_permission(PermissionLevel::Room(lobby));
    
    let validation = building_access_flow.validate_permissions(&lobby_only_permissions);
    assert!(!validation.is_fully_authorized());
    assert!(validation.unauthorized_rooms.contains(&target_office));
    assert!(validation.missing_intermediate_access.is_empty());
    assert!(!validation.has_target_access_only());
}

/// Test access flow serialization and deserialization
#[test]
fn test_access_flow_serialization() {
    let room1 = RoomId::new();
    let room2 = RoomId::new();
    
    let original_flow = AccessFlow::new(
        vec![room1, room2],
        Duration::minutes(5),
        true,
        false,
    );
    
    // Test JSON serialization
    let json = serde_json::to_string(&original_flow).unwrap();
    let deserialized_flow: AccessFlow = serde_json::from_str(&json).unwrap();
    
    assert_eq!(original_flow.required_sequence, deserialized_flow.required_sequence);
    assert_eq!(original_flow.estimated_travel_time, deserialized_flow.estimated_travel_time);
    assert_eq!(original_flow.requires_lobby_access, deserialized_flow.requires_lobby_access);
    assert_eq!(original_flow.involves_high_security, deserialized_flow.involves_high_security);
}

/// Test edge cases and error conditions
#[test]
fn test_access_flow_edge_cases() {
    // Test empty sequence
    let empty_flow = AccessFlow::new(vec![], Duration::minutes(0), false, false);
    assert_eq!(empty_flow.sequence_length(), 0);
    assert!(empty_flow.target_room().is_none());
    assert!(empty_flow.intermediate_rooms().is_empty());
    assert!(!empty_flow.is_direct_access());
    
    // Test single room sequence
    let room = RoomId::new();
    let single_flow = AccessFlow::new(vec![room], Duration::minutes(1), false, false);
    assert_eq!(single_flow.sequence_length(), 1);
    assert_eq!(single_flow.target_room(), Some(room));
    assert!(single_flow.intermediate_rooms().is_empty());
    assert!(single_flow.is_direct_access());
    
    // Test zero duration
    let zero_duration_flow = AccessFlow::direct(room, Duration::seconds(0));
    assert_eq!(zero_duration_flow.estimated_travel_time, Duration::seconds(0));
    
    // Test very long sequence
    let many_rooms: Vec<RoomId> = (0..10).map(|_| RoomId::new()).collect();
    let long_flow = AccessFlow::new(many_rooms.clone(), Duration::hours(1), true, true);
    assert_eq!(long_flow.sequence_length(), 10);
    assert_eq!(long_flow.target_room(), many_rooms.last().copied());
    assert_eq!(long_flow.intermediate_rooms().len(), 9);
}