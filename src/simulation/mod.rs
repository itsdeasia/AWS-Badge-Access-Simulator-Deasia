//! Simulation orchestration and control
//!
//! This module contains the main simulation orchestrator, time management,
//! behavior engine, statistics collection, and error handling.
//!
//! # Overview
//!
//! The simulation module orchestrates the entire badge access simulation:
//!
//! - **SimulationOrchestrator**: Main controller that coordinates all simulation components
//! - **TimeManager**: Handles time acceleration and temporal calculations
//! - **BehaviorEngine**: Generates realistic user activity patterns
//! - **SimulationStatistics**: Collects and reports simulation metrics
//! - **SimulationError**: Comprehensive error handling for simulation operations
//!
//! # Usage Example
//!
//! ```rust
//! use amzn_career_pathway_activity_rust::simulation::*;
//! use amzn_career_pathway_activity_rust::types::*;
//!
//! // Create simulation configuration
//! let config = SimulationConfig {
//!     user_count: 50,
//!     location_count: 3,
//!     ..Default::default()
//! };
//!
//! // Initialize simulation
//! let orchestrator = SimulationOrchestrator::new(config).unwrap();
//!
//! // Time management
//! let time_manager = TimeManager::new();
//! let is_business_hours = time_manager.is_business_hours(chrono::Utc::now());
//! ```

pub mod batch_generator;
pub mod behavior_engine;
pub mod error;
pub mod logging;
pub mod orchestrator;
pub mod statistics;
pub mod time_manager;
pub mod time_variance;

// Re-export all public types for convenience
pub use batch_generator::*;
pub use behavior_engine::*;
pub use error::*;
pub use logging::*;
pub use orchestrator::*;
pub use statistics::*;
pub use time_manager::*;
pub use time_variance::*;
