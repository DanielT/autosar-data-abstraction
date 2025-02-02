//! Module for ECU configuration
//!
//! This module provides the structures representing the ECU configuration in the AUTOSAR model.
//! A complete Autosar system has multiple [`EcucModuleConfigurationValues`]
//! containers: one per used base software module

mod definition;
mod values;

pub use definition::*;
pub use values::*;
