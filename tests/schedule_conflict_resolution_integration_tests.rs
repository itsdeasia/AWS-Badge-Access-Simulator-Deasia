//! Integration tests for schedule conflict resolution with travel time and location persistence

use amzn_career_pathway_activity_rust::user::{User, ScheduledActivity};
use amzn_career_pathway_activity_rust::facility::{
    Building, Location, LocationRegistry, Room,
};
use amzn_career_pathway_activity_rust::permissions::{PermissionLevel, PermissionSet};
use amzn_career_pathway_activity_rust::simulation::{BehaviorEngine, TimeManager};
use amzn_career_pathway_activity_rust::types::{
    ActivityType, BuildingId, LocationId, RoomId, RoomType, SecurityLevel, SimulationConfig,
};
use chrono::{Duration, NaiveDate};

#[test]
fn test_simple_schedule_conflict_resolution() {
    // Simple test to verify the method works
    let config = SimulationConfig::default();
    let time_manager = TimeManager::new();
    let mut engine = BehaviorEngine::new(config, time_manager);

    let mut registry = LocationRegistry::new();
    let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));
    let location_id = location.id;

    let mut building = Building::new(location_id, "Test Building".to_string());
    let building_id = building.id;

    let room = Room::new(
        building_id,
        "Test Room".to_string(),
        RoomType::Workspace,
        SecurityLevel::Standard,
    );
    let room_id = room.id;

    building.add_room(room);
    location.add_building(building);
    registry.add_location(location);

    let mut permissions = PermissionSet::new();
    permissions.add_permission(PermissionLevel::Location(location_id));
    let user = User::new(location_id, building_id, room_id, permissions);

    let base_time =
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap().and_hms_opt(9, 0, 0).unwrap().and_utc();

    let schedule = vec![
        ScheduledActivity::new(ActivityType::Arrival, room_id, base_time, Duration::minutes(15)),
        ScheduledActivity::new(
            ActivityType::Departure,
            room_id,
            base_time + Duration::hours(8),
            Duration::minutes(10),
        ),
    ];

    let resolved_schedule =
        engine.resolve_schedule_conflicts(schedule, &user, &registry).unwrap();

    assert_eq!(resolved_schedule.len(), 2);
    assert_eq!(resolved_schedule[0].activity_type, ActivityType::Arrival);
    assert_eq!(resolved_schedule[1].activity_type, ActivityType::Departure);
}

/// Create a test user with basic permissions
fn create_test_user(
    primary_location: LocationId,
    primary_building: BuildingId,
    primary_workspace: RoomId,
) -> User {
    let mut permissions = PermissionSet::new();
    permissions.add_permission(PermissionLevel::Location(primary_location));

    User::new(primary_location, primary_building, primary_workspace, permissions)
}

/// Create a test location registry with two locations for cross-location travel testing
fn create_test_registry(
) -> (LocationRegistry, LocationId, LocationId, BuildingId, BuildingId, RoomId, RoomId) {
    let mut registry = LocationRegistry::new();

    // Location 1 (Seattle)
    let mut location1 = Location::new("Seattle".to_string(), (47.6062, -122.3321));
    let location1_id = location1.id;

    let mut building1 = Building::new(location1_id, "Building 1".to_string());
    let building1_id = building1.id;

    let room1 =
        Room::new(building1_id, "Room 1".to_string(), RoomType::Workspace, SecurityLevel::Standard);
    let room1_id = room1.id;
    let meeting_room1 = Room::new(
        building1_id,
        "Meeting Room 1".to_string(),
        RoomType::MeetingRoom,
        SecurityLevel::Standard,
    );

    building1.add_room(room1);
    building1.add_room(meeting_room1);
    location1.add_building(building1);

    // Location 2 (Portland)
    let mut location2 = Location::new("Portland".to_string(), (45.5152, -122.6784));
    let location2_id = location2.id;

    let mut building2 = Building::new(location2_id, "Building 2".to_string());
    let building2_id = building2.id;

    let room2 =
        Room::new(building2_id, "Room 2".to_string(), RoomType::Workspace, SecurityLevel::Standard);
    let room2_id = room2.id;
    let meeting_room2 = Room::new(
        building2_id,
        "Meeting Room 2".to_string(),
        RoomType::MeetingRoom,
        SecurityLevel::Standard,
    );

    building2.add_room(room2);
    building2.add_room(meeting_room2);
    location2.add_building(building2);

    registry.add_location(location1);
    registry.add_location(location2);

    (registry, location1_id, location2_id, building1_id, building2_id, room1_id, room2_id)
}

#[test]
fn test_resolve_schedule_conflicts_basic_overlap_resolution() {
    let config = SimulationConfig::default();
    let time_manager = TimeManager::new();
    let mut engine = BehaviorEngine::new(config, time_manager);

    let (registry, location1_id, _location2_id, building1_id, _building2_id, room1_id, _room2_id) =
        create_test_registry();
    let user = create_test_user(location1_id, building1_id, room1_id);

    let base_time =
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap().and_hms_opt(9, 0, 0).unwrap().and_utc();

    // Create schedule with overlapping activities (basic time conflicts)
    let schedule = vec![
        ScheduledActivity::new(
            ActivityType::Arrival,
            room1_id,
            base_time,
            Duration::minutes(60), // Long arrival
        ),
        ScheduledActivity::new(
            ActivityType::Meeting,
            room1_id,
            base_time + Duration::minutes(30), // Overlaps with arrival
            Duration::minutes(60),
        ),
        ScheduledActivity::new(
            ActivityType::Departure,
            room1_id,
            base_time + Duration::minutes(45), // Overlaps with meeting
            Duration::minutes(10),
        ),
    ];

    let resolved_schedule =
        engine.resolve_schedule_conflicts(schedule, &user, &registry).unwrap();

    // All activities should be preserved
    assert_eq!(resolved_schedule.len(), 3);

    // Activities should not overlap
    for i in 1..resolved_schedule.len() {
        let prev_end = resolved_schedule[i - 1].start_time + resolved_schedule[i - 1].duration;
        let current_start = resolved_schedule[i].start_time;

        // Current activity should start after previous activity ends (with buffer)
        assert!(current_start >= prev_end);
    }

    // Meeting should be adjusted to start after arrival ends
    let arrival_end = resolved_schedule[0].start_time + resolved_schedule[0].duration;
    assert!(resolved_schedule[1].start_time >= arrival_end);

    // Departure should be adjusted to start after meeting ends
    let meeting_end = resolved_schedule[1].start_time + resolved_schedule[1].duration;
    assert!(resolved_schedule[2].start_time >= meeting_end);
}
