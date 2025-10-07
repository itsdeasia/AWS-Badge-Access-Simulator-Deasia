//! Facility generation system
//!
//! This module contains generators for creating realistic facility structures
//! including locations, buildings, and rooms with proper relationships and
//! realistic geographical distribution.

use crate::facility::{
    building::Building, location::Location, registry::LocationRegistry, room::Room,
};
use crate::types::{LocationId, RoomType, SecurityLevel, SimulationConfig};
use rand::{prelude::*, rngs::StdRng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Generator for creating geographical locations with realistic coordinates
pub struct LocationGenerator {
    rng: Box<dyn RngCore>,
    used_coordinates: Vec<(f64, f64)>,
}

impl fmt::Debug for LocationGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocationGenerator")
            .field("used_coordinates", &self.used_coordinates)
            .finish()
    }
}

impl LocationGenerator {
    /// Create a new location generator
    pub fn new() -> Self {
        Self { rng: Box::new(thread_rng()), used_coordinates: Vec::new() }
    }

    /// Create a new location generator with a specific seed
    pub fn with_seed(seed: u64) -> Self {
        Self { rng: Box::new(StdRng::seed_from_u64(seed)), used_coordinates: Vec::new() }
    }

    /// Generate a single location with realistic coordinates
    pub fn generate_location(&mut self, name: String) -> Location {
        let coordinates = self.generate_realistic_coordinates();
        self.used_coordinates.push(coordinates);
        Location::new(name, coordinates)
    }

    /// Generate multiple locations with minimum distance between them
    pub fn generate_locations(&mut self, count: usize) -> Vec<Location> {
        let mut locations = Vec::with_capacity(count);

        for i in 0..count {
            let name = self.generate_location_name(i);
            let location = self.generate_location(name);
            locations.push(location);
        }

        locations
    }

    /// Generate realistic geographical coordinates (avoiding oceans and extreme locations)
    fn generate_realistic_coordinates(&mut self) -> (f64, f64) {
        // Define realistic coordinate ranges for major populated areas
        let coordinate_ranges = [
            // North America
            (25.0, 49.0, -125.0, -66.0),
            // Europe
            (36.0, 71.0, -10.0, 40.0),
            // Asia-Pacific
            (-45.0, 55.0, 95.0, 180.0),
            // South America
            (-55.0, 12.0, -82.0, -35.0),
            // Africa
            (-35.0, 37.0, -18.0, 52.0),
            // Australia
            (-45.0, -10.0, 113.0, 154.0),
        ];

        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 100;
        const MIN_DISTANCE_KM: f64 = 100.0; // Minimum 100km between locations

        loop {
            attempts += 1;

            // Select a random coordinate range
            let (min_lat, max_lat, min_lon, max_lon) =
                coordinate_ranges[self.rng.gen_range(0..coordinate_ranges.len())];

            let lat = self.rng.gen_range(min_lat..=max_lat);
            let lon = self.rng.gen_range(min_lon..=max_lon);
            let coordinates = (lat, lon);

            // Check minimum distance from existing locations
            if self.is_valid_distance(coordinates, MIN_DISTANCE_KM) || attempts >= MAX_ATTEMPTS {
                return coordinates;
            }
        }
    }

    /// Check if coordinates are at least min_distance_km away from existing locations
    fn is_valid_distance(&self, coordinates: (f64, f64), min_distance_km: f64) -> bool {
        for &existing in &self.used_coordinates {
            let distance = self.calculate_distance(coordinates, existing);
            if distance < min_distance_km {
                return false;
            }
        }
        true
    }

    /// Calculate distance between two coordinate points using Haversine formula
    fn calculate_distance(&self, coord1: (f64, f64), coord2: (f64, f64)) -> f64 {
        let (lat1, lon1) = coord1;
        let (lat2, lon2) = coord2;

        let r = 6371.0; // Earth's radius in kilometers
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }

