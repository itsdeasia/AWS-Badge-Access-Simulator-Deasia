//! User behavior profiles and activity preferences
//!
//! This module contains behavior modeling for users including activity preferences.

use serde::{Deserialize, Serialize};

/// Behavior profile for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorProfile {
    /// How often the user travels to different locations (0.0-1.0)
    pub travel_frequency: f64,
    /// How curious the user is about unauthorized areas (0.0-1.0)
    pub curiosity_level: f64,
    /// How strictly the user follows schedules (0.0-1.0)
    pub schedule_adherence: f64,
    /// How social the user is (affects meeting frequency) (0.0-1.0)
    pub social_level: f64,
}

impl BehaviorProfile {
    /// Create a new behavior profile with default values
    pub fn new() -> Self {
        Self {
            travel_frequency: 0.1,
            curiosity_level: 0.0,
            schedule_adherence: 0.8,
            social_level: 0.5,
        }
    }

    /// Create a behavior profile for a curious user
    pub fn curious() -> Self {
        Self {
            travel_frequency: 0.15,
            curiosity_level: 0.7,
            schedule_adherence: 0.6,
            social_level: 0.6,
        }
    }

    /// Create a behavior profile for a highly social user
    pub fn social() -> Self {
        Self {
            travel_frequency: 0.2,
            curiosity_level: 0.1,
            schedule_adherence: 0.7,
            social_level: 0.9,
        }
    }

    /// Create a behavior profile for a focused/introverted user
    pub fn focused() -> Self {
        Self {
            travel_frequency: 0.05,
            curiosity_level: 0.0,
            schedule_adherence: 0.95,
            social_level: 0.2,
        }
    }

    /// Create a behavior profile for a night-shift user
    pub fn night_shift() -> Self {
        Self {
            travel_frequency: 0.05,  // Stay in assigned building
            curiosity_level: 0.1,    // Low curiosity (security-focused)
            schedule_adherence: 0.9, // High adherence to patrol schedule
            social_level: 0.2,       // Low social interaction during night
        }
    }

    /// Check if this user is likely to attempt unauthorized access
    pub fn is_curious(&self) -> bool {
        self.curiosity_level > 0.5
    }

    /// Check if this user travels frequently
    pub fn travels_frequently(&self) -> bool {
        self.travel_frequency > 0.15
    }

    /// Check if this user is highly social
    pub fn is_social(&self) -> bool {
        self.social_level > 0.7
    }

    /// Check if this user is schedule-focused
    pub fn is_schedule_focused(&self) -> bool {
        self.schedule_adherence > 0.8
    }
}

impl Default for BehaviorProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Activity preferences that define how a user typically behaves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPreferences {
    /// Typical arrival time range (hour of day, 0-23)
    pub typical_arrival_hour_range: (u32, u32),
    /// Typical departure time range (hour of day, 0-23)
    pub typical_departure_hour_range: (u32, u32),
    /// Average number of meetings per day
    pub average_meetings_per_day: f64,
    /// Average number of bathroom breaks per day
    pub average_bathroom_breaks_per_day: f64,
    /// Preferred lunch time range (hour of day, 0-23)
    pub preferred_lunch_hour_range: (u32, u32),

}

