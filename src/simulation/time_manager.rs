//! Time management and acceleration
//!
//! This module contains time management and time acceleration logic.

// Error handling types available if needed
use crate::types::{BuildingId, LocationId, RoomId};
use chrono::{DateTime, Duration, Timelike, Utc};
use rand::Rng;
use tracing::{debug, info, instrument, warn};

/// Time management system with acceleration for realistic simulation
#[derive(Debug, Clone)]
pub struct TimeManager {
    /// Factor by which time is accelerated (simulated time units per real time unit)
    acceleration_factor: f64,
    /// When the simulation started in simulated time
    simulation_start: DateTime<Utc>,
}

impl TimeManager {
    /// Create a new TimeManager for batch processing
    pub fn new() -> Self {
        info!("Initializing time manager for batch processing");
        Self { acceleration_factor: 1.0, simulation_start: Utc::now() }
    }

    /// Get the current simulated time
    #[instrument(skip(self))]
    pub fn current_simulated_time(&self) -> DateTime<Utc> {
        let elapsed_real_time = Utc::now() - self.simulation_start;
        let simulated_elapsed = Duration::milliseconds(
            (elapsed_real_time.num_milliseconds() as f64 * self.acceleration_factor) as i64,
        );
        let simulated_time = self.simulation_start + simulated_elapsed;

        debug!(
            "Current simulated time: {} (acceleration: {}x)",
            simulated_time, self.acceleration_factor
        );
        simulated_time
    }

    /// Advance the simulation time by a specific duration
    pub fn advance_by(&mut self, duration: Duration) {
        // Adjust the simulation start time backwards to effectively advance current time
        let real_duration_to_subtract = Duration::milliseconds(
            (duration.num_milliseconds() as f64 / self.acceleration_factor) as i64,
        );
        self.simulation_start = self.simulation_start - real_duration_to_subtract;

        debug!(
            "Advanced simulation time by {} (real time adjustment: {})",
            duration, real_duration_to_subtract
        );
    }

    /// Check if the given time is during business hours (9 AM - 5 PM, every day)
    #[instrument(skip(self))]
    pub fn is_business_hours(&self, timestamp: DateTime<Utc>) -> bool {
        let hour = timestamp.hour();

        // Business hours: 9 AM to 5 PM (17:00) - every day is a work day
        (9..17).contains(&hour)
    }



    /// Calculate realistic travel time between two rooms
    ///
    /// This method calculates travel time based on the relationship between rooms:
    /// - Same room: 0 seconds
    /// - Same building: 30 seconds to 3 minutes
    /// - Same location, different building: 2-10 minutes
    /// - Different locations: 4-12 hours (including travel time)
    pub fn calculate_travel_time<R: Rng>(
        &self,
        from_room: Option<RoomId>,
        to_room: RoomId,
        from_building: BuildingId,
        to_building: BuildingId,
        from_location: LocationId,
        to_location: LocationId,
        rng: &mut R,
    ) -> Duration {
        // Same room - no travel time
        if let Some(from) = from_room {
            if from == to_room {
                return Duration::seconds(0);
            }
        }

        // Different geographical locations - significant travel time
        if from_location != to_location {
            // 4-12 hours including travel, transit, and processing time
            let hours = rng.gen_range(4..=12);
            return Duration::hours(hours);
        }

        // Same location, different buildings - walking/shuttle time
        if from_building != to_building {
            // 2-10 minutes walking between buildings
            let minutes = rng.gen_range(2..=10);
            return Duration::minutes(minutes);
        }

        // Same building, different rooms - walking within building
        // 30 seconds to 3 minutes depending on building size and distance
        let seconds = rng.gen_range(30..=180);
        Duration::seconds(seconds)
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rand::thread_rng;

    #[test]
    fn test_time_manager_creation() {
        let tm = TimeManager::new();
        assert_eq!(tm.acceleration_factor, 1.0);
    }

    #[test]
    fn test_default_time_manager() {
        let tm = TimeManager::default();
        assert_eq!(tm.acceleration_factor, 1.0);
    }

    #[test]
    fn test_business_hours_detection() {
        let tm = TimeManager::default();

        // Test business hours (9 AM - 5 PM, every day)
        let monday_10am = Utc.with_ymd_and_hms(2024, 1, 8, 10, 0, 0).unwrap(); // Monday
        let friday_2pm = Utc.with_ymd_and_hms(2024, 1, 12, 14, 0, 0).unwrap(); // Friday
        let saturday_10am = Utc.with_ymd_and_hms(2024, 1, 13, 10, 0, 0).unwrap(); // Saturday
        let monday_8am = Utc.with_ymd_and_hms(2024, 1, 8, 8, 0, 0).unwrap(); // Before business hours
        let monday_6pm = Utc.with_ymd_and_hms(2024, 1, 8, 18, 0, 0).unwrap(); // After business hours

        assert!(tm.is_business_hours(monday_10am));
        assert!(tm.is_business_hours(friday_2pm));
        assert!(tm.is_business_hours(saturday_10am)); // Saturday is now a work day
        assert!(!tm.is_business_hours(monday_8am));
        assert!(!tm.is_business_hours(monday_6pm));
    }

    #[test]
    fn test_travel_time_same_room() {
        let tm = TimeManager::default();
        let mut rng = thread_rng();

        let room_id = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let travel_time = tm.calculate_travel_time(
            Some(room_id),
            room_id,
            building_id,
            building_id,
            location_id,
            location_id,
            &mut rng,
        );

        assert_eq!(travel_time, Duration::seconds(0));
    }

    #[test]
    fn test_travel_time_different_locations() {
        let tm = TimeManager::default();
        let mut rng = thread_rng();

        let room_id1 = RoomId::new();
        let room_id2 = RoomId::new();
        let building_id = BuildingId::new();
        let location_id1 = LocationId::new();
        let location_id2 = LocationId::new();

        let travel_time = tm.calculate_travel_time(
            Some(room_id1),
            room_id2,
            building_id,
            building_id,
            location_id1,
            location_id2,
            &mut rng,
        );

        // Should be between 4-12 hours
        assert!(travel_time >= Duration::hours(4));
        assert!(travel_time <= Duration::hours(12));
    }

    #[test]
    fn test_travel_time_different_buildings() {
        let tm = TimeManager::default();
        let mut rng = thread_rng();

        let room_id1 = RoomId::new();
        let room_id2 = RoomId::new();
        let building_id1 = BuildingId::new();
        let building_id2 = BuildingId::new();
        let location_id = LocationId::new();

        let travel_time = tm.calculate_travel_time(
            Some(room_id1),
            room_id2,
            building_id1,
            building_id2,
            location_id,
            location_id,
            &mut rng,
        );

        // Should be between 2-10 minutes
        assert!(travel_time >= Duration::minutes(2));
        assert!(travel_time <= Duration::minutes(10));
    }

    #[test]
    fn test_travel_time_same_building() {
        let tm = TimeManager::default();
        let mut rng = thread_rng();

        let room_id1 = RoomId::new();
        let room_id2 = RoomId::new();
        let building_id = BuildingId::new();
        let location_id = LocationId::new();

        let travel_time = tm.calculate_travel_time(
            Some(room_id1),
            room_id2,
            building_id,
            building_id,
            location_id,
            location_id,
            &mut rng,
        );

        // Should be between 30 seconds and 3 minutes
        assert!(travel_time >= Duration::seconds(30));
        assert!(travel_time <= Duration::seconds(180));
    }
}
