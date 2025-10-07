//! Badge Access Simulator
//!
//! A realistic user badge access event simulation system that generates batch data
//! to mimic real-world security badge usage patterns across multiple geographical locations.
//!
//! # Overview
//!
//! This library provides a comprehensive simulation framework for modeling user badge
//! access patterns in corporate environments. It generates realistic access events that
//! can be used for security analysis, anomaly detection testing, and system validation.
//!
//! ## Key Features
//!
//! - **Realistic User Modeling**: Users with behavior profiles, schedules, and permissions
//! - **Hierarchical Facility Structure**: Locations, buildings, and rooms with security levels
//! - **Complex Access Patterns**: Multi-step access flows and permission validation
//! - **Event Generation**: Realistic timing and access patterns with anomaly injection
//! - **Batch Processing**: Efficient event generation for complete simulation runs
//! - **Configurable Simulation**: Extensive configuration options for different scenarios
//!
//! ## Quick Start
//!
//! ```rust
//! use amzn_career_pathway_activity_rust::*;
//!
//! // Create a basic simulation configuration
//! let config = SimulationConfig {
//!     user_count: 100,
//!     location_count: 3,
//!     ..Default::default()
//! };
//!
//! // Initialize the simulation
//! let orchestrator = SimulationOrchestrator::new(config)?;
//!
//! // Get simulation statistics
//! let stats = orchestrator.get_statistics();
//! println!("Simulation configured with {} users", stats.total_users);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Module Organization
//!
//! - [`types`]: Core types, identifiers, and configuration
//! - [`permissions`]: Access control and permission management
//! - [`user`]: User modeling and behavior profiles
//! - [`facility`]: Location, building, and room management
//! - [`events`]: Access event generation and metadata
//! - [`simulation`]: Simulation orchestration and control

//!
//! ## Architecture
//!
//! The library follows a modular architecture with clear separation of concerns:
//!
//! ```text
//! ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//! │   Types     │    │ Permissions │    │    User     │
//! │             │    │             │    │             │
//! │ Identifiers │◄───┤ Access      │◄───┤ Behavior    │
//! │ Enums       │    │ Control     │    │ Profiles    │
//! │ Config      │    │             │    │             │
//! └─────────────┘    └─────────────┘    └─────────────┘
//!        ▲                   ▲                   ▲
//!        │                   │                   │
//! ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//! │  Facility   │    │   Events    │    │ Simulation  │
//! │             │    │             │    │             │
//! │ Locations   │◄───┤ Generation  │◄───┤ Orchestrator│
//! │ Buildings   │    │ Metadata    │    │ Statistics  │
//! │ Rooms       │    │             │    │             │
//! └─────────────┘    └─────────────┘    └─────────────┘
//!                            ▲                   ▲
//!                            │                   │

//! ```
#![warn(missing_docs, missing_debug_implementations, unreachable_pub)]

// Module declarations
pub mod user;
pub mod events;
pub mod facility;
pub mod permissions;
pub mod simulation;

pub mod types;

// Re-export all public types for backward compatibility

// Core types and identifiers
pub use types::{
    ActivityType,
    BuildingId,
    ConfigValidationError,
    // Identifiers
    UserId,
    EventType,
    LocationId,
    OutputFormat,
    RoomId,
    // Enums
    RoomType,
    SecurityLevel,
    // Configuration
    SimulationConfig,
};

// Permissions and access control
pub use permissions::{access_flow::ValidationResult, AccessFlow, PermissionLevel, PermissionSet};

// User types and functionality
pub use user::{
    ActivityPreferences, BehaviorProfile, User, UserGenerator, UserState,
    UserStats, ScheduledActivity, TravelPatterns, UserProfile,
};

// Facility types and functionality
pub use facility::{
    AccessComplexityStats as FacilityAccessComplexityStats, Building, BuildingGenerator,
    FacilityGenerator, FacilityStats, Location, LocationGenerator, LocationRegistry, Room,
    RoomGenerator,
};

// Event types and functionality
pub use events::{AccessAttempt, AccessEvent, EventGenerator, ImpossibleTravelerMetadata};

// Simulation types and functionality
pub use simulation::{
    AccessComplexityStats as SimulationAccessComplexityStats, BehaviorEngine, RuntimeStatistics,
    SimulationError, SimulationOrchestrator, SimulationStatistics, TimeManager,
};