    /// Generate a realistic location name
    fn generate_location_name(&mut self, index: usize) -> String {
        let city_names = [
            "Seattle",
            "Portland",
            "San Francisco",
            "Los Angeles",
            "Denver",
            "Chicago",
            "Austin",
            "Dallas",
            "Atlanta",
            "Miami",
            "Boston",
            "New York",
            "Philadelphia",
            "Washington DC",
            "Toronto",
            "Vancouver",
            "London",
            "Paris",
            "Berlin",
            "Amsterdam",
            "Stockholm",
            "Copenhagen",
            "Dublin",
            "Madrid",
            "Rome",
            "Zurich",
            "Vienna",
            "Tokyo",
            "Seoul",
            "Singapore",
            "Sydney",
            "Melbourne",
            "Mumbai",
            "Bangalore",
            "Tel Aviv",
            "Dubai",
            "São Paulo",
            "Mexico City",
            "Buenos Aires",
            "Cape Town",
        ];

        if index < city_names.len() {
            format!("{} Office", city_names[index])
        } else {
            format!("Location {}", index + 1)
        }
    }
}

impl Default for LocationGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generator for creating buildings with proper lobby rooms
pub struct BuildingGenerator {
    rng: Box<dyn RngCore>,
}

impl fmt::Debug for BuildingGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuildingGenerator").finish()
    }
}

impl BuildingGenerator {
    /// Create a new building generator
    pub fn new() -> Self {
        Self { rng: Box::new(thread_rng()) }
    }

    /// Create a new building generator with a specific seed
    pub fn with_seed(seed: u64) -> Self {
        Self { rng: Box::new(StdRng::seed_from_u64(seed)) }
    }

    /// Generate a single building with the specified number of rooms
    pub fn generate_building(
        &mut self,
        location_id: LocationId,
        name: String,
        room_count: usize,
    ) -> Building {
        let mut building = Building::new(location_id, name);

        // Always create a lobby first (required for building access)
        let lobby = Room::new(
            building.id,
            "Main Lobby".to_string(),
            RoomType::Lobby,
            SecurityLevel::Public,
        );
        building.add_room(lobby);

        // Generate remaining rooms
        let remaining_rooms = room_count.saturating_sub(1); // Subtract 1 for the lobby
        for i in 0..remaining_rooms {
            let room = self.generate_room(building.id, i + 1);
            building.add_room(room);
        }

        building
    }

    /// Generate multiple buildings for a location
    pub fn generate_buildings(
        &mut self,
        location_id: LocationId,
        building_count: usize,
        min_rooms_per_building: usize,
        max_rooms_per_building: usize,
    ) -> Vec<Building> {
        let mut buildings = Vec::with_capacity(building_count);

        for i in 0..building_count {
            let name = self.generate_building_name(i);
            let room_count = self.rng.gen_range(min_rooms_per_building..=max_rooms_per_building);
            let building = self.generate_building(location_id, name, room_count);
            buildings.push(building);
        }

        buildings
    }

    /// Generate a room for a building
    fn generate_room(&mut self, building_id: crate::types::BuildingId, room_number: usize) -> Room {
        let room_type = self.select_room_type();
        let security_level = self.select_security_level(&room_type);
        let name = self.generate_room_name(&room_type, room_number);

        let room = Room::new(building_id, name, room_type, security_level);

        // Add intermediate access requirements for high-security rooms
        if matches!(security_level, SecurityLevel::HighSecurity | SecurityLevel::MaxSecurity) {
            // High-security rooms might require going through security checkpoints
            // For now, we'll leave this empty but the structure is ready for future enhancement
        }

        room
    }

    /// Select a room type based on realistic distribution
    fn select_room_type(&mut self) -> RoomType {
        let rand_val = self.rng.gen::<f64>();

        // Realistic distribution of room types in an office building
        match rand_val {
            x if x < 0.40 => RoomType::Workspace,       // 40% - Most common
            x if x < 0.55 => RoomType::MeetingRoom,     // 15% - Meeting rooms
            x if x < 0.65 => RoomType::Bathroom,        // 10% - Bathrooms
            x if x < 0.75 => RoomType::Kitchen,         // 10% - Break rooms/kitchens
            x if x < 0.85 => RoomType::Storage,         // 10% - Storage areas
            x if x < 0.92 => RoomType::Cafeteria,       // 7% - Dining areas
            x if x < 0.96 => RoomType::ExecutiveOffice, // 4% - Executive offices
            x if x < 0.98 => RoomType::ServerRoom,      // 2% - Technical areas
            _ => RoomType::Laboratory,                  // 2% - Labs/special areas
        }
    }

