//! User modeling and behavior management
//!
//! This module contains all user-related functionality including behavior profiles,
//! state management, user profiles, and user generation.
//!
//! # Overview
//!
//! The user module provides comprehensive modeling of users in a badge access
//! simulation system. It includes:
//!
//! - **User**: Core user entity with access permissions and behavior
//! - **BehaviorProfile**: Defines how users behave and their activity preferences
//! - **UserState**: Manages current state and scheduled activities
//! - **UserProfile**: Provides validation and analysis capabilities
//! - **UserGenerator**: Creates realistic user populations with statistics
//!
//! # Usage Example
//!
//! ```rust
//! use amzn_career_pathway_activity_rust::user::*;
//! use amzn_career_pathway_activity_rust::types::*;
//! use amzn_career_pathway_activity_rust::permissions::*;
//! use amzn_career_pathway_activity_rust::facility::*;
//!
//! // Create a new user
//! let location_id = LocationId::new();
//! let building_id = BuildingId::new();
//! let room_id = RoomId::new();
//! let permissions = PermissionSet::new();
//!
//! let user = User::new(
//!     location_id,
//!     building_id,
//!     room_id,
//!     permissions,
//! );
//!
//! // Generate multiple users with proper setup
//! let mut generator = UserGenerator::new();
//! let config = SimulationConfig::default();
//!
//! // Create a registry with location, building, and workspace rooms
//! let mut registry = LocationRegistry::new();
//! let mut location = Location::new("Test Location".to_string(), (0.0, 0.0));
//!
//! let mut building = Building::new(location.id, "Test Building".to_string());
//! let workspace_room = Room::new(building.id, "Workspace".to_string(), RoomType::Workspace, SecurityLevel::Public);
//! building.add_room(workspace_room);
//!
//! location.add_building(building);
//! registry.add_location(location);
//!
//! let users = generator.generate_users(&config, &registry).unwrap();
//! println!("Generated {} users", users.len());
//! ```

pub mod behavior;
#[allow(clippy::module_inception)]
pub mod user;
pub mod generator;
pub mod profile;
pub mod state;

// Re-export all public types for convenience
pub use behavior::{ActivityPreferences, BehaviorProfile};
pub use user::User;
pub use generator::{UserGenerator, UserStats};
pub use profile::{TravelPatterns, UserProfile};
pub use state::{UserState, ScheduledActivity};
