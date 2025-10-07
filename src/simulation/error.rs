//! Error types and handling
//!
//! This module contains error types and error handling for the simulation.

use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Errors that can occur during simulation
#[derive(Debug, Error)]
pub enum SimulationError {
    /// Configuration validation failed
    #[error("Configuration validation failed: {0}")]
    ConfigurationError(String),

    /// User generation failed
    #[error("User generation failed: {0}")]
    UserGenerationError(String),

    /// Location setup failed
    #[error("Location setup failed: {0}")]
    LocationSetupError(String),

    /// Event generation failed
    #[error("Event generation failed: {0}")]
    EventGenerationError(String),

    /// Time management error
    #[error("Time management error: {0}")]
    TimeError(String),

    /// I/O error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Permission error
    #[error("Permission error: {0}")]
    PermissionError(String),

    /// Facility error
    #[error("Facility error: {0}")]
    FacilityError(String),

    /// Behavior engine error
    #[error("Behavior engine error: {0}")]
    BehaviorEngineError(String),

    /// Statistics error
    #[error("Statistics error: {0}")]
    StatisticsError(String),
}

impl From<String> for SimulationError {
    fn from(s: String) -> Self {
        SimulationError::EventGenerationError(s)
    }
}

impl From<&str> for SimulationError {
    fn from(s: &str) -> Self {
        SimulationError::EventGenerationError(s.to_string())
    }
}

impl From<anyhow::Error> for SimulationError {
    fn from(error: anyhow::Error) -> Self {
        SimulationError::EventGenerationError(error.to_string())
    }
}

impl SimulationError {
    /// Create a configuration error
    pub fn configuration_error(msg: impl Into<String>) -> Self {
        Self::ConfigurationError(msg.into())
    }

    /// Create a user generation error
    pub fn user_generation_error(msg: impl Into<String>) -> Self {
        Self::UserGenerationError(msg.into())
    }

    /// Create a location setup error
    pub fn location_setup_error(msg: impl Into<String>) -> Self {
        Self::LocationSetupError(msg.into())
    }

    /// Create an event generation error
    pub fn event_generation_error(msg: impl Into<String>) -> Self {
        Self::EventGenerationError(msg.into())
    }

    /// Create a time management error
    pub fn time_error(msg: impl Into<String>) -> Self {
        Self::TimeError(msg.into())
    }

    /// Create a permission error
    pub fn permission_error(msg: impl Into<String>) -> Self {
        Self::PermissionError(msg.into())
    }

    /// Create a facility error
    pub fn facility_error(msg: impl Into<String>) -> Self {
        Self::FacilityError(msg.into())
    }

    /// Create a behavior engine error
    pub fn behavior_engine_error(msg: impl Into<String>) -> Self {
        Self::BehaviorEngineError(msg.into())
    }

    /// Create a statistics error
    pub fn statistics_error(msg: impl Into<String>) -> Self {
        Self::StatisticsError(msg.into())
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        match self {
            SimulationError::ConfigurationError(_) => false,
            SimulationError::UserGenerationError(_) => true,
            SimulationError::LocationSetupError(_) => false,
            SimulationError::EventGenerationError(_) => true,
            SimulationError::TimeError(_) => true,
            SimulationError::IoError(_) => true,
            SimulationError::SerializationError(_) => true,
            SimulationError::PermissionError(_) => true,
            SimulationError::FacilityError(_) => true,
            SimulationError::BehaviorEngineError(_) => true,
            SimulationError::StatisticsError(_) => true,
        }
    }

    /// Get the error category
    pub fn category(&self) -> &'static str {
        match self {
            SimulationError::ConfigurationError(_) => "Configuration",
            SimulationError::UserGenerationError(_) => "User Generation",
            SimulationError::LocationSetupError(_) => "Location Setup",
            SimulationError::EventGenerationError(_) => "Event Generation",
            SimulationError::TimeError(_) => "Time Management",
            SimulationError::IoError(_) => "IO",
            SimulationError::SerializationError(_) => "Serialization",
            SimulationError::PermissionError(_) => "Permission",
            SimulationError::FacilityError(_) => "Facility",
            SimulationError::BehaviorEngineError(_) => "Behavior Engine",
            SimulationError::StatisticsError(_) => "Statistics",
        }
    }
}

