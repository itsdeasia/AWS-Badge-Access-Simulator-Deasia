//! Event generation and access event management
//!
//! This module handles access events, event generation logic, and metadata
//! for various event scenarios including impossible traveler detection.
//!
//! # Overview
//!
//! The events module provides realistic event generation and management:
//!
//! - **AccessEvent**: Represents badge access attempts with timestamps and outcomes
//! - **EventGenerator**: Creates realistic access events with proper timing
//! - **ImpossibleTravelerMetadata**: Specialized metadata for anomaly detection
//! - **AccessAttempt**: Models individual access attempts with context
//!
//! # Usage Example
//!
//! ```rust
//! use amzn_career_pathway_activity_rust::events::*;
//! use amzn_career_pathway_activity_rust::types::*;
//! use amzn_career_pathway_activity_rust::facility::*;
//! use amzn_career_pathway_activity_rust::simulation::*;
//! use chrono::Utc;
//!
//! // Create access event
//! let event = AccessEvent::new(
//!     Utc::now(),
//!     UserId::new(),
//!     RoomId::new(),
//!     BuildingId::new(),
//!     LocationId::new(),
//!     true,
//!     EventType::Success,
//! );
//!
//! // Generate events
//! let config = SimulationConfig::default();
//! let registry = LocationRegistry::new();
//! let time_manager = TimeManager::new();
//! let mut generator = EventGenerator::new(config, registry, time_manager);
//! ```

pub mod access_event;
pub mod generator;
pub mod metadata;

// Re-export all public types for convenience
pub use access_event::*;
pub use generator::*;
pub use metadata::*;
