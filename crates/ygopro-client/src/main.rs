//! Desktop client entry point.
//!
//! Phase 4 replaces this with the Bevy + `bevy_egui` app (Deck Editor, Banlist
//! Time-Machine, 3D Duel Board). Until then it is a tiny smoke test that the
//! data layer loads.

use std::path::PathBuf;

use ygopro_data::{BanlistRegistry, CardDatabase};

fn main() {
    let data = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"));
    match CardDatabase::open(data.join("cards.cdb")) {
        Ok(db) => println!("cards.cdb: {} cards", db.len()),
        Err(e) => eprintln!("cards.cdb: {e}"),
    }
    match BanlistRegistry::load_dir(data.join("lflists")) {
        Ok(reg) => println!("banlists: {} dated lists", reg.len()),
        Err(e) => eprintln!("banlists: {e}"),
    }
    println!("ygopro-client: Bevy frontend arrives in Phase 4.");
}
