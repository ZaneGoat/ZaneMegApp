//! The Hardware tab's sections, one file per card group.
//!
//! Split out of `tab_hardware.rs` when that file passed AC-43's 500-line ceiling: the skeleton and
//! the state live in the parent, each section here paints one group of cards.

pub mod dials;
pub mod fan_battery;
pub mod plots;
pub mod profiles;
