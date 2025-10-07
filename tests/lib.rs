// Integration tests test your crate's public API. They only have access to items
// in your crate that are marked pub. See the Cargo Targets page of the Cargo Book
// for more information.
//
//   https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests
//

use amzn_career_pathway_activity_rust::*;

// Include unit test modules for core components
mod access_flow_tests;
mod behavior_engine_tests;
mod user_generation_tests;
mod time_management_tests;

// Include new test modules for batch processing system
mod batch_event_generation_tests;
mod cli_argument_parsing_tests;
mod event_ordering_tests;
mod statistics_consolidation_tests;

#[test]
fn test_core_id_types() {
    let user_id = UserId::new();
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();

    // Test that IDs are unique
    assert_ne!(user_id, UserId::new());
    assert_ne!(location_id, LocationId::new());
    assert_ne!(building_id, BuildingId::new());
    assert_ne!(room_id, RoomId::new());

    // Test string formatting
    assert!(user_id.to_string().starts_with("USER_"));
    assert!(location_id.to_string().starts_with("LOC_"));
    assert!(building_id.to_string().starts_with("BLD_"));
    assert!(room_id.to_string().starts_with("ROOM_"));
}

#[test]
fn test_enum_types() {
    // Test RoomType
    let room_types = [
        RoomType::Lobby,
        RoomType::Workspace,
        RoomType::MeetingRoom,
        RoomType::Bathroom,
        RoomType::Cafeteria,
        RoomType::Kitchen,
        RoomType::ServerRoom,
        RoomType::ExecutiveOffice,
        RoomType::Storage,
        RoomType::Laboratory,
    ];

    for room_type in &room_types {
        assert!(!room_type.to_string().is_empty());
    }

    // Test SecurityLevel
    let security_levels = [
        SecurityLevel::Public,
        SecurityLevel::Standard,
        SecurityLevel::Restricted,
        SecurityLevel::HighSecurity,
        SecurityLevel::MaxSecurity,
    ];

    for level in &security_levels {
        assert!(!level.to_string().is_empty());
    }

    // Test ActivityType
    let activity_types = [
        ActivityType::Arrival,
        ActivityType::Meeting,
        ActivityType::Bathroom,
        ActivityType::Lunch,
        ActivityType::Collaboration,
        ActivityType::Departure,
    ];

    for activity in &activity_types {
        assert!(!activity.to_string().is_empty());
    }

    // Test EventType
    let event_types = [
        EventType::Success,
        EventType::Failure,
        EventType::InvalidBadge,
        EventType::OutsideHours,
        EventType::Suspicious,
    ];

    for event in &event_types {
        assert!(!event.to_string().is_empty());
    }
}

#[test]
fn test_serialization_roundtrip() {
    let user_id = UserId::new();
    let json = serde_json::to_string(&user_id).unwrap();
    let deserialized: UserId = serde_json::from_str(&json).unwrap();
    assert_eq!(user_id, deserialized);

    let room_type = RoomType::ServerRoom;
    let json = serde_json::to_string(&room_type).unwrap();
    let deserialized: RoomType = serde_json::from_str(&json).unwrap();
    assert_eq!(room_type, deserialized);

    let security_level = SecurityLevel::HighSecurity;
    let json = serde_json::to_string(&security_level).unwrap();
    let deserialized: SecurityLevel = serde_json::from_str(&json).unwrap();
    assert_eq!(security_level, deserialized);
}

#[test]
fn test_id_json_output_has_prefixes() {
    let user_id = UserId::new();
    let room_id = RoomId::new();
    let building_id = BuildingId::new();
    let location_id = LocationId::new();

    let user_json = serde_json::to_string(&user_id).unwrap();
    let room_json = serde_json::to_string(&room_id).unwrap();
    let building_json = serde_json::to_string(&building_id).unwrap();
    let location_json = serde_json::to_string(&location_id).unwrap();

    println!("User ID JSON: {}", user_json);
    println!("Room ID JSON: {}", room_json);
    println!("Building ID JSON: {}", building_json);
    println!("Location ID JSON: {}", location_json);

    assert!(user_json.contains("USER_"));
    assert!(room_json.contains("ROOM_"));
    assert!(building_json.contains("BLD_"));
    assert!(location_json.contains("LOC_"));
}

#[test]
fn test_user_badge_cloning() {
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();
    let permissions = PermissionSet::new();

    // Test creating user with cloned badge
    let user =
        User::new_with_cloned_badge(location_id, building_id, room_id, permissions.clone());
    assert!(user.has_cloned_badge);

    // Test marking user badge as cloned
    let mut regular_user = User::new(location_id, building_id, room_id, permissions);
    assert!(!regular_user.has_cloned_badge);

    regular_user.mark_badge_as_cloned();
    assert!(regular_user.has_cloned_badge);

    regular_user.unmark_cloned_badge();
    assert!(!regular_user.has_cloned_badge);
}

#[test]
fn test_impossible_traveler_metadata() {
    use chrono::Duration;

    let user_id = UserId::new();
    let primary_location = LocationId::new();
    let remote_location = LocationId::new();

    let metadata = ImpossibleTravelerMetadata {
        user_id,
        primary_location,
        remote_location,
        geographical_distance_km: 1000.0,
        actual_time_gap: Duration::hours(2),
        minimum_required_time: Duration::hours(8),
        impossibility_factor: 4.0,
    };

    // Test serialization
    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: ImpossibleTravelerMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(metadata.user_id, deserialized.user_id);
    assert_eq!(metadata.geographical_distance_km, deserialized.geographical_distance_km);
    assert_eq!(metadata.impossibility_factor, deserialized.impossibility_factor);
}

#[test]
fn test_user_eligibility_for_badge_cloning() {
    let location_id = LocationId::new();
    let building_id = BuildingId::new();
    let room_id = RoomId::new();

    // Test user with high travel frequency
    let mut permissions = PermissionSet::new();
    let mut user = User::new(location_id, building_id, room_id, permissions.clone());
    user.behavior_profile.travel_frequency = 0.2; // High travel frequency
    assert!(user.is_eligible_for_badge_cloning());

    // Test user with location-level permissions
    permissions.add_permission(PermissionLevel::Location(location_id));
    let user_with_location_access =
        User::new(location_id, building_id, room_id, permissions);
    assert!(user_with_location_access.is_eligible_for_badge_cloning());

    // Test user with low travel frequency and no location permissions
    let basic_permissions = PermissionSet::new();
    let mut basic_user = User::new(location_id, building_id, room_id, basic_permissions);
    basic_user.behavior_profile.travel_frequency = 0.05; // Low travel frequency
    assert!(!basic_user.is_eligible_for_badge_cloning());
}
