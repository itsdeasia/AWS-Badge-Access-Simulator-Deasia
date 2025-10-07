//! Event metadata and special scenarios
//!
//! This module contains metadata for special scenarios like impossible traveler detection.

use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::types::{UserId, LocationId};

/// Metadata for impossible traveler scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpossibleTravelerMetadata {
    /// User ID involved in the impossible traveler scenario
    pub user_id: UserId,
    /// Primary location where the first event occurred
    pub primary_location: LocationId,
    /// Remote location where the impossible event occurred
    pub remote_location: LocationId,
    /// Geographical distance between locations in kilometers
    pub geographical_distance_km: f64,
    /// Actual time gap between the two events
    pub actual_time_gap: Duration,
    /// Minimum time required for physical travel between locations
    pub minimum_required_time: Duration,
    /// Factor indicating how impossible the scenario is (>1.0 means impossible)
    pub impossibility_factor: f64,
}

impl ImpossibleTravelerMetadata {
    /// Create new impossible traveler metadata
    pub fn new(
        user_id: UserId,
        primary_location: LocationId,
        remote_location: LocationId,
        geographical_distance_km: f64,
        actual_time_gap: Duration,
        minimum_required_time: Duration,
    ) -> Self {
        let impossibility_factor = if actual_time_gap.num_seconds() > 0 {
            minimum_required_time.num_seconds() as f64 / actual_time_gap.num_seconds() as f64
        } else {
            f64::INFINITY
        };

        Self {
            user_id,
            primary_location,
            remote_location,
            geographical_distance_km,
            actual_time_gap,
            minimum_required_time,
            impossibility_factor,
        }
    }

    /// Check if this scenario is physically impossible
    pub fn is_impossible(&self) -> bool {
        self.impossibility_factor > 1.0
    }

    /// Get the severity of the impossibility (higher values are more impossible)
    pub fn get_impossibility_severity(&self) -> f64 {
        self.impossibility_factor
    }

    /// Get the time deficit (how much time was missing for physical travel)
    pub fn get_time_deficit(&self) -> Duration {
        if self.minimum_required_time > self.actual_time_gap {
            self.minimum_required_time - self.actual_time_gap
        } else {
            Duration::zero()
        }
    }

    /// Check if the geographical distance indicates cross-country travel
    pub fn is_cross_country_travel(&self) -> bool {
        self.geographical_distance_km > 1000.0
    }

    /// Check if the geographical distance indicates international travel
    pub fn is_international_travel(&self) -> bool {
        self.geographical_distance_km > 2000.0
    }

    /// Get a human-readable description of the impossibility
    pub fn get_description(&self) -> String {
        if !self.is_impossible() {
            return "Travel is physically possible within the given timeframe".to_string();
        }

        let time_deficit = self.get_time_deficit();
        let _hours_deficit = time_deficit.num_hours();
        let _minutes_deficit = time_deficit.num_minutes() % 60;

        let travel_type = if self.is_international_travel() {
            "international"
        } else if self.is_cross_country_travel() {
            "cross-country"
        } else {
            "regional"
        };

        format!(
            "Impossible {} travel: {:.1} km in {} hours {} minutes (requires {} hours {} minutes minimum)",
            travel_type,
            self.geographical_distance_km,
            self.actual_time_gap.num_hours(),
            self.actual_time_gap.num_minutes() % 60,
            self.minimum_required_time.num_hours(),
            self.minimum_required_time.num_minutes() % 60
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{UserId, LocationId};

    #[test]
    fn test_impossible_traveler_metadata_creation() {
        let user_id = UserId::new();
        let primary_location = LocationId::new();
        let remote_location = LocationId::new();
        let distance = 3000.0; // 3000 km
        let actual_gap = Duration::hours(2); // 2 hours
        let required_time = Duration::hours(8); // 8 hours minimum

        let metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            distance,
            actual_gap,
            required_time,
        );

        assert_eq!(metadata.user_id, user_id);
        assert_eq!(metadata.primary_location, primary_location);
        assert_eq!(metadata.remote_location, remote_location);
        assert_eq!(metadata.geographical_distance_km, distance);
        assert_eq!(metadata.actual_time_gap, actual_gap);
        assert_eq!(metadata.minimum_required_time, required_time);
        assert_eq!(metadata.impossibility_factor, 4.0); // 8 hours / 2 hours = 4.0
    }

    #[test]
    fn test_impossibility_detection() {
        let user_id = UserId::new();
        let primary_location = LocationId::new();
        let remote_location = LocationId::new();

        // Impossible scenario
        let impossible_metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            2000.0,
            Duration::hours(1),
            Duration::hours(6),
        );

        assert!(impossible_metadata.is_impossible());
        assert_eq!(impossible_metadata.get_impossibility_severity(), 6.0);

        // Possible scenario
        let possible_metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            100.0,
            Duration::hours(3),
            Duration::hours(2),
        );

        assert!(!possible_metadata.is_impossible());
        assert!(possible_metadata.get_impossibility_severity() < 1.0);
    }

    #[test]
    fn test_time_deficit_calculation() {
        let user_id = UserId::new();
        let primary_location = LocationId::new();
        let remote_location = LocationId::new();

        let metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            1500.0,
            Duration::hours(2),
            Duration::hours(5),
        );

        let deficit = metadata.get_time_deficit();
        assert_eq!(deficit, Duration::hours(3)); // 5 - 2 = 3 hours deficit

        // Test case where no deficit exists
        let no_deficit_metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            100.0,
            Duration::hours(5),
            Duration::hours(2),
        );

        assert_eq!(no_deficit_metadata.get_time_deficit(), Duration::zero());
    }

    #[test]
    fn test_travel_type_classification() {
        let user_id = UserId::new();
        let primary_location = LocationId::new();
        let remote_location = LocationId::new();

        // Regional travel
        let regional_metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            500.0,
            Duration::hours(1),
            Duration::hours(3),
        );

        assert!(!regional_metadata.is_cross_country_travel());
        assert!(!regional_metadata.is_international_travel());

        // Cross-country travel
        let cross_country_metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            1500.0,
            Duration::hours(2),
            Duration::hours(6),
        );

        assert!(cross_country_metadata.is_cross_country_travel());
        assert!(!cross_country_metadata.is_international_travel());

        // International travel
        let international_metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            5000.0,
            Duration::hours(3),
            Duration::hours(12),
        );

        assert!(international_metadata.is_cross_country_travel());
        assert!(international_metadata.is_international_travel());
    }

    #[test]
    fn test_description_generation() {
        let user_id = UserId::new();
        let primary_location = LocationId::new();
        let remote_location = LocationId::new();

        // Possible travel
        let possible_metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            100.0,
            Duration::hours(3),
            Duration::hours(2),
        );

        let description = possible_metadata.get_description();
        assert!(description.contains("physically possible"));

        // Impossible international travel
        let impossible_metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            5000.0,
            Duration::hours(2),
            Duration::hours(12),
        );

        let description = impossible_metadata.get_description();
        assert!(description.contains("Impossible international travel"));
        assert!(description.contains("5000.0 km"));
        assert!(description.contains("2 hours"));
        assert!(description.contains("12 hours"));
    }

    #[test]
    fn test_zero_time_gap_handling() {
        let user_id = UserId::new();
        let primary_location = LocationId::new();
        let remote_location = LocationId::new();

        let metadata = ImpossibleTravelerMetadata::new(
            user_id,
            primary_location,
            remote_location,
            1000.0,
            Duration::zero(),
            Duration::hours(4),
        );

        assert!(metadata.impossibility_factor.is_infinite());
        assert!(metadata.is_impossible());
    }
}
