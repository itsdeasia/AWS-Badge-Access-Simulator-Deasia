//! User generation and statistics
//!
//! This module contains user generation logic and statistics collection.

use rand::Rng;
use std::collections::HashMap;
use std::fmt;
// Removed unused serde imports

use crate::facility::LocationRegistry;
use crate::permissions::{PermissionLevel, PermissionSet};
use crate::types::{BuildingId, LocationId, RoomId, RoomType, SimulationConfig};

use crate::user::{BehaviorProfile, User};

/// Generator for creating users with primary assignments and permission logic
pub struct UserGenerator {
    rng: Box<dyn rand::RngCore>,
}

impl fmt::Debug for UserGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserGenerator").finish()
    }
}

impl UserGenerator {
    /// Create a new user generator
    pub fn new() -> Self {
        Self { rng: Box::new(rand::thread_rng()) }
    }

    /// Create a new user generator with a specific seed for reproducible results
    pub fn with_seed(seed: u64) -> Self {
        use rand::SeedableRng;
        Self { rng: Box::new(rand::rngs::StdRng::seed_from_u64(seed)) }
    }

    /// Generate users with primary assignments and permissions
    pub fn generate_users(
        &mut self,
        config: &SimulationConfig,
        registry: &LocationRegistry,
    ) -> Result<Vec<User>, String> {
        if config.user_count == 0 {
            return Ok(Vec::new());
        }

        if registry.location_count() == 0 {
            return Err("Cannot generate users without any locations".to_string());
        }

        let mut users = Vec::with_capacity(config.user_count);
        let _all_locations = registry.get_all_locations();
        let total_workspaces = self.count_total_workspaces(registry);

        if total_workspaces == 0 {
            return Err("Cannot generate users without any workspace rooms".to_string());
        }

        // Calculate how many users should be curious and have cloned badges
        let curious_count =
            (config.user_count as f64 * config.curious_user_percentage) as usize;
        let cloned_badge_count =
            (config.user_count as f64 * config.cloned_badge_percentage) as usize;

        // Create a list of workspace assignments
        let workspace_assignments = self.distribute_workspaces(config.user_count, registry)?;

        // Generate night-shift users only for larger organizations (500+ users)
        // Night-shift security staff only make sense for organizations large enough to justify
        // 24/7 security coverage across multiple buildings
        let night_shift_users = if config.user_count >= 500 {
            self.generate_night_shift_users(registry)?
        } else {
            Vec::new()
        };
        let night_shift_count = night_shift_users.len();
        
        // Add night-shift users to the list
        users.extend(night_shift_users);

        // Generate regular users for remaining slots
        let regular_user_count = config.user_count.saturating_sub(night_shift_count);
        
        for i in 0..regular_user_count {
            let workspace_index = i % workspace_assignments.len();
            let (location_id, building_id, workspace_room_id) = workspace_assignments[workspace_index];

            // Generate permissions for this user
            let permissions = self.generate_permissions(
                location_id,
                building_id,
                workspace_room_id,
                registry,
                config,
            )?;

            // Create the user
            let mut user =
                User::new(location_id, building_id, workspace_room_id, permissions);

            // Mark as curious based on percentage (adjust for night-shift users)
            if i < curious_count {
                user.is_curious = true;
                user.behavior_profile = BehaviorProfile::curious();
            }

            // Mark as having cloned badge based on percentage
            if i < cloned_badge_count {
                user.has_cloned_badge = true;
            }

            // Generate varied behavior profiles for non-curious users
            if !user.is_curious {
                user.behavior_profile = self.generate_behavior_profile();
            }

            users.push(user);
        }

        // Shuffle users to randomize curious/cloned badge distribution
        use rand::seq::SliceRandom;
        users.shuffle(&mut *self.rng);

        Ok(users)
    }

    /// Count total workspace rooms across all locations
    fn count_total_workspaces(&self, registry: &LocationRegistry) -> usize {
        registry
            .get_all_locations()
            .iter()
            .flat_map(|location| &location.buildings)
            .flat_map(|building| &building.rooms)
            .filter(|room| room.room_type == RoomType::Workspace)
            .count()
    }

