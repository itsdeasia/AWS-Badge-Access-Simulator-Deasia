//! Core types and identifiers for the badge access simulator
//!
//! This module contains fundamental types, identifiers, and configuration structures
//! used throughout the simulation system.
//!
//! # Overview
//!
//! The types module provides the foundational data types for the simulation:
//!
//! - **Identifiers**: UUID-based unique identifiers for all entities
//! - **Enums**: Type-safe enumerations for room types, security levels, etc.
//! - **Configuration**: Simulation configuration with validation and CLI support
//!
//! # Usage Example
//!
//! ```rust
//! use amzn_career_pathway_activity_rust::types::*;
//!
//! // Create unique identifiers
//! let user_id = UserId::new();
//! let location_id = LocationId::new();
//! let room_id = RoomId::new();
//!
//! // Use enums for type safety
//! let room_type = RoomType::MeetingRoom;
//! let security_level = SecurityLevel::HighSecurity;
//! let activity_type = ActivityType::Meeting;
//!
//! // Configure simulation
//! let config = SimulationConfig {
//!     user_count: 100,
//!     location_count: 5,
//!     ..Default::default()
//! };
//! ```

pub mod config;
pub mod enums;
pub mod identifiers;

// Re-export all public types for convenience
pub use config::*;
pub use enums::*;
pub use identifiers::*;
