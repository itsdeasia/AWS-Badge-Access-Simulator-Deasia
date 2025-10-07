//! Access events and access attempts
//!
//! This module contains access event structures and related functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{BuildingId, UserId, EventType, FailureReason, LocationId, RoomId};
use crate::types::config::OutputFieldConfig;

/// Metadata for access events with failure-specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Whether this is a curious user unauthorized access attempt
    pub is_curious_attempt: bool,
    /// Whether this is an impossible traveler scenario
    pub is_impossible_traveler: bool,
    /// Whether this is a badge technical failure
    pub is_badge_reader_failure: bool,
    /// Whether this is a night-shift user event during off-hours
    pub is_night_shift_event: bool,
    /// Retry attempt number for badge technical failures (1 for first retry, 2 for second, etc.)
    pub retry_attempt_number: Option<u8>,
    /// Time violation for impossible traveler scenarios
    pub travel_time_violation: Option<chrono::Duration>,
    /// Geographical distance for impossible traveler scenarios (in kilometers)
    pub geographical_distance: Option<f64>,
}

impl EventMetadata {
    /// Create new event metadata with all fields set to default values
    pub fn new() -> Self {
        Self {
            is_curious_attempt: false,
            is_impossible_traveler: false,
            is_badge_reader_failure: false,
            is_night_shift_event: false,
            retry_attempt_number: None,
            travel_time_violation: None,
            geographical_distance: None,
        }
    }

    /// Create metadata for a curious user event
    pub fn curious_attempt() -> Self {
        Self {
            is_curious_attempt: true,
            is_impossible_traveler: false,
            is_badge_reader_failure: false,
            is_night_shift_event: false,
            retry_attempt_number: None,
            travel_time_violation: None,
            geographical_distance: None,
        }
    }

    /// Create metadata for an impossible traveler event
    pub fn impossible_traveler(travel_time_violation: chrono::Duration, geographical_distance: f64) -> Self {
        Self {
            is_curious_attempt: false,
            is_impossible_traveler: true,
            is_badge_reader_failure: false,
            is_night_shift_event: false,
            retry_attempt_number: None,
            travel_time_violation: Some(travel_time_violation),
            geographical_distance: Some(geographical_distance),
        }
    }

    /// Create metadata for a badge technical failure event
    pub fn badge_reader_failure(retry_attempt_number: Option<u8>) -> Self {
        Self {
            is_curious_attempt: false,
            is_impossible_traveler: false,
            is_badge_reader_failure: true,
            is_night_shift_event: false,
            retry_attempt_number,
            travel_time_violation: None,
            geographical_distance: None,
        }
    }

    /// Create metadata for a night-shift user event
    pub fn night_shift_event() -> Self {
        Self {
            is_curious_attempt: false,
            is_impossible_traveler: false,
            is_badge_reader_failure: false,
            is_night_shift_event: true,
            retry_attempt_number: None,
            travel_time_violation: None,
            geographical_distance: None,
        }
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an access attempt by a user to a specific room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessAttempt {
    /// ID of the user making the access attempt
    pub user_id: UserId,
    /// ID of the room being accessed
    pub target_room: RoomId,
    /// Whether the user is authorized to access this room
    pub is_authorized: bool,
    /// Timestamp when the access attempt occurs
    pub timestamp: DateTime<Utc>,
}

impl AccessAttempt {
    /// Create a new access attempt
    pub fn new(
        user_id: UserId,
        target_room: RoomId,
        is_authorized: bool,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self { user_id, target_room, is_authorized, timestamp }
    }

    /// Check if this access attempt should succeed
    pub fn should_succeed(&self) -> bool {
        self.is_authorized
    }
}

/// Represents an access event that occurred in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessEvent {
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    /// ID of the user who attempted access
    pub user_id: UserId,
    /// ID of the room that was accessed
    pub room_id: RoomId,
    /// ID of the building containing the room
    pub building_id: BuildingId,
    /// ID of the location containing the building
    pub location_id: LocationId,
    /// Whether the access attempt was successful
    pub success: bool,
    /// Type of event that occurred
    pub event_type: EventType,
    /// Reason for failure (if applicable)
    pub failure_reason: Option<FailureReason>,
    /// Additional metadata about the event
    pub metadata: Option<EventMetadata>,
}

impl AccessEvent {
    /// Create a new access event
    pub fn new(
        timestamp: DateTime<Utc>,
        user_id: UserId,
        room_id: RoomId,
        building_id: BuildingId,
        location_id: LocationId,
        success: bool,
        event_type: EventType,
    ) -> Self {
        Self { 
            timestamp, 
            user_id, 
            room_id, 
            building_id, 
            location_id, 
            success, 
            event_type,
            failure_reason: None,
            metadata: None,
        }
    }