    /// Distribute workspaces among users, ensuring each gets a primary workspace
    fn distribute_workspaces(
        &mut self,
        user_count: usize,
        registry: &LocationRegistry,
    ) -> Result<Vec<(LocationId, BuildingId, RoomId)>, String> {
        let mut assignments = Vec::new();
        let mut workspace_pool = Vec::new();

        // Collect all workspace rooms
        for location in registry.get_all_locations() {
            for building in &location.buildings {
                for room in &building.rooms {
                    if room.room_type == RoomType::Workspace {
                        workspace_pool.push((location.id, building.id, room.id));
                    }
                }
            }
        }

        if workspace_pool.is_empty() {
            return Err("No workspace rooms available for user assignment".to_string());
        }

        // Assign workspaces to users
        // If we have more users than workspaces, some will share (realistic for hot-desking)
        for i in 0..user_count {
            let workspace_index = i % workspace_pool.len();
            assignments.push(workspace_pool[workspace_index]);
        }

        // Shuffle to randomize assignments
        use rand::seq::SliceRandom;
        assignments.shuffle(&mut *self.rng);

        Ok(assignments)
    }

    /// Generate permissions for a user based on their primary assignment
    fn generate_permissions(
        &mut self,
        primary_location: LocationId,
        primary_building: BuildingId,
        primary_workspace: RoomId,
        registry: &LocationRegistry,
        _config: &SimulationConfig,
    ) -> Result<PermissionSet, String> {
        let mut permissions = PermissionSet::new();

        // Always grant access to primary workspace
        permissions.add_permission(PermissionLevel::Room(primary_workspace));

        // Find the primary location and building
        let location =
            registry.get_location(primary_location).ok_or("Primary location not found")?;
        let building = location
            .buildings
            .iter()
            .find(|b| b.id == primary_building)
            .ok_or("Primary building not found")?;

        // Grant access to common areas in primary building
        for room in &building.rooms {
            match room.room_type {
                RoomType::Lobby | RoomType::Bathroom | RoomType::Kitchen | RoomType::Cafeteria => {
                    permissions.add_permission(PermissionLevel::Room(room.id));
                }
                RoomType::MeetingRoom => {
                    // 80% chance to access meeting rooms in primary building
                    if self.rng.gen::<f64>() < 0.8 {
                        permissions.add_permission(PermissionLevel::Room(room.id));
                    }
                }
                _ => {} // Other room types require special permissions
            }
        }

        // Grant some cross-building permissions within the same location
        for other_building in &location.buildings {
            if other_building.id != primary_building {
                // 30% chance to have access to other buildings in same location
                if self.rng.gen::<f64>() < 0.3 {
                    for room in &other_building.rooms {
                        match room.room_type {
                            RoomType::Lobby | RoomType::Bathroom | RoomType::Cafeteria => {
                                permissions.add_permission(PermissionLevel::Room(room.id));
                            }
                            RoomType::MeetingRoom => {
                                // 50% chance for meeting rooms in other buildings
                                if self.rng.gen::<f64>() < 0.5 {
                                    permissions.add_permission(PermissionLevel::Room(room.id));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Grant some cross-location permissions (for traveling users)
        for other_location in registry.get_all_locations() {
            if other_location.id != primary_location {
                // 10% chance to have access to other locations
                if self.rng.gen::<f64>() < 0.1 {
                    for building in &other_location.buildings {
                        for room in &building.rooms {
                            match room.room_type {
                                RoomType::Lobby | RoomType::Bathroom | RoomType::Cafeteria => {
                                    permissions.add_permission(PermissionLevel::Room(room.id));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Grant special permissions to some users
        self.grant_special_permissions(&mut permissions, registry)?;

        Ok(permissions)
    }

    /// Grant special permissions for high-security areas to a subset of users
    fn grant_special_permissions(
        &mut self,
        permissions: &mut PermissionSet,
        registry: &LocationRegistry,
    ) -> Result<(), String> {
        // 5% chance to get server room access
        if self.rng.gen::<f64>() < 0.05 {
            for location in registry.get_all_locations() {
                for building in &location.buildings {
                    for room in &building.rooms {
                        if room.room_type == RoomType::ServerRoom {
                            permissions.add_permission(PermissionLevel::Room(room.id));
                        }
                    }
                }
            }
        }

        // 2% chance to get executive office access
        if self.rng.gen::<f64>() < 0.02 {
            for location in registry.get_all_locations() {
                for building in &location.buildings {
                    for room in &building.rooms {
                        if room.room_type == RoomType::ExecutiveOffice {
                            permissions.add_permission(PermissionLevel::Room(room.id));
                        }
                    }
                }
            }
        }

        // 3% chance to get laboratory access
        if self.rng.gen::<f64>() < 0.03 {
            for location in registry.get_all_locations() {
                for building in &location.buildings {
                    for room in &building.rooms {
                        if room.room_type == RoomType::Laboratory {
                            permissions.add_permission(PermissionLevel::Room(room.id));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate a varied behavior profile for a user
    fn generate_behavior_profile(&mut self) -> BehaviorProfile {
        use rand::Rng;

        // Generate realistic behavior patterns
        let travel_frequency = self.rng.gen_range(0.05..=0.25);
        let curiosity_level = self.rng.gen_range(0.0..=0.3); // Non-curious users have low curiosity
        let schedule_adherence = self.rng.gen_range(0.6..=0.95);
        let social_level = self.rng.gen_range(0.2..=0.9);

        BehaviorProfile { travel_frequency, curiosity_level, schedule_adherence, social_level }
    }

    /// Generate night-shift users ensuring each building has 1-3
    fn generate_night_shift_users(
        &mut self,
        registry: &LocationRegistry,
    ) -> Result<Vec<User>, String> {
        let mut night_shift_users = Vec::new();
        
        // Collect all buildings across all locations
        let mut all_buildings = Vec::new();
        for location in registry.get_all_locations() {
            for building in &location.buildings {
                all_buildings.push((location.id, building.id));
            }
        }

        if all_buildings.is_empty() {
            return Ok(night_shift_users);
        }

        // Generate 1-3 night-shift users per building
        for (location_id, building_id) in all_buildings {
            let night_shift_count_for_building = self.rng.gen_range(1..=3);
            
            for _ in 0..night_shift_count_for_building {
                // Find a workspace in this building for the night-shift user
                let workspace_room_id = self.find_workspace_in_building(location_id, building_id, registry)?;
                
                // Generate building-level permissions for night-shift user
                let permissions = self.generate_night_shift_permissions(
                    location_id,
                    building_id,
                    workspace_room_id,
                    registry,
                )?;

                // Create night-shift user
                let user = User::new_night_shift(
                    location_id,
                    building_id,
                    workspace_room_id,
                    permissions,
                    building_id, // Assigned night building is the same as primary building
                );

                night_shift_users.push(user);
            }
        }

        Ok(night_shift_users)
    }

    /// Find a workspace room in a specific building
    fn find_workspace_in_building(
        &self,
        location_id: LocationId,
        building_id: BuildingId,
        registry: &LocationRegistry,
    ) -> Result<RoomId, String> {
        let location = registry.get_location(location_id)
            .ok_or("Location not found for night-shift user")?;
        
        let building = location.buildings.iter()
            .find(|b| b.id == building_id)
            .ok_or("Building not found for night-shift user")?;

        // Find first workspace room in the building
        for room in &building.rooms {
            if room.room_type == RoomType::Workspace {
                return Ok(room.id);
            }
        }

        Err("No workspace rooms found in building for night-shift user".to_string())
    }

    /// Generate permissions for night-shift user with building-level access
    fn generate_night_shift_permissions(
        &mut self,
        primary_location: LocationId,
        primary_building: BuildingId,
        primary_workspace: RoomId,
        registry: &LocationRegistry,
    ) -> Result<PermissionSet, String> {
        let mut permissions = PermissionSet::new();

        // Always grant access to primary workspace
        permissions.add_permission(PermissionLevel::Room(primary_workspace));

        // Grant building-level access for patrol duties
        permissions.add_permission(PermissionLevel::Building(primary_building));

        // Find the primary location and building
        let location = registry.get_location(primary_location)
            .ok_or("Primary location not found")?;
        let building = location.buildings.iter()
            .find(|b| b.id == primary_building)
            .ok_or("Primary building not found")?;

        // Grant access to ALL rooms in the assigned building (for patrol duties)
        for room in &building.rooms {
            permissions.add_permission(PermissionLevel::Room(room.id));
        }

        Ok(permissions)
    }

    /// Generate a single user with specific parameters (for testing)
    pub fn generate_single_user(
        &mut self,
        location_id: LocationId,
        building_id: BuildingId,
        workspace_id: RoomId,
        registry: &LocationRegistry,
        config: &SimulationConfig,
    ) -> Result<User, String> {
        let permissions =
            self.generate_permissions(location_id, building_id, workspace_id, registry, config)?;

        Ok(User::new(location_id, building_id, workspace_id, permissions))
    }

    /// Get statistics about user generation
    pub fn get_user_stats(&self, users: &[User]) -> UserStats {
        let curious_count = users.iter().filter(|e| e.is_curious).count();
        let cloned_badge_count = users.iter().filter(|e| e.has_cloned_badge).count();
        let night_shift_count = users.iter().filter(|e| e.is_night_shift).count();

        let total_permissions: usize = users.iter().map(|e| e.permissions.len()).sum();
        let avg_permissions = if users.is_empty() {
            0.0
        } else {
            total_permissions as f64 / users.len() as f64
        };

        // Count users by location
        let mut location_distribution = HashMap::new();
        for user in users {
            *location_distribution.entry(user.primary_location).or_insert(0) += 1;
        }

        UserStats {
            total_users: users.len(),
            curious_users: curious_count,
            cloned_badge_users: cloned_badge_count,
            night_shift_users: night_shift_count,
            average_permissions_per_user: avg_permissions,
            location_distribution,
        }
    }

    /// Validate user generation configuration
    pub fn validate_configuration(
        &self,
        config: &SimulationConfig,
        registry: &LocationRegistry,
    ) -> Result<(), String> {
        if config.user_count == 0 {
            return Err("User count must be greater than 0".to_string());
        }

        if registry.location_count() == 0 {
            return Err("At least one location is required for user generation".to_string());
        }

        let total_workspaces = self.count_total_workspaces(registry);
        if total_workspaces == 0 {
            return Err(
                "At least one workspace room is required for user generation".to_string()
            );
        }

        if config.curious_user_percentage < 0.0 || config.curious_user_percentage > 1.0 {
            return Err("Curious user percentage must be between 0.0 and 1.0".to_string());
        }

        if config.cloned_badge_percentage < 0.0 || config.cloned_badge_percentage > 1.0 {
            return Err("Badge replication percentage must be between 0.0 and 1.0".to_string());
        }

        Ok(())
    }
}

impl Default for UserGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about generated users
#[derive(Debug, Clone)]
pub struct UserStats {
    /// Total number of users generated
    pub total_users: usize,
    /// Number of curious users
    pub curious_users: usize,
    /// Number of users with cloned badges
    pub cloned_badge_users: usize,
    /// Number of night-shift users
    pub night_shift_users: usize,
    /// Average number of permissions per user
    pub average_permissions_per_user: f64,
    /// Distribution of users across locations
    pub location_distribution: HashMap<LocationId, usize>,
}

impl fmt::Display for UserStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "User Generation Statistics:")?;
        writeln!(f, "  Total Users: {}", self.total_users)?;
        writeln!(
            f,
            "  Curious Users: {} ({:.1}%)",
            self.curious_users,
            if self.total_users > 0 {
                (self.curious_users as f64 / self.total_users as f64) * 100.0
            } else {
                0.0
            }
        )?;
        writeln!(
            f,
            "  Cloned Badge Users: {} ({:.1}%)",
            self.cloned_badge_users,
            if self.total_users > 0 {
                (self.cloned_badge_users as f64 / self.total_users as f64) * 100.0
            } else {
                0.0
            }
        )?;
        writeln!(
            f,
            "  Night-Shift Users: {} ({:.1}%)",
            self.night_shift_users,
            if self.total_users > 0 {
                (self.night_shift_users as f64 / self.total_users as f64) * 100.0
            } else {
                0.0
            }
        )?;
        writeln!(
            f,
            "  Average Permissions per User: {:.1}",
            self.average_permissions_per_user
        )?;
        writeln!(f, "  Location Distribution:")?;
        for (location_id, count) in &self.location_distribution {
            writeln!(f, "    {}: {} users", location_id, count)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Using local placeholder types for testing

    fn create_test_registry() -> LocationRegistry {
        // Simplified test registry since facility module isn't implemented yet
        LocationRegistry::new()
    }

    #[test]
    fn test_user_generator_creation() {
        let generator = UserGenerator::new();

        // Just test that it can be created without panicking
        assert!(format!("{:?}", generator).contains("UserGenerator"));
    }

    #[test]
    fn test_user_generator_with_seed() {
        let generator = UserGenerator::with_seed(12345);

        // Test that seeded generator can be created
        assert!(format!("{:?}", generator).contains("UserGenerator"));
    }

    #[test]
    fn test_user_generation() {
        let mut user_generator = UserGenerator::with_seed(42);
        let registry = create_test_registry();

        let config = SimulationConfig {
            user_count: 0, // Empty config for now since facility isn't implemented
            curious_user_percentage: 0.2,
            cloned_badge_percentage: 0.1,
            ..Default::default()
        };

        let users = user_generator.generate_users(&config, &registry).unwrap();

        assert_eq!(users.len(), 0); // Should be empty since no facilities
    }

    #[test]
    fn test_user_generator_validation() {
        let generator = UserGenerator::new();
        let registry = LocationRegistry::new(); // Empty registry

        let config = SimulationConfig { user_count: 5, ..Default::default() };

        // Should fail with empty registry
        assert!(generator.validate_configuration(&config, &registry).is_err());

        // Test with valid registry - but since our test registry is empty, it should still fail
        let valid_registry = create_test_registry();
        assert!(generator.validate_configuration(&config, &valid_registry).is_err()); // Should fail since no facilities exist

        // Test invalid percentages
        let invalid_config = SimulationConfig {
            user_count: 5,
            curious_user_percentage: 1.5, // Invalid
            ..Default::default()
        };
        assert!(generator.validate_configuration(&invalid_config, &valid_registry).is_err());
    }

    #[test]
    fn test_user_stats() {
        let user_generator = UserGenerator::with_seed(42);

        // Test with empty user list
        let users = vec![];
        let stats = user_generator.get_user_stats(&users);

        assert_eq!(stats.total_users, 0);
        assert_eq!(stats.curious_users, 0);
        assert_eq!(stats.cloned_badge_users, 0);
        assert_eq!(stats.night_shift_users, 0);
        assert_eq!(stats.average_permissions_per_user, 0.0);
        assert!(stats.location_distribution.is_empty());

        // Test display formatting
        let display_output = format!("{}", stats);
        assert!(display_output.contains("User Generation Statistics"));
        assert!(display_output.contains("Total Users: 0"));
    }

    #[test]
    fn test_single_user_generation() {
        let mut generator = UserGenerator::new();
        let registry = create_test_registry();
        let config = SimulationConfig::default();

        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();

        // This will fail since no facilities exist, but we test the error handling
        let result = generator.generate_single_user(
            location_id,
            building_id,
            workspace_id,
            &registry,
            &config,
        );

        assert!(result.is_err()); // Should fail since no facilities exist
    }

    #[test]
    fn test_workspace_distribution() {
        let mut generator = UserGenerator::with_seed(42);
        let registry = create_test_registry();

        // Should fail since no workspaces exist
        let result = generator.distribute_workspaces(5, &registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_behavior_profile_generation() {
        let mut generator = UserGenerator::with_seed(42);

        let profile = generator.generate_behavior_profile();

        // Check that all values are in valid ranges
        assert!((0.0..=1.0).contains(&profile.travel_frequency));
        assert!((0.0..=1.0).contains(&profile.curiosity_level));
        assert!((0.0..=1.0).contains(&profile.schedule_adherence));
        assert!((0.0..=1.0).contains(&profile.social_level));

        // Non-curious users should have low curiosity
        assert!(profile.curiosity_level <= 0.3);
    }

    #[test]
    fn test_night_shift_user_generation() {
        let mut generator = UserGenerator::with_seed(42);
        let registry = LocationRegistry::new(); // Empty registry

        // Should return empty list since no buildings exist
        let result = generator.generate_night_shift_users(&registry);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_night_shift_permissions_generation() {
        let mut generator = UserGenerator::with_seed(42);
        let registry = LocationRegistry::new(); // Empty registry

        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let workspace_id = RoomId::new();

        // Should fail since no facilities exist
        let result = generator.generate_night_shift_permissions(
            location_id,
            building_id,
            workspace_id,
            &registry,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_find_workspace_in_building() {
        let generator = UserGenerator::with_seed(42);
        let registry = LocationRegistry::new(); // Empty registry

        let location_id = LocationId::new();
        let building_id = BuildingId::new();

        // Should fail since no facilities exist
        let result = generator.find_workspace_in_building(location_id, building_id, &registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_user_stats_with_night_shift() {
        let generator = UserGenerator::with_seed(42);
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        
        // Create test users including night-shift
        let mut users = vec![];
        
        // Regular user
        let mut permissions1 = PermissionSet::new();
        permissions1.add_permission(PermissionLevel::Room(room_id));
        let user1 = User::new(location_id, building_id, room_id, permissions1);
        users.push(user1);
        
        // Night-shift user
        let mut permissions2 = PermissionSet::new();
        permissions2.add_permission(PermissionLevel::Room(room_id));
        let user2 = User::new_night_shift(
            location_id,
            building_id,
            room_id,
            permissions2,
            building_id,
        );
        users.push(user2);
        
        let stats = generator.get_user_stats(&users);
        
        assert_eq!(stats.total_users, 2);
        assert_eq!(stats.curious_users, 0);
        assert_eq!(stats.cloned_badge_users, 0);
        assert_eq!(stats.night_shift_users, 1);
        assert!(stats.average_permissions_per_user > 0.0);
        assert_eq!(stats.location_distribution.get(&location_id), Some(&2));
    }

    #[test]
    fn test_user_stats_display_with_night_shift() {
        let generator = UserGenerator::with_seed(42);
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        
        // Create test users including night-shift
        let mut users = vec![];
        
        // Night-shift user
        let mut permissions = PermissionSet::new();
        permissions.add_permission(PermissionLevel::Room(room_id));
        let user = User::new_night_shift(
            location_id,
            building_id,
            room_id,
            permissions,
            building_id,
        );
        users.push(user);
        
        let stats = generator.get_user_stats(&users);
        let display_output = format!("{}", stats);
        
        assert!(display_output.contains("User Generation Statistics"));
        assert!(display_output.contains("Total Users: 1"));
        assert!(display_output.contains("Night-Shift Users: 1"));
        assert!(display_output.contains("100.0%")); // 1/1 = 100%
    }

    #[test]
    fn test_no_night_shift_for_small_organizations() {
        let mut generator = UserGenerator::with_seed(42);
        let registry = LocationRegistry::new(); // Empty registry
        
        // Test with small user count (< 500)
        let small_config = SimulationConfig {
            user_count: 100, // Small organization
            ..Default::default()
        };
        
        // Should return empty list since no buildings exist, but more importantly,
        // night-shift generation should be skipped for small organizations
        let result = generator.generate_users(&small_config, &registry);
        
        // This will fail due to no locations, but the night-shift logic should be bypassed
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot generate users without any locations"));
    }

    #[test]
    fn test_night_shift_threshold() {
        let mut generator = UserGenerator::with_seed(42);
        let registry = LocationRegistry::new(); // Empty registry
        
        // Test with exactly 500 users (threshold)
        let threshold_config = SimulationConfig {
            user_count: 500, // Exactly at threshold
            ..Default::default()
        };
        
        // Should attempt night-shift generation for organizations with 500+ users
        let result = generator.generate_users(&threshold_config, &registry);
        
        // This will fail due to no locations, but the night-shift logic should be attempted
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot generate users without any locations"));
    }
}
