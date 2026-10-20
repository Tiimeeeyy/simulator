//! Core trait for AI duel agents.

use ygopro_core::{CoreMessage, PlayerResponse};
use ygopro_data::CardDatabase;

/// An autonomous AI agent capable of playing a duel.
pub trait DuelAgent: Send {
    /// Human-readable identifier for the bot.
    fn name(&self) -> &str;

    /// The player slot this agent controls (0 or 1).
    fn player_id(&self) -> u8;

    /// Reset internal state for a new duel.
    fn reset(&mut self);

    /// Observe an engine message to update internal game state and HOPT tracking.
    fn on_message(&mut self, msg: &CoreMessage);

    /// Compute a decision when the engine prompts this agent.
    fn respond(&mut self, prompt: &CoreMessage, db: &CardDatabase) -> PlayerResponse;
}