    /// Create a new access event with failure reason and metadata
    pub fn new_with_failure_info(
        timestamp: DateTime<Utc>,
        user_id: UserId,
        room_id: RoomId,
        building_id: BuildingId,
        location_id: LocationId,
        success: bool,
        event_type: EventType,
        failure_reason: Option<FailureReason>,
        metadata: Option<EventMetadata>,
    ) -> Self {
        Self { 
            timestamp, 
            user_id, 
            room_id, 
            building_id, 
            location_id, 
            success, 
            event_type,
            failure_reason,
            metadata,
        }
    }

    /// Create an access event from an access attempt
    pub fn from_access_attempt(
        attempt: &AccessAttempt,
        building_id: BuildingId,
        location_id: LocationId,
    ) -> Self {
        let success = attempt.is_authorized;
        let event_type = if success { EventType::Success } else { EventType::Failure };
        let failure_reason = if success { None } else { Some(FailureReason::Unauthorized) };

        Self::new_with_failure_info(
            attempt.timestamp,
            attempt.user_id,
            attempt.target_room,
            building_id,
            location_id,
            success,
            event_type,
            failure_reason,
            None,
        )
    }

    /// Check if this event represents a successful access
    pub fn is_successful(&self) -> bool {
        self.success
    }

    /// Check if this event represents a failed access
    pub fn is_failed(&self) -> bool {
        !self.success
    }

    /// Check if this event occurred outside business hours
    pub fn is_outside_hours(&self) -> bool {
        matches!(self.event_type, EventType::OutsideHours)
    }

    /// Check if this event is marked as suspicious
    pub fn is_suspicious(&self) -> bool {
        matches!(self.event_type, EventType::Suspicious)
    }

    /// Check if this event is a badge technical failure
    pub fn is_badge_reader_failure(&self) -> bool {
        matches!(self.failure_reason, Some(FailureReason::BadgeReaderError))
    }

    /// Check if this event is a curious user attempt
    pub fn is_curious_attempt(&self) -> bool {
        matches!(self.failure_reason, Some(FailureReason::CuriousUser))
    }

    /// Check if this event is an impossible traveler scenario
    pub fn is_impossible_traveler(&self) -> bool {
        matches!(self.failure_reason, Some(FailureReason::ImpossibleTraveler))
    }

