//! Safe, idiomatic wrapper around [`ocgcore_sys`].
//!
//! Provides the [`Duel`] abstraction, [`ScriptLoader`], [`CoreMessage`] parsing,
//! and [`PlayerResponse`] builders.

pub mod duel;
pub mod error;
pub mod message;
pub mod response;
pub mod script;

pub use duel::{Duel, DuelConfig, ProcessResult};
pub use error::{Error, Result};
pub use message::{
    parse_message_stream, CmdActivatableItem, CmdAttackableItem, CmdCardItem, CoreMessage,
    SelectCardItem, SelectChainItem,
};
pub use response::{BattleCmdChoice, IdleCmdChoice, PlayerResponse};
pub use script::ScriptLoader;