    /// Select security level based on room type
    fn select_security_level(&mut self, room_type: &RoomType) -> SecurityLevel {
        match room_type {
            RoomType::Lobby => SecurityLevel::Public,
            RoomType::Bathroom | RoomType::Cafeteria | RoomType::Kitchen => SecurityLevel::Public,
            RoomType::Workspace | RoomType::MeetingRoom => SecurityLevel::Standard,
            RoomType::Storage => SecurityLevel::Standard,
            RoomType::ExecutiveOffice => SecurityLevel::Restricted,
            RoomType::ServerRoom | RoomType::Laboratory => {
                // Server rooms and labs can be high security or max security
                if self.rng.gen::<f64>() < 0.7 {
                    SecurityLevel::HighSecurity
                } else {
                    SecurityLevel::MaxSecurity
                }
            }
        }
    }

    /// Generate a realistic room name
    fn generate_room_name(&mut self, room_type: &RoomType, room_number: usize) -> String {
        match room_type {
            RoomType::Lobby => "Main Lobby".to_string(),
            RoomType::Workspace => {
                let workspace_types = ["Open Office", "Workspace", "Desk Area", "Team Space"];
                let workspace_type = workspace_types[self.rng.gen_range(0..workspace_types.len())];
                format!("{} {}", workspace_type, room_number)
            }
            RoomType::MeetingRoom => {
                let meeting_names = [
                    "Conference Room",
                    "Meeting Room",
                    "Boardroom",
                    "Discussion Room",
                    "Collaboration Space",
                    "Video Conference Room",
                    "Training Room",
                ];
                let meeting_name = meeting_names[self.rng.gen_range(0..meeting_names.len())];
                format!("{} {}", meeting_name, room_number)
            }
            RoomType::Bathroom => {
                format!("Restroom {}", room_number)
            }
            RoomType::Cafeteria => {
                let cafeteria_names = ["Cafeteria", "Dining Hall", "Food Court", "Lunch Room"];
                let cafeteria_name = cafeteria_names[self.rng.gen_range(0..cafeteria_names.len())];
                format!("{} {}", cafeteria_name, room_number)
            }
            RoomType::Kitchen => {
                let kitchen_names = ["Break Room", "Kitchen", "Pantry", "Coffee Station"];
                let kitchen_name = kitchen_names[self.rng.gen_range(0..kitchen_names.len())];
                format!("{} {}", kitchen_name, room_number)
            }
            RoomType::ServerRoom => {
                let server_names = ["Server Room", "Data Center", "Network Room", "IT Closet"];
                let server_name = server_names[self.rng.gen_range(0..server_names.len())];
                format!("{} {}", server_name, room_number)
            }
            RoomType::ExecutiveOffice => {
                let exec_titles = ["Executive Office", "Director Office", "VP Office", "C-Suite"];
                let exec_title = exec_titles[self.rng.gen_range(0..exec_titles.len())];
                format!("{} {}", exec_title, room_number)
            }
            RoomType::Storage => {
                let storage_names = ["Storage Room", "Supply Closet", "Archive Room", "Warehouse"];
                let storage_name = storage_names[self.rng.gen_range(0..storage_names.len())];
                format!("{} {}", storage_name, room_number)
            }
            RoomType::Laboratory => {
                let lab_names =
                    ["Research Lab", "Testing Lab", "Development Lab", "Innovation Lab"];
                let lab_name = lab_names[self.rng.gen_range(0..lab_names.len())];
                format!("{} {}", lab_name, room_number)
            }
        }
    }

    /// Generate a realistic building name
    fn generate_building_name(&mut self, index: usize) -> String {
        let building_types = [
            "Main Building",
            "North Tower",
            "South Tower",
            "East Wing",
            "West Wing",
            "Executive Building",
            "Research Center",
            "Innovation Hub",
            "Technology Center",
            "Operations Center",
            "Administrative Building",
            "Development Center",
        ];

        if index < building_types.len() {
            building_types[index].to_string()
        } else {
            format!("Building {}", index + 1)
        }
    }
}

impl Default for BuildingGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generator for creating rooms of different types with access requirements
pub struct RoomGenerator {
    rng: Box<dyn RngCore>,
}

impl fmt::Debug for RoomGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoomGenerator").finish()
    }
}

impl RoomGenerator {
    /// Create a new room generator
    pub fn new() -> Self {
        Self { rng: Box::new(thread_rng()) }
    }

    /// Create a new room generator with a specific seed
    pub fn with_seed(seed: u64) -> Self {
        Self { rng: Box::new(StdRng::seed_from_u64(seed)) }
    }

