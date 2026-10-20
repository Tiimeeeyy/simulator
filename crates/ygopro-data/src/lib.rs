//! # ygopro-data
//!
//! Everything the simulator knows about cards *before* a duel starts:
//!
//! * [`CardDatabase`] – `cards.cdb` (SQLite) loaded fully into memory, with
//!   O(1) code / name lookup, fuzzy name search, alternate-art alias
//!   normalisation and precomputed bitset indices for fast filtering.
//! * [`StringsConf`] – `strings.conf` (system strings, counters, setnames).
//! * [`BanlistRegistry`] – a **time-travel** Forbidden/Limited registry. Ask it
//!   for "TCG as of 2023-03-01" and it resolves the list in force on that day.
//! * [`ReleaseDateIndex`] – per-passcode TCG/OCG release dates, used to reject
//!   cards that did not exist yet in the selected format.
//! * [`Deck`] – `.ydk` / `ydke://` import & export.
//! * [`import`] – the smart plain-text deck importer (auto-partitions Extra
//!   Deck monsters and splits Main/Side overflow deterministically).
//! * [`validate`] – deck legality checks against an [`ActiveBanlist`].
//! * [`builtin`] – decks shipped with the simulator (the Ishizu Tearlaments
//!   bot deck).

pub mod banlist;
pub mod builtin;
pub mod card;
pub mod constants;
pub mod database;
pub mod deck;
pub mod error;
pub mod import;
pub mod release;
pub mod strings_conf;
pub mod validate;

pub use banlist::{
    ActiveBanlist, Banlist, BanlistQuery, BanlistRegistry, BanlistSelection, FormatRegion,
};
pub use card::Card;
pub use database::{CardDatabase, CardFilter, normalize_name};
pub use deck::{Deck, DeckSection};
pub use error::{Error, Result};
pub use import::{ImportOptions, ImportedDeck, SplitStrategy, import_plaintext, parse_plaintext_deck};
pub use release::{ReleaseDateIndex, ReleaseDates};
pub use strings_conf::StringsConf;
pub use validate::{DeckReport, DeckRules, DeckValidator, Violation};
