//! Core logic for add_new_method_to_wrapper.
//!
//! Testable without a window. The egui app calls into this crate.

pub mod edit;
pub mod events;
pub mod guard;
pub mod linefold;
pub mod pipeline;
pub mod precheck;
pub mod report;
pub mod runner;
pub mod scan;
pub mod settings;
pub mod ui_state;