    /// Generate a room with specific type and security level
    pub fn generate_room(
        &mut self,
        building_id: crate::types::BuildingId,
        room_type: RoomType,
        security_level: SecurityLevel,
        name: String,
    ) -> Room {
        let mut room = Room::new(building_id, name, room_type, security_level);

        // Add intermediate access requirements for high-security rooms
        if matches!(security_level, SecurityLevel::HighSecurity | SecurityLevel::MaxSecurity) {
            room.required_intermediate_access = self.generate_intermediate_access_requirements();
        }

        room
    }

    /// Generate a room with automatic type and security level selection
    pub fn generate_random_room(
        &mut self,
        building_id: crate::types::BuildingId,
        room_number: usize,
    ) -> Room {
        let room_type = self.select_random_room_type();
        let security_level = self.select_security_level_for_type(&room_type);
        let name = self.generate_room_name(&room_type, room_number);

        self.generate_room(building_id, room_type, security_level, name)
    }

    /// Generate multiple rooms for a building
    pub fn generate_rooms(
        &mut self,
        building_id: crate::types::BuildingId,
        room_count: usize,
        include_lobby: bool,
    ) -> Vec<Room> {
        let mut rooms = Vec::with_capacity(room_count);

        let mut room_counter = 1;

        // Add lobby if requested
        if include_lobby {
            let lobby = Room::new(
                building_id,
                "Main Lobby".to_string(),
                RoomType::Lobby,
                SecurityLevel::Public,
            );
            rooms.push(lobby);
        }

        // Generate remaining rooms
        let remaining_count = if include_lobby { room_count.saturating_sub(1) } else { room_count };

        for _ in 0..remaining_count {
            let room = self.generate_random_room(building_id, room_counter);
            rooms.push(room);
            room_counter += 1;
        }

        rooms
    }

    /// Select a random room type with realistic distribution
    fn select_random_room_type(&mut self) -> RoomType {
        let rand_val = self.rng.gen::<f64>();

        match rand_val {
            x if x < 0.40 => RoomType::Workspace,
            x if x < 0.55 => RoomType::MeetingRoom,
            x if x < 0.65 => RoomType::Bathroom,
            x if x < 0.75 => RoomType::Kitchen,
            x if x < 0.85 => RoomType::Storage,
            x if x < 0.92 => RoomType::Cafeteria,
            x if x < 0.96 => RoomType::ExecutiveOffice,
            x if x < 0.98 => RoomType::ServerRoom,
            _ => RoomType::Laboratory,
        }
    }

    /// Select appropriate security level for a room type
    fn select_security_level_for_type(&mut self, room_type: &RoomType) -> SecurityLevel {
        match room_type {
            RoomType::Lobby | RoomType::Bathroom | RoomType::Cafeteria | RoomType::Kitchen => {
                SecurityLevel::Public
            }
            RoomType::Workspace | RoomType::MeetingRoom | RoomType::Storage => {
                SecurityLevel::Standard
            }
            RoomType::ExecutiveOffice => SecurityLevel::Restricted,
            RoomType::ServerRoom | RoomType::Laboratory => {
                if self.rng.gen::<f64>() < 0.7 {
                    SecurityLevel::HighSecurity
                } else {
                    SecurityLevel::MaxSecurity
                }
            }
        }
    }

    /// Generate intermediate access requirements for high-security rooms
    fn generate_intermediate_access_requirements(&mut self) -> Vec<crate::types::RoomId> {
        // For now, return empty vector. In a full implementation, this would
        // reference actual security checkpoint rooms that must be accessed first
        Vec::new()
    }