/// Result type for simulation operations
pub type SimulationResult<T> = Result<T, SimulationError>;

/// Error recovery strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Retry the operation with the same parameters
    Retry,
    /// Retry with fallback parameters
    RetryWithFallback,
    /// Skip the current operation and continue
    Skip,
    /// Use default values and continue
    UseDefaults,
    /// Abort the entire simulation
    Abort,
}

/// Error recovery context
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    /// The recovery strategy to use
    pub strategy: RecoveryStrategy,
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Current retry count
    pub retry_count: usize,
    /// Whether to log the error
    pub log_error: bool,
    /// Additional context information
    pub context: String,
}

impl Default for RecoveryContext {
    fn default() -> Self {
        Self {
            strategy: RecoveryStrategy::Skip,
            max_retries: 3,
            retry_count: 0,
            log_error: true,
            context: String::new(),
        }
    }
}

impl RecoveryContext {
    /// Create a new recovery context with retry strategy
    pub fn retry(max_retries: usize) -> Self {
        Self { strategy: RecoveryStrategy::Retry, max_retries, ..Default::default() }
    }

    /// Create a new recovery context with retry and fallback strategy
    pub fn retry_with_fallback(max_retries: usize) -> Self {
        Self { strategy: RecoveryStrategy::RetryWithFallback, max_retries, ..Default::default() }
    }

    /// Create a new recovery context with skip strategy
    pub fn skip() -> Self {
        Self { strategy: RecoveryStrategy::Skip, ..Default::default() }
    }

    /// Create a new recovery context with use defaults strategy
    pub fn use_defaults() -> Self {
        Self { strategy: RecoveryStrategy::UseDefaults, ..Default::default() }
    }

    /// Create a new recovery context with abort strategy
    pub fn abort() -> Self {
        Self { strategy: RecoveryStrategy::Abort, max_retries: 0, ..Default::default() }
    }

    /// Add context information
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = context.into();
        self
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Check if more retries are available
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
}

/// Error handler for graceful error recovery
#[derive(Debug)]
pub struct ErrorHandler {
    /// Whether to continue on recoverable errors
    pub continue_on_recoverable: bool,
    /// Default recovery context
    pub default_recovery: RecoveryContext,
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self { continue_on_recoverable: true, default_recovery: RecoveryContext::default() }
    }
}

impl ErrorHandler {
    /// Create a new error handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle an error with the given recovery context
    pub fn handle_error(
        &self,
        error: &SimulationError,
        context: &RecoveryContext,
    ) -> RecoveryStrategy {
        // Log the error if requested
        if context.log_error {
            match error.category() {
                "Configuration" | "Location Setup" => {
                    error!("Critical error in {}: {}", error.category(), error);
                }
                "User Generation" | "Event Generation" | "Behavior Engine" => {
                    warn!("Recoverable error in {}: {}", error.category(), error);
                }
                _ => {
                    info!("Error in {}: {}", error.category(), error);
                }
            }

            if !context.context.is_empty() {
                debug!("Error context: {}", context.context);
            }
        }

        // Determine recovery strategy based on error type and recoverability
        if !error.is_recoverable() {
            warn!("Non-recoverable error encountered, aborting operation");
            return RecoveryStrategy::Abort;
        }

        // Use the specified strategy for recoverable errors
        match context.strategy {
            RecoveryStrategy::Retry if context.can_retry() => {
                info!(
                    "Retrying operation (attempt {} of {})",
                    context.retry_count + 1,
                    context.max_retries
                );
                RecoveryStrategy::Retry
            }
            RecoveryStrategy::RetryWithFallback if context.can_retry() => {
                info!(
                    "Retrying operation with fallback (attempt {} of {})",
                    context.retry_count + 1,
                    context.max_retries
                );
                RecoveryStrategy::RetryWithFallback
            }
            RecoveryStrategy::Retry | RecoveryStrategy::RetryWithFallback => {
                warn!("Max retries exceeded, skipping operation");
                RecoveryStrategy::Skip
            }
            strategy => strategy,
        }
    }

