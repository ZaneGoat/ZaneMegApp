//! The health snapshot: what this machine is doing, read from `sysfs` and `/proc` and handed to
//! the shell as [`Measured`](crate::core::measured::Measured) values (design §5.2; AC-11 tiles,
//! AC-14 health rows).
//!
//! Two rules shape this module.
//!
//! * **Nothing here starts a process.** A subprocess on the UI thread is what made the old app
//!   spawn `sudo -n ufw status` every second (D19); the fan, temperatures, memory, battery and
//!   firewall are all plain file reads, and the root filesystem is one `statvfs(2)` call. The
//!   shell's `ip`/`systemctl` reads belong to the sampler thread in 4b, not to a frame.
//! * **A health word is a function of a reading, never a value.** [`SubsystemHealth`] has private
//!   fields and the only constructors derive the word from a [`Measured`]
//!   (crate::core::measured::Measured), a [`Belief`](crate::core::measured::Belief) or an
//!   [`Effect`](crate::core::measured::Effect). That is what stops `ok` sitting over an
//!   `Unavailable` (T-29, SEC-V4-25), and what makes AC-14's "`off` only when something really
//!   is off" true.
//!
//! The four files are AC-43's 500-line rule, not a design change: `readings.rs` holds the value
//! types, `subsystems.rs` the derived rows, `samplers.rs` the readers under them, and `tests.rs`
//! the fixture machine they are all proved against.

mod readings;
mod samplers;
mod subsystems;

pub use readings::{BatteryInfo, BatteryState, DiskInfo, FirewallState, MemInfo, Roots};
pub use samplers::Sampler;
pub use subsystems::{HealthSnapshot, HealthWord, Subsystem, SubsystemHealth};

#[cfg(test)]
mod tests;