    /// Generate a realistic room name based on type and number
    fn generate_room_name(&mut self, room_type: &RoomType, room_number: usize) -> String {
        match room_type {
            RoomType::Lobby => "Main Lobby".to_string(),
            RoomType::Workspace => {
                let workspace_types = ["Open Office", "Workspace", "Desk Area", "Team Space"];
                let workspace_type = workspace_types[self.rng.gen_range(0..workspace_types.len())];
                format!("{} {}", workspace_type, room_number)
            }
            RoomType::MeetingRoom => {
                let meeting_names = [
                    "Conference Room",
                    "Meeting Room",
                    "Boardroom",
                    "Discussion Room",
                    "Collaboration Space",
                    "Video Conference Room",
                    "Training Room",
                ];
                let meeting_name = meeting_names[self.rng.gen_range(0..meeting_names.len())];
                format!("{} {}", meeting_name, room_number)
            }
            RoomType::Bathroom => format!("Restroom {}", room_number),
            RoomType::Cafeteria => {
                let cafeteria_names = ["Cafeteria", "Dining Hall", "Food Court", "Lunch Room"];
                let cafeteria_name = cafeteria_names[self.rng.gen_range(0..cafeteria_names.len())];
                format!("{} {}", cafeteria_name, room_number)
            }
            RoomType::Kitchen => {
                let kitchen_names = ["Break Room", "Kitchen", "Pantry", "Coffee Station"];
                let kitchen_name = kitchen_names[self.rng.gen_range(0..kitchen_names.len())];
                format!("{} {}", kitchen_name, room_number)
            }
            RoomType::ServerRoom => {
                let server_names = ["Server Room", "Data Center", "Network Room", "IT Closet"];
                let server_name = server_names[self.rng.gen_range(0..server_names.len())];
                format!("{} {}", server_name, room_number)
            }
            RoomType::ExecutiveOffice => {
                let exec_titles = ["Executive Office", "Director Office", "VP Office", "C-Suite"];
                let exec_title = exec_titles[self.rng.gen_range(0..exec_titles.len())];
                format!("{} {}", exec_title, room_number)
            }
            RoomType::Storage => {
                let storage_names = ["Storage Room", "Supply Closet", "Archive Room", "Warehouse"];
                let storage_name = storage_names[self.rng.gen_range(0..storage_names.len())];
                format!("{} {}", storage_name, room_number)
            }
            RoomType::Laboratory => {
                let lab_names =
                    ["Research Lab", "Testing Lab", "Development Lab", "Innovation Lab"];
                let lab_name = lab_names[self.rng.gen_range(0..lab_names.len())];
                format!("{} {}", lab_name, room_number)
            }
        }
    }
}

impl Default for RoomGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Facility generation orchestrator that coordinates all generators
#[derive(Debug)]
pub struct FacilityGenerator {
    location_generator: LocationGenerator,
    building_generator: BuildingGenerator,
    #[allow(dead_code)]
    room_generator: RoomGenerator,
}

impl FacilityGenerator {
    /// Create a new facility generator
    pub fn new() -> Self {
        Self {
            location_generator: LocationGenerator::new(),
            building_generator: BuildingGenerator::new(),
            room_generator: RoomGenerator::new(),
        }
    }

    /// Create a new facility generator with a specific seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            location_generator: LocationGenerator::with_seed(seed),
            building_generator: BuildingGenerator::with_seed(seed + 1),
            room_generator: RoomGenerator::with_seed(seed + 2),
        }
    }

    /// Generate a complete facility structure according to configuration
    pub fn generate_facilities(
        &mut self,
        config: &SimulationConfig,
    ) -> Result<LocationRegistry, String> {
        // Validate configuration
        self.validate_configuration(config)?;

        let mut registry = LocationRegistry::new();

        // Generate locations
        let locations = self.location_generator.generate_locations(config.location_count);

        for mut location in locations {
            // Generate buildings for this location
            let building_count = thread_rng()
                .gen_range(config.min_buildings_per_location..=config.max_buildings_per_location);

            let buildings = self.building_generator.generate_buildings(
                location.id,
                building_count,
                config.min_rooms_per_building,
                config.max_rooms_per_building,
            );

            // Add buildings to location
            for building in buildings {
                location.add_building(building);
            }

            // Validate the generated location
            location.validate().map_err(|e| {
                format!("Generated location {} failed validation: {}", location.name, e)
            })?;

            registry.add_location(location);
        }

        // Final validation
        registry
            .validate()
            .map_err(|e| format!("Generated facility registry failed validation: {}", e))?;

        Ok(registry)
    }

    /// Validate that the configuration meets minimum requirements
    fn validate_configuration(&self, config: &SimulationConfig) -> Result<(), String> {
        if config.location_count == 0 {
            return Err("Location count must be greater than 0".to_string());
        }

        if config.min_buildings_per_location == 0 {
            return Err("Minimum buildings per location must be greater than 0".to_string());
        }

        if config.min_buildings_per_location > config.max_buildings_per_location {
            return Err("Minimum buildings per location cannot be greater than maximum".to_string());
        }

        if config.min_rooms_per_building == 0 {
            return Err("Minimum rooms per building must be greater than 0".to_string());
        }

        if config.min_rooms_per_building > config.max_rooms_per_building {
            return Err("Minimum rooms per building cannot be greater than maximum".to_string());
        }

        Ok(())
    }

    /// Get statistics about the generated facilities
    pub fn get_generation_stats(&self, registry: &LocationRegistry) -> FacilityStats {
        FacilityStats {
            total_locations: registry.location_count(),
            total_buildings: registry.total_building_count(),
            total_rooms: registry.total_room_count(),
            average_buildings_per_location: registry.total_building_count() as f64
                / registry.location_count() as f64,
            average_rooms_per_building: registry.total_room_count() as f64
                / registry.total_building_count() as f64,
        }
    }
}

