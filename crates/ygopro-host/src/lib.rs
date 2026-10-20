//! Local duel coordinator.
//!
//! Provides [`DuelSession`], [`PlayerController`], and [`WindBotBridge`] for
//! driving local and networked duels.

pub mod error;
pub mod session;
pub mod windbot;

pub use error::{HostError, Result};
pub use session::{DuelSession, PlayerController, StepOutcome};
pub use windbot::WindBotBridge;
