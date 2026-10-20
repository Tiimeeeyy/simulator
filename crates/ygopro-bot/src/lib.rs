//! Native duel AI for ygopro.
//!
//! Provides the [`DuelAgent`] trait, [`BotBoardState`] tracker,
//! [`TearlamentsPolicy`] specialized AI, and [`GenericAgent`].

pub mod agent;
pub mod generic;
pub mod state;
pub mod tearlaments;

pub use agent::DuelAgent;
pub use generic::GenericAgent;
pub use state::BotBoardState;
pub use tearlaments::TearlamentsPolicy;