impl Default for FacilityGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about generated facilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacilityStats {
    /// Total number of locations generated
    pub total_locations: usize,
    /// Total number of buildings across all locations
    pub total_buildings: usize,
    /// Total number of rooms across all buildings
    pub total_rooms: usize,
    /// Average number of buildings per location
    pub average_buildings_per_location: f64,
    /// Average number of rooms per building
    pub average_rooms_per_building: f64,
}

impl fmt::Display for FacilityStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Facility Generation Statistics:\n\
             - Locations: {}\n\
             - Buildings: {} (avg {:.1} per location)\n\
             - Rooms: {} (avg {:.1} per building)\n\
             - Total capacity: {} rooms across {} buildings in {} locations",
            self.total_locations,
            self.total_buildings,
            self.average_buildings_per_location,
            self.total_rooms,
            self.average_rooms_per_building,
            self.total_rooms,
            self.total_buildings,
            self.total_locations
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_generator() {
        let mut generator = LocationGenerator::new();
        let location = generator.generate_location("Test Location".to_string());

        assert_eq!(location.name, "Test Location");
        assert!(location.buildings.is_empty());
        assert_ne!(location.coordinates, (0.0, 0.0)); // Should have realistic coordinates
    }

    #[test]
    fn test_location_generator_with_seed() {
        let mut generator1 = LocationGenerator::with_seed(12345);
        let mut generator2 = LocationGenerator::with_seed(12345);

        let location1 = generator1.generate_location("Test".to_string());
        let location2 = generator2.generate_location("Test".to_string());

        // Same seed should produce same coordinates
        assert_eq!(location1.coordinates, location2.coordinates);
    }

    #[test]
    fn test_multiple_locations_generation() {
        let mut generator = LocationGenerator::new();
        let locations = generator.generate_locations(3);

        assert_eq!(locations.len(), 3);

        // All locations should have different coordinates
        for i in 0..locations.len() {
            for j in (i + 1)..locations.len() {
                assert_ne!(locations[i].coordinates, locations[j].coordinates);
            }
        }
    }

    #[test]
    fn test_building_generator() {
        let mut generator = BuildingGenerator::new();
        let location_id = LocationId::new();
        let building = generator.generate_building(
            location_id,
            "Test Building".to_string(),
            5, // 5 rooms total
        );

        assert_eq!(building.name, "Test Building");
        assert_eq!(building.location_id, location_id);
        assert_eq!(building.room_count(), 5);
        assert!(building.has_lobby()); // Should always have a lobby
    }

    #[test]
    fn test_building_generator_multiple_buildings() {
        let mut generator = BuildingGenerator::new();
        let location_id = LocationId::new();
        let buildings = generator.generate_buildings(
            location_id,
            3,  // 3 buildings
            5,  // min 5 rooms
            10, // max 10 rooms
        );

        assert_eq!(buildings.len(), 3);

        for building in &buildings {
            assert_eq!(building.location_id, location_id);
            assert!(building.has_lobby());
            assert!(building.room_count() >= 5);
            assert!(building.room_count() <= 10);
        }
    }

    #[test]
    fn test_room_generator() {
        let mut generator = RoomGenerator::new();
        let building_id = crate::types::BuildingId::new();

        let room = generator.generate_room(
            building_id,
            RoomType::Workspace,
            SecurityLevel::Standard,
            "Test Room".to_string(),
        );

        assert_eq!(room.building_id, building_id);
        assert_eq!(room.room_type, RoomType::Workspace);
        assert_eq!(room.security_level, SecurityLevel::Standard);
        assert_eq!(room.name, "Test Room");
    }

    #[test]
    fn test_room_generator_random_rooms() {
        let mut generator = RoomGenerator::new();
        let building_id = crate::types::BuildingId::new();
        let rooms = generator.generate_rooms(building_id, 5, true);

        assert_eq!(rooms.len(), 5);

        // First room should be lobby when include_lobby is true
        assert!(rooms[0].is_lobby());

        // All rooms should belong to the building
        for room in &rooms {
            assert_eq!(room.building_id, building_id);
        }
    }

    #[test]
    fn test_facility_generator() {
        let mut generator = FacilityGenerator::new();
        let config = SimulationConfig {
            location_count: 2,
            min_buildings_per_location: 1,
            max_buildings_per_location: 2,
            min_rooms_per_building: 3,
            max_rooms_per_building: 5,
            ..Default::default()
        };

        let registry = generator.generate_facilities(&config).unwrap();

        assert_eq!(registry.location_count(), 2);
        assert!(registry.total_building_count() >= 2); // At least 1 per location
        assert!(registry.total_building_count() <= 4); // At most 2 per location
        assert!(registry.total_room_count() >= 6); // At least 3 per building
    }

    #[test]
    fn test_facility_generator_validation() {
        let generator = FacilityGenerator::new();

        // Invalid config - no locations
        let invalid_config = SimulationConfig { location_count: 0, ..Default::default() };
        assert!(generator.validate_configuration(&invalid_config).is_err());

        // Invalid config - no buildings
        let invalid_config = SimulationConfig {
            location_count: 1,
            min_buildings_per_location: 0,
            ..Default::default()
        };
        assert!(generator.validate_configuration(&invalid_config).is_err());

        // Invalid config - min > max buildings
        let invalid_config = SimulationConfig {
            location_count: 1,
            min_buildings_per_location: 5,
            max_buildings_per_location: 3,
            ..Default::default()
        };
        assert!(generator.validate_configuration(&invalid_config).is_err());

        // Valid config
        let valid_config = SimulationConfig {
            location_count: 1,
            min_buildings_per_location: 1,
            max_buildings_per_location: 2,
            min_rooms_per_building: 3,
            max_rooms_per_building: 5,
            ..Default::default()
        };
        assert!(generator.validate_configuration(&valid_config).is_ok());
    }

    #[test]
    fn test_facility_stats() {
        let mut generator = FacilityGenerator::new();
        let config = SimulationConfig {
            location_count: 2,
            min_buildings_per_location: 2,
            max_buildings_per_location: 2,
            min_rooms_per_building: 4,
            max_rooms_per_building: 4,
            ..Default::default()
        };

        let registry = generator.generate_facilities(&config).unwrap();
        let stats = generator.get_generation_stats(&registry);

        assert_eq!(stats.total_locations, 2);
        assert_eq!(stats.total_buildings, 4); // 2 per location
        assert_eq!(stats.total_rooms, 16); // 4 per building
        assert_eq!(stats.average_buildings_per_location, 2.0);
        assert_eq!(stats.average_rooms_per_building, 4.0);
    }

    #[test]
    fn test_distance_calculation() {
        let generator = LocationGenerator::new();

        // Test distance between Seattle and Portland
        let seattle = (47.6062, -122.3321);
        let portland = (45.5152, -122.6784);

        let distance = generator.calculate_distance(seattle, portland);

        // Distance should be approximately 233 km
        assert!(distance > 200.0 && distance < 300.0);
    }

    #[test]
    fn test_room_type_distribution() {
        let mut generator = BuildingGenerator::new();
        let mut room_type_counts = std::collections::HashMap::new();

        // Generate many rooms to test distribution
        for _ in 0..1000 {
            let room_type = generator.select_room_type();
            *room_type_counts.entry(room_type).or_insert(0) += 1;
        }

        // Workspace should be most common (40% target)
        let workspace_count = room_type_counts.get(&RoomType::Workspace).unwrap_or(&0);
        assert!(*workspace_count > 300); // Should be around 400, allow some variance

        // Meeting rooms should be second most common (15% target)
        let meeting_count = room_type_counts.get(&RoomType::MeetingRoom).unwrap_or(&0);
        assert!(*meeting_count > 100); // Should be around 150, allow some variance
    }
}