impl ActivityPreferences {
    /// Create activity preferences from a behavior profile
    pub fn from_behavior_profile(profile: &BehaviorProfile) -> Self {
        // Base arrival time on schedule adherence
        let arrival_base = if profile.schedule_adherence > 0.8 { 8 } else { 9 };
        let arrival_variance = if profile.schedule_adherence > 0.8 { 1 } else { 2 };

        // Base departure time on social level and schedule adherence
        let departure_base = if profile.social_level > 0.7 { 17 } else { 16 };
        let departure_variance = if profile.schedule_adherence > 0.8 { 1 } else { 2 };

        Self {
            typical_arrival_hour_range: (arrival_base, arrival_base + arrival_variance),
            typical_departure_hour_range: (departure_base, departure_base + departure_variance),
            average_meetings_per_day: profile.social_level * 4.0 + 1.0, // 1-5 meetings based on social level
            average_bathroom_breaks_per_day: 2.5, // Standard for most users
            preferred_lunch_hour_range: (11, 14), // 11 AM to 2 PM
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_profile_creation() {
        let default_profile = BehaviorProfile::new();
        assert_eq!(default_profile.travel_frequency, 0.1);
        assert_eq!(default_profile.curiosity_level, 0.0);
        assert_eq!(default_profile.schedule_adherence, 0.8);
        assert_eq!(default_profile.social_level, 0.5);
        assert!(!default_profile.is_curious());

        let curious_profile = BehaviorProfile::curious();
        assert!(curious_profile.is_curious());
        assert!(curious_profile.curiosity_level > 0.5);

        let social_profile = BehaviorProfile::social();
        assert!(social_profile.is_social());
        assert!(social_profile.social_level > 0.7);

        let focused_profile = BehaviorProfile::focused();
        assert!(focused_profile.is_schedule_focused());
        assert!(focused_profile.schedule_adherence > 0.8);

        let night_shift_profile = BehaviorProfile::night_shift();
        assert_eq!(night_shift_profile.travel_frequency, 0.05);
        assert_eq!(night_shift_profile.curiosity_level, 0.1);
        assert_eq!(night_shift_profile.schedule_adherence, 0.9);
        assert_eq!(night_shift_profile.social_level, 0.2);
        assert!(!night_shift_profile.is_curious()); // 0.1 < 0.5 threshold
        assert!(!night_shift_profile.travels_frequently()); // 0.05 < 0.15 threshold
        assert!(!night_shift_profile.is_social()); // 0.2 < 0.7 threshold
        assert!(night_shift_profile.is_schedule_focused()); // 0.9 > 0.8 threshold
    }

    #[test]
    fn test_behavior_profile_characteristics() {
        let mut profile = BehaviorProfile::new();

        // Test curiosity
        profile.curiosity_level = 0.6;
        assert!(profile.is_curious());

        profile.curiosity_level = 0.4;
        assert!(!profile.is_curious());

        // Test travel frequency
        profile.travel_frequency = 0.2;
        assert!(profile.travels_frequently());

        profile.travel_frequency = 0.1;
        assert!(!profile.travels_frequently());

        // Test social level
        profile.social_level = 0.8;
        assert!(profile.is_social());

        profile.social_level = 0.6;
        assert!(!profile.is_social());

        // Test schedule adherence
        profile.schedule_adherence = 0.9;
        assert!(profile.is_schedule_focused());

        profile.schedule_adherence = 0.7;
        assert!(!profile.is_schedule_focused());
    }

    #[test]
    fn test_activity_preferences_from_behavior_profile() {
        let focused_profile = BehaviorProfile::focused();
        let preferences = ActivityPreferences::from_behavior_profile(&focused_profile);

        // Focused users should arrive early and have fewer meetings
        assert_eq!(preferences.typical_arrival_hour_range, (8, 9));
        assert!(preferences.average_meetings_per_day < 2.0);

        let social_profile = BehaviorProfile::social();
        let social_preferences = ActivityPreferences::from_behavior_profile(&social_profile);

        // Social users should have more meetings and later departure
        assert!(social_preferences.average_meetings_per_day > 4.0);
        assert_eq!(social_preferences.typical_departure_hour_range, (17, 19));
    }

    #[test]
    fn test_night_shift_behavior_profile() {
        let night_shift_profile = BehaviorProfile::night_shift();

        // Night-shift users should stay in their assigned building
        assert_eq!(night_shift_profile.travel_frequency, 0.05);
        assert!(!night_shift_profile.travels_frequently());

        // Night-shift users should be security-focused with low curiosity
        assert_eq!(night_shift_profile.curiosity_level, 0.1);
        assert!(!night_shift_profile.is_curious());

        // Night-shift users should have high schedule adherence for patrol duties
        assert_eq!(night_shift_profile.schedule_adherence, 0.9);
        assert!(night_shift_profile.is_schedule_focused());

        // Night-shift users should have low social interaction during night hours
        assert_eq!(night_shift_profile.social_level, 0.2);
        assert!(!night_shift_profile.is_social());
    }

    #[test]
    fn test_night_shift_activity_preferences() {
        let night_shift_profile = BehaviorProfile::night_shift();
        let preferences = ActivityPreferences::from_behavior_profile(&night_shift_profile);

        // Night-shift users should have early arrival (high schedule adherence)
        assert_eq!(preferences.typical_arrival_hour_range, (8, 9));

        // Night-shift users should have fewer meetings due to low social level
        assert!(preferences.average_meetings_per_day < 2.0);
    }

    #[test]
    fn test_behavior_profile_default() {
        let default_profile = BehaviorProfile::default();
        let new_profile = BehaviorProfile::new();

        assert_eq!(default_profile.travel_frequency, new_profile.travel_frequency);
        assert_eq!(default_profile.curiosity_level, new_profile.curiosity_level);
        assert_eq!(default_profile.schedule_adherence, new_profile.schedule_adherence);
        assert_eq!(default_profile.social_level, new_profile.social_level);
    }
}