    /// Get the retry attempt number for badge technical failures
    pub fn get_retry_attempt_number(&self) -> Option<u8> {
        self.metadata.as_ref().and_then(|m| m.retry_attempt_number)
    }
}

/// Filtered access event structure for custom serialization based on field configuration
#[derive(Debug, Clone, Serialize)]
pub struct FilteredAccessEvent {
    /// Timestamp when the event occurred (always included)
    pub timestamp: DateTime<Utc>,
    /// ID of the user who attempted access (always included)
    pub user_id: UserId,
    /// ID of the room that was accessed (always included)
    pub room_id: RoomId,
    /// ID of the building containing the room (always included)
    pub building_id: BuildingId,
    /// ID of the location containing the building (always included)
    pub location_id: LocationId,
    /// Whether the access attempt was successful (always included)
    pub success: bool,
    /// Type of event that occurred (optional based on configuration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<EventType>,
    /// Reason for failure (optional based on configuration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<FailureReason>,
    /// Additional metadata about the event (optional based on configuration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<EventMetadata>,
}

impl FilteredAccessEvent {
    /// Create a FilteredAccessEvent from an AccessEvent based on field configuration
    pub fn from_access_event(event: &AccessEvent, field_config: &OutputFieldConfig) -> Self {
        Self {
            // Core fields are always included
            timestamp: event.timestamp,
            user_id: event.user_id,
            room_id: event.room_id,
            building_id: event.building_id,
            location_id: event.location_id,
            success: event.success,
            // Optional fields based on configuration
            event_type: if field_config.include_event_type || field_config.include_all {
                Some(event.event_type)
            } else {
                None
            },
            failure_reason: if field_config.include_failure_reason || field_config.include_all {
                event.failure_reason
            } else {
                None
            },
            metadata: if field_config.include_metadata || field_config.include_all {
                event.metadata.clone()
            } else {
                None
            },
        }
    }

    /// Get the core field names that are always included in output
    pub fn get_core_field_names() -> Vec<&'static str> {
        vec![
            "timestamp",
            "user_id", 
            "room_id",
            "building_id",
            "location_id",
            "success"
        ]
    }

    /// Get the optional field names based on configuration
    pub fn get_optional_field_names(field_config: &OutputFieldConfig) -> Vec<&'static str> {
        let mut fields = Vec::new();
        
        if field_config.include_event_type || field_config.include_all {
            fields.push("event_type");
        }
        if field_config.include_failure_reason || field_config.include_all {
            fields.push("failure_reason");
        }
        if field_config.include_metadata || field_config.include_all {
            fields.push("metadata");
        }
        
