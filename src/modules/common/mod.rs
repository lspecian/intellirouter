//! Common utilities and functionality shared across modules

pub mod error_handling;

pub use error_handling::{
    create_default_error_handler, default_retryable_errors, ErrorHandler, ShutdownCoordinator,
    ShutdownSignal, TimeoutConfig,
};
