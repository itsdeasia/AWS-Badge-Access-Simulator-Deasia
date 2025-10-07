//! Facility and location management
//!
//! This module manages locations, buildings, and rooms with their relationships,
//! including facility generation and registry functionality.
//!
//! # Overview
//!
//! The facility module provides a hierarchical model of physical spaces:
//!
//! - **Location**: Top-level geographical locations (e.g., campuses)
//! - **Building**: Buildings within locations with coordinate systems
//! - **Room**: Individual rooms with security levels and access requirements
//! - **LocationRegistry**: Efficient lookup and search functionality
//! - **Generators**: Create realistic facility layouts with proper relationships
//!
//! # Usage Example
//!
//! ```rust
//! use amzn_career_pathway_activity_rust::facility::*;
//! use amzn_career_pathway_activity_rust::types::*;
//!
//! // Create a facility hierarchy
//! let location_id = LocationId::new();
//! let building_id = BuildingId::new();
//!
//! let room = Room::new(
//!     building_id,
//!     "Conference Room A".to_string(),
//!     RoomType::MeetingRoom,
//!     SecurityLevel::Standard,
//! );
//!
//! let mut building = Building::new(
//!     location_id,
//!     "Main Building".to_string(),
//! );
//! building.add_room(room);
//!
//! // Generate facilities automatically
//! let mut generator = FacilityGenerator::new();
//! let config = SimulationConfig::default();
//! let registry = generator.generate_facilities(&config).unwrap();
//! ```

pub mod building;
pub mod generator;
pub mod location;
pub mod registry;
pub mod room;

// Re-export all public types for convenience
pub use building::Building;
pub use generator::{
    BuildingGenerator, FacilityGenerator, FacilityStats, LocationGenerator, RoomGenerator,
};
pub use location::Location;
pub use registry::{AccessComplexityStats, LocationRegistry};
pub use room::Room;
