//! Local duel coordinator.
//!
//! **Phase 3**: drives a `ygopro_core` duel, connecting the human UI over
//! in-memory channels and Player 2 to either the in-process `ygopro-bot` or an
//! external WindBot process via a `127.0.0.1:0` `STOC_*`/`CTOS_*` listener.