    /// Handle an error with default recovery context
    pub fn handle_error_default(&self, error: &SimulationError) -> RecoveryStrategy {
        self.handle_error(error, &self.default_recovery)
    }

    /// Execute an operation with error recovery
    pub fn execute_with_recovery<T, F>(
        &self,
        mut operation: F,
        mut context: RecoveryContext,
    ) -> SimulationResult<Option<T>>
    where
        F: FnMut() -> SimulationResult<T>,
    {
        loop {
            match operation() {
                Ok(result) => return Ok(Some(result)),
                Err(error) => {
                    let strategy = self.handle_error(&error, &context);

                    match strategy {
                        RecoveryStrategy::Retry => {
                            context.increment_retry();
                            continue;
                        }
                        RecoveryStrategy::RetryWithFallback => {
                            context.increment_retry();
                            // The operation function should handle fallback logic internally
                            continue;
                        }
                        RecoveryStrategy::Skip => {
                            warn!("Skipping operation due to error: {}", error);
                            return Ok(None);
                        }
                        RecoveryStrategy::UseDefaults => {
                            info!("Using default values due to error: {}", error);
                            return Ok(None);
                        }
                        RecoveryStrategy::Abort => {
                            error!("Aborting due to non-recoverable error: {}", error);
                            return Err(error);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_creation() {
        let config_error = SimulationError::configuration_error("Invalid config");
        assert!(matches!(config_error, SimulationError::ConfigurationError(_)));
        assert_eq!(config_error.to_string(), "Configuration validation failed: Invalid config");

        let user_error =
            SimulationError::user_generation_error("Failed to create user");
        assert!(matches!(user_error, SimulationError::UserGenerationError(_)));
        assert_eq!(
            user_error.to_string(),
            "User generation failed: Failed to create user"
        );
    }

    #[test]
    fn test_error_from_string() {
        let error: SimulationError = "Test error".to_string().into();
        assert!(matches!(error, SimulationError::EventGenerationError(_)));
        assert_eq!(error.to_string(), "Event generation failed: Test error");
    }

    #[test]
    fn test_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let sim_error: SimulationError = io_error.into();
        assert!(matches!(sim_error, SimulationError::IoError(_)));
    }

    #[test]
    fn test_error_recoverability() {
        let config_error = SimulationError::configuration_error("Invalid config");
        assert!(!config_error.is_recoverable());

        let event_error = SimulationError::event_generation_error("Event failed");
        assert!(event_error.is_recoverable());

        let location_error = SimulationError::location_setup_error("Location failed");
        assert!(!location_error.is_recoverable());
    }

    #[test]
    fn test_error_categories() {
        let config_error = SimulationError::configuration_error("Invalid config");
        assert_eq!(config_error.category(), "Configuration");

        let user_error = SimulationError::user_generation_error("User failed");
        assert_eq!(user_error.category(), "User Generation");

        let time_error = SimulationError::time_error("Time failed");
        assert_eq!(time_error.category(), "Time Management");

        let permission_error = SimulationError::permission_error("Permission denied");
        assert_eq!(permission_error.category(), "Permission");

        let facility_error = SimulationError::facility_error("Facility error");
        assert_eq!(facility_error.category(), "Facility");

        let behavior_error = SimulationError::behavior_engine_error("Behavior error");
        assert_eq!(behavior_error.category(), "Behavior Engine");

        let stats_error = SimulationError::statistics_error("Stats error");
        assert_eq!(stats_error.category(), "Statistics");
    }

    #[test]
    fn test_simulation_result_type() {
        let success: SimulationResult<i32> = Ok(42);
        assert!(success.is_ok());
        if let Ok(value) = success {
            assert_eq!(value, 42);
        }

        let failure: SimulationResult<i32> = Err(SimulationError::configuration_error("Test"));
        assert!(failure.is_err());
    }

    #[test]
    fn test_recovery_context_creation() {
        let retry_context = RecoveryContext::retry(5);
        assert_eq!(retry_context.strategy, RecoveryStrategy::Retry);
        assert_eq!(retry_context.max_retries, 5);
        assert_eq!(retry_context.retry_count, 0);

        let fallback_context = RecoveryContext::retry_with_fallback(3);
        assert_eq!(fallback_context.strategy, RecoveryStrategy::RetryWithFallback);
        assert_eq!(fallback_context.max_retries, 3);

        let skip_context = RecoveryContext::skip();
        assert_eq!(skip_context.strategy, RecoveryStrategy::Skip);

        let defaults_context = RecoveryContext::use_defaults();
        assert_eq!(defaults_context.strategy, RecoveryStrategy::UseDefaults);

        let abort_context = RecoveryContext::abort();
        assert_eq!(abort_context.strategy, RecoveryStrategy::Abort);
        assert_eq!(abort_context.max_retries, 0);
    }

    #[test]
    fn test_recovery_context_retry_logic() {
        let mut context = RecoveryContext::retry(3);
        assert!(context.can_retry());
        assert_eq!(context.retry_count, 0);

        context.increment_retry();
        assert!(context.can_retry());
        assert_eq!(context.retry_count, 1);

        context.increment_retry();
        context.increment_retry();
        assert!(!context.can_retry());
        assert_eq!(context.retry_count, 3);
    }

    #[test]
    fn test_recovery_context_with_context() {
        let context = RecoveryContext::retry(3).with_context("Test operation");
        assert_eq!(context.context, "Test operation");
    }

    #[test]
    fn test_error_handler_strategy_selection() {
        let handler = ErrorHandler::new();

        // Non-recoverable error should always abort
        let config_error = SimulationError::configuration_error("Invalid config");
        let context = RecoveryContext::retry(3);
        let strategy = handler.handle_error(&config_error, &context);
        assert_eq!(strategy, RecoveryStrategy::Abort);

        // Recoverable error with retries available should retry
        let event_error = SimulationError::event_generation_error("Event failed");
        let context = RecoveryContext::retry(3);
        let strategy = handler.handle_error(&event_error, &context);
        assert_eq!(strategy, RecoveryStrategy::Retry);

        // Recoverable error with no retries left should skip
        let mut context = RecoveryContext::retry(2);
        context.retry_count = 2; // Max retries reached
        let strategy = handler.handle_error(&event_error, &context);
        assert_eq!(strategy, RecoveryStrategy::Skip);
    }

    #[test]
    fn test_error_handler_execute_with_recovery_success() {
        let handler = ErrorHandler::new();
        let context = RecoveryContext::retry(3);

        let result = handler.execute_with_recovery(|| Ok(42), context);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(42));
    }

    #[test]
    fn test_error_handler_execute_with_recovery_retry_then_success() {
        let handler = ErrorHandler::new();
        let context = RecoveryContext::retry(3);

        let mut attempt_count = 0;
        let result = handler.execute_with_recovery(
            || {
                attempt_count += 1;
                if attempt_count < 3 {
                    Err(SimulationError::event_generation_error("Temporary failure"))
                } else {
                    Ok(42)
                }
            },
            context,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(42));
        assert_eq!(attempt_count, 3);
    }

    #[test]
    fn test_error_handler_execute_with_recovery_max_retries_exceeded() {
        let handler = ErrorHandler::new();
        let context = RecoveryContext::retry(2);

        let mut attempt_count = 0;
        let result: SimulationResult<Option<i32>> = handler.execute_with_recovery(
            || {
                attempt_count += 1;
                Err(SimulationError::event_generation_error("Persistent failure"))
            },
            context,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None); // Should skip after max retries
        assert_eq!(attempt_count, 3); // Initial attempt + 2 retries
    }

    #[test]
    fn test_error_handler_execute_with_recovery_non_recoverable() {
        let handler = ErrorHandler::new();
        let context = RecoveryContext::retry(3);

        let result: SimulationResult<Option<i32>> = handler.execute_with_recovery(
            || Err(SimulationError::configuration_error("Fatal error")),
            context,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SimulationError::ConfigurationError(_)));
    }
}
