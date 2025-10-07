//! Access control and permissions management
//!
//! This module handles permission sets, access validation, and access flow logic
//! for controlling user access to rooms, buildings, and locations.
//!
//! # Overview
//!
//! The permissions module provides comprehensive access control:
//!
//! - **PermissionSet**: Manages user permissions and access levels
//! - **AccessFlow**: Handles complex multi-step access sequences
//! - **PermissionLevel**: Defines hierarchical access levels
//!
//! # Usage Example
//!
//! ```rust
//! use amzn_career_pathway_activity_rust::permissions::*;
//! use amzn_career_pathway_activity_rust::types::*;
//! use chrono::Duration;
//!
//! // Create permission set
//! let location_id = LocationId::new();
//! let room_id = RoomId::new();
//! let permissions = PermissionSet::with_permissions(vec![
//!     PermissionLevel::Location(location_id),
//!     PermissionLevel::Room(room_id),
//! ]);
//!
//! // Check access
//! let building_id = BuildingId::new();
//! let has_access = permissions.can_access_room(room_id, building_id, location_id);
//!
//! // Create access flow for complex sequences
//! let target_room = RoomId::new();
//! let intermediate_room = RoomId::new();
//! let sequence = vec![intermediate_room, target_room];
//! let access_flow = AccessFlow::new(
//!     sequence,
//!     Duration::seconds(30),
//!     true,  // requires_lobby_access
//!     false, // involves_high_security
//! );
//! ```

pub mod access_flow;
pub mod permission_set;

// Re-export all public types for convenience
pub use access_flow::*;
pub use permission_set::*;