        fields
    }

    /// Get all field names that will be included in output based on configuration
    pub fn get_all_field_names(field_config: &OutputFieldConfig) -> Vec<&'static str> {
        let mut fields = Self::get_core_field_names();
        fields.extend(Self::get_optional_field_names(field_config));
        fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BuildingId, UserId, LocationId, RoomId};

    #[test]
    fn test_access_attempt_creation() {
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let timestamp = Utc::now();

        let attempt = AccessAttempt::new(user_id, room_id, true, timestamp);

        assert_eq!(attempt.user_id, user_id);
        assert_eq!(attempt.target_room, room_id);
        assert!(attempt.is_authorized);
        assert_eq!(attempt.timestamp, timestamp);
        assert!(attempt.should_succeed());

        let unauthorized_attempt = AccessAttempt::new(user_id, room_id, false, timestamp);
        assert!(!unauthorized_attempt.should_succeed());
    }

    #[test]
    fn test_access_event_creation() {
        let timestamp = Utc::now();
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let event = AccessEvent::new(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            true,
            EventType::Success,
        );

        assert_eq!(event.timestamp, timestamp);
        assert_eq!(event.user_id, user_id);
        assert_eq!(event.room_id, room_id);
        assert_eq!(event.building_id, building_id);
        assert_eq!(event.location_id, location_id);
        assert!(event.success);
        assert_eq!(event.event_type, EventType::Success);
        assert!(event.is_successful());
        assert!(!event.is_failed());
        assert!(event.failure_reason.is_none());
        assert!(event.metadata.is_none());
    }

    #[test]
    fn test_access_event_from_attempt() {
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let attempt = AccessAttempt::new(user_id, room_id, true, timestamp);
        let event = AccessEvent::from_access_attempt(&attempt, building_id, location_id);

        assert_eq!(event.timestamp, timestamp);
        assert_eq!(event.user_id, user_id);
        assert_eq!(event.room_id, room_id);
        assert_eq!(event.building_id, building_id);
        assert_eq!(event.location_id, location_id);
        assert!(event.success);
        assert_eq!(event.event_type, EventType::Success);
    }

    #[test]
    fn test_failed_access_event() {
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();
        let timestamp = Utc::now();

        let failed_attempt = AccessAttempt::new(user_id, room_id, false, timestamp);
        let failed_event =
            AccessEvent::from_access_attempt(&failed_attempt, building_id, location_id);

        assert!(!failed_event.success);
        assert_eq!(failed_event.event_type, EventType::Failure);
        assert!(!failed_event.is_successful());
        assert!(failed_event.is_failed());
        assert_eq!(failed_event.failure_reason, Some(FailureReason::Unauthorized));
    }

    #[test]
    fn test_event_type_checks() {
        let timestamp = Utc::now();
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let outside_hours_event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::OutsideHours,
            Some(FailureReason::OutsideHours),
            None,
        );

        assert!(outside_hours_event.is_outside_hours());
        assert!(!outside_hours_event.is_suspicious());

        let suspicious_event = AccessEvent::new(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Suspicious,
        );

        assert!(!suspicious_event.is_outside_hours());
        assert!(suspicious_event.is_suspicious());
    }

    #[test]
    fn test_badge_reader_failure_event() {
        let timestamp = Utc::now();
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        // Test badge technical failure event
        let badge_reader_failure_event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Failure,
            Some(FailureReason::BadgeReaderError),
            Some(EventMetadata::badge_reader_failure(None)),
        );

        assert!(!badge_reader_failure_event.success);
        assert_eq!(badge_reader_failure_event.event_type, EventType::Failure);
        assert_eq!(badge_reader_failure_event.failure_reason, Some(FailureReason::BadgeReaderError));
        assert!(badge_reader_failure_event.is_badge_reader_failure());
        assert!(!badge_reader_failure_event.is_curious_attempt());
        assert!(!badge_reader_failure_event.is_impossible_traveler());
        assert_eq!(badge_reader_failure_event.get_retry_attempt_number(), None);

        // Test successful retry event
        let retry_event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            true,
            EventType::Success,
            None,
            Some(EventMetadata::badge_reader_failure(Some(1))),
        );

        assert!(retry_event.success);
        assert_eq!(retry_event.event_type, EventType::Success);
        assert!(retry_event.failure_reason.is_none());
        assert!(!retry_event.is_badge_reader_failure());
        assert_eq!(retry_event.get_retry_attempt_number(), Some(1));
    }

    #[test]
    fn test_event_metadata_creation() {
        // Test default metadata
        let default_metadata = EventMetadata::new();
        assert!(!default_metadata.is_curious_attempt);
        assert!(!default_metadata.is_impossible_traveler);
        assert!(!default_metadata.is_badge_reader_failure);
        assert!(default_metadata.retry_attempt_number.is_none());

        // Test curious attempt metadata
        let curious_metadata = EventMetadata::curious_attempt();
        assert!(curious_metadata.is_curious_attempt);
        assert!(!curious_metadata.is_impossible_traveler);
        assert!(!curious_metadata.is_badge_reader_failure);

        // Test impossible traveler metadata
        let travel_time = chrono::Duration::hours(2);
        let distance = 1000.0;
        let impossible_metadata = EventMetadata::impossible_traveler(travel_time, distance);
        assert!(!impossible_metadata.is_curious_attempt);
        assert!(impossible_metadata.is_impossible_traveler);
        assert!(!impossible_metadata.is_badge_reader_failure);
        assert_eq!(impossible_metadata.travel_time_violation, Some(travel_time));
        assert_eq!(impossible_metadata.geographical_distance, Some(distance));

        // Test badge technical failure metadata
        let badge_failure_metadata = EventMetadata::badge_reader_failure(Some(2));
        assert!(!badge_failure_metadata.is_curious_attempt);
        assert!(!badge_failure_metadata.is_impossible_traveler);
        assert!(badge_failure_metadata.is_badge_reader_failure);
        assert_eq!(badge_failure_metadata.retry_attempt_number, Some(2));
    }

    #[test]
    fn test_filtered_access_event_minimal_output() {
        let timestamp = Utc::now();
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Failure,
            Some(FailureReason::Unauthorized),
            Some(EventMetadata::curious_attempt()),
        );

        // Test minimal output (default configuration)
        let field_config = OutputFieldConfig::default();
        let filtered_event = FilteredAccessEvent::from_access_event(&event, &field_config);

        // Core fields should be present
        assert_eq!(filtered_event.timestamp, timestamp);
        assert_eq!(filtered_event.user_id, user_id);
        assert_eq!(filtered_event.room_id, room_id);
        assert_eq!(filtered_event.building_id, building_id);
        assert_eq!(filtered_event.location_id, location_id);
        assert!(!filtered_event.success);

        // Optional fields should be None
        assert!(filtered_event.event_type.is_none());
        assert!(filtered_event.failure_reason.is_none());
        assert!(filtered_event.metadata.is_none());
    }

    #[test]
    fn test_filtered_access_event_with_failure_reason() {
        let timestamp = Utc::now();
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Failure,
            Some(FailureReason::Unauthorized),
            Some(EventMetadata::curious_attempt()),
        );

        // Test with failure_reason enabled
        let field_config = OutputFieldConfig {
            include_failure_reason: true,
            include_event_type: false,
            include_metadata: false,
            include_all: false,
        };
        let filtered_event = FilteredAccessEvent::from_access_event(&event, &field_config);

        // Core fields should be present
        assert_eq!(filtered_event.timestamp, timestamp);
        assert_eq!(filtered_event.user_id, user_id);
        assert!(!filtered_event.success);

        // Only failure_reason should be included
        assert!(filtered_event.event_type.is_none());
        assert_eq!(filtered_event.failure_reason, Some(FailureReason::Unauthorized));
        assert!(filtered_event.metadata.is_none());
    }

    #[test]
    fn test_filtered_access_event_with_event_type() {
        let timestamp = Utc::now();
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Suspicious,
            Some(FailureReason::Unauthorized),
            Some(EventMetadata::curious_attempt()),
        );

        // Test with event_type enabled
        let field_config = OutputFieldConfig {
            include_failure_reason: false,
            include_event_type: true,
            include_metadata: false,
            include_all: false,
        };
        let filtered_event = FilteredAccessEvent::from_access_event(&event, &field_config);

        // Only event_type should be included
        assert_eq!(filtered_event.event_type, Some(EventType::Suspicious));
        assert!(filtered_event.failure_reason.is_none());
        assert!(filtered_event.metadata.is_none());
    }

    #[test]
    fn test_filtered_access_event_with_metadata() {
        let timestamp = Utc::now();
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let metadata = EventMetadata::curious_attempt();
        let event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Failure,
            Some(FailureReason::CuriousUser),
            Some(metadata.clone()),
        );

        // Test with metadata enabled
        let field_config = OutputFieldConfig {
            include_failure_reason: false,
            include_event_type: false,
            include_metadata: true,
            include_all: false,
        };
        let filtered_event = FilteredAccessEvent::from_access_event(&event, &field_config);

        // Only metadata should be included
        assert!(filtered_event.event_type.is_none());
        assert!(filtered_event.failure_reason.is_none());
        assert!(filtered_event.metadata.is_some());
        assert!(filtered_event.metadata.as_ref().unwrap().is_curious_attempt);
    }

    #[test]
    fn test_filtered_access_event_with_all_fields() {
        let timestamp = Utc::now();
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let metadata = EventMetadata::impossible_traveler(chrono::Duration::hours(1), 500.0);
        let event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Suspicious,
            Some(FailureReason::ImpossibleTraveler),
            Some(metadata.clone()),
        );

        // Test with all fields enabled
        let field_config = OutputFieldConfig {
            include_failure_reason: false,
            include_event_type: false,
            include_metadata: false,
            include_all: true,
        };
        let filtered_event = FilteredAccessEvent::from_access_event(&event, &field_config);

        // All fields should be included when include_all is true
        assert_eq!(filtered_event.event_type, Some(EventType::Suspicious));
        assert_eq!(filtered_event.failure_reason, Some(FailureReason::ImpossibleTraveler));
        assert!(filtered_event.metadata.is_some());
        assert!(filtered_event.metadata.as_ref().unwrap().is_impossible_traveler);
    }

    #[test]
    fn test_filtered_access_event_include_all_overrides_individual_settings() {
        let timestamp = Utc::now();
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Failure,
            Some(FailureReason::Unauthorized),
            Some(EventMetadata::curious_attempt()),
        );

        // Test that include_all overrides individual settings
        let field_config = OutputFieldConfig {
            include_failure_reason: false,
            include_event_type: false,
            include_metadata: false,
            include_all: true,
        };
        let filtered_event = FilteredAccessEvent::from_access_event(&event, &field_config);

        // All fields should be included despite individual settings being false
        assert_eq!(filtered_event.event_type, Some(EventType::Failure));
        assert_eq!(filtered_event.failure_reason, Some(FailureReason::Unauthorized));
        assert!(filtered_event.metadata.is_some());
    }

    #[test]
    fn test_filtered_access_event_field_names() {
        // Test core field names
        let core_fields = FilteredAccessEvent::get_core_field_names();
        assert_eq!(core_fields, vec![
            "timestamp", "user_id", "room_id", "building_id", "location_id", "success"
        ]);

        // Test optional field names with minimal config
        let minimal_config = OutputFieldConfig::default();
        let optional_fields = FilteredAccessEvent::get_optional_field_names(&minimal_config);
        assert!(optional_fields.is_empty());

        // Test optional field names with individual fields enabled
        let partial_config = OutputFieldConfig {
            include_failure_reason: true,
            include_event_type: false,
            include_metadata: true,
            include_all: false,
        };
        let optional_fields = FilteredAccessEvent::get_optional_field_names(&partial_config);
        assert_eq!(optional_fields, vec!["failure_reason", "metadata"]);

        // Test optional field names with all fields enabled
        let all_config = OutputFieldConfig {
            include_failure_reason: false,
            include_event_type: false,
            include_metadata: false,
            include_all: true,
        };
        let optional_fields = FilteredAccessEvent::get_optional_field_names(&all_config);
        assert_eq!(optional_fields, vec!["event_type", "failure_reason", "metadata"]);

        // Test all field names
        let all_fields = FilteredAccessEvent::get_all_field_names(&all_config);
        assert_eq!(all_fields, vec![
            "timestamp", "user_id", "room_id", "building_id", "location_id", "success",
            "event_type", "failure_reason", "metadata"
        ]);
    }

    #[test]
    fn test_filtered_access_event_json_serialization() {
        let timestamp = Utc::now();
        let user_id = UserId::new();
        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let event = AccessEvent::new_with_failure_info(
            timestamp,
            user_id,
            room_id,
            building_id,
            location_id,
            false,
            EventType::Failure,
            Some(FailureReason::Unauthorized),
            Some(EventMetadata::curious_attempt()),
        );

        // Test JSON serialization with minimal output
        let minimal_config = OutputFieldConfig::default();
        let filtered_event = FilteredAccessEvent::from_access_event(&event, &minimal_config);
        let json = serde_json::to_string(&filtered_event).unwrap();
        
        // Should not contain optional fields
        assert!(!json.contains("event_type"));
        assert!(!json.contains("failure_reason"));
        assert!(!json.contains("metadata"));
        
        // Should contain core fields
        assert!(json.contains("timestamp"));
        assert!(json.contains("user_id"));
        assert!(json.contains("success"));

        // Test JSON serialization with all fields
        let all_config = OutputFieldConfig {
            include_failure_reason: false,
            include_event_type: false,
            include_metadata: false,
            include_all: true,
        };
        let filtered_event_all = FilteredAccessEvent::from_access_event(&event, &all_config);
        let json_all = serde_json::to_string(&filtered_event_all).unwrap();
        
        // Should contain all fields
        assert!(json_all.contains("event_type"));
        assert!(json_all.contains("failure_reason"));
        assert!(json_all.contains("metadata"));
    }
}
