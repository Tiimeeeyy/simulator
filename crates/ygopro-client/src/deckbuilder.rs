//! Master Duel inspired Deck Builder with instant search, deep filtering,
//! sort deck, test opening hand (draw 5), YDK save/load, and format legality.

use std::path::{Path, PathBuf};

use bevy_egui::egui::{
    self, Color32, Layout, RichText, ScrollArea, Ui, Vec2,
};
use chrono::NaiveDate;
use ygopro_data::{
    constants::*, parse_plaintext_deck, BanlistQuery, BanlistRegistry, Card, CardDatabase, CardFilter,
    Deck, DeckValidator, ReleaseDateIndex,
};

use crate::card_widget::draw_card_item;
use crate::theme::*;

/// State for the Deck Builder screen.
pub struct DeckBuilderState {
    pub deck: Deck,
    pub deck_name: String,
    pub search_query: String,
    pub filter_type: u32,
    pub filter_attribute: u32,
    pub filter_level: u32,
    pub selected_card: Option<Card>,
    pub selected_banlist: String,
    pub target_date: NaiveDate,
    pub catalog_page: usize,
    pub page_size: usize,
    // Modals
    pub import_dialog_open: bool,
    pub import_text: String,
    pub export_dialog_open: bool,
    pub export_format: usize, // 0 = YDK, 1 = YDKE, 2 = Plaintext
    pub violations_dialog_open: bool,
    pub test_hand_open: bool,
    pub test_hand_cards: Vec<u32>,
    pub status_message: Option<(String, bool)>, // (msg, is_error)
}

impl DeckBuilderState {
    pub fn new(banlist_name: &str, target_date: NaiveDate) -> Self {
        let deck = ygopro_data::builtin::tearlaments_ishizu_deck();
        let deck_name = deck.name.clone();
        Self {
            deck,
            deck_name,
            search_query: String::new(),
            filter_type: 0,
            filter_attribute: 0,
            filter_level: 0,
            selected_card: None,
            selected_banlist: banlist_name.to_string(),
            target_date,
            catalog_page: 0,
            page_size: 42,
            import_dialog_open: false,
            import_text: String::new(),
            export_dialog_open: false,
            export_format: 0,
            violations_dialog_open: false,
            test_hand_open: false,
            test_hand_cards: Vec::new(),
            status_message: None,
        }
    }

    /// Add a card into Main or Extra deck.
    pub fn add_card(&mut self, card: &Card) {
        let count_in_deck = self.deck.card_count(card.code);
        if count_in_deck >= 3 {
            self.status_message = Some((format!("Cannot add more than 3 copies of '{}'", card.name), true));
            return;
        }

        if card.is_extra_deck_monster() {
            if self.deck.extra.len() >= 15 {
                self.status_message = Some(("Extra Deck maximum is 15 cards".into(), true));
                return;
            }
            self.deck.extra.push(card.code);
            self.status_message = Some((format!("Added '{}' to Extra Deck", card.name), false));
        } else {
            if self.deck.main.len() >= 60 {
                self.status_message = Some(("Main Deck maximum is 60 cards".into(), true));
                return;
            }
            self.deck.main.push(card.code);
            self.status_message = Some((format!("Added '{}' to Main Deck", card.name), false));
        }
    }

    /// Add a card explicitly into the Side Deck.
    pub fn add_card_to_side(&mut self, card: &Card) {
        let count_in_deck = self.deck.card_count(card.code);
        if count_in_deck >= 3 {
            self.status_message = Some((format!("Cannot add more than 3 copies of '{}'", card.name), true));
            return;
        }

        if self.deck.side.len() >= 15 {
            self.status_message = Some(("Side Deck maximum is 15 cards".into(), true));
            return;
        }
        self.deck.side.push(card.code);
        self.status_message = Some((format!("Added '{}' to Side Deck", card.name), false));
    }

    /// Remove a card from a specific zone (0 = Main, 1 = Extra, 2 = Side).
    pub fn remove_card(&mut self, code: u32, zone: u8) {
        let list = match zone {
            1 => &mut self.deck.extra,
            2 => &mut self.deck.side,
            _ => &mut self.deck.main,
        };
        if let Some(pos) = list.iter().position(|&c| c == code) {
            list.remove(pos);
        }
    }

    /// Sort the deck according to standard tournament rules:
    /// Monsters (Level desc -> Name) -> Spells (Name) -> Traps (Name)
    pub fn sort_deck(&mut self, db: &CardDatabase) {
        let sort_section = |list: &mut Vec<u32>| {
            list.sort_by(|&a, &b| {
                let ca = db.get(a);
                let cb = db.get(b);
                match (ca, cb) {
                    (Some(ca), Some(cb)) => {
                        let type_order = |c: &Card| {
                            if c.is_monster() { 0 }
                            else if c.is_spell() { 1 }
                            else { 2 }
                        };
                        let ta = type_order(ca);
                        let tb = type_order(cb);
                        if ta != tb {
                            return ta.cmp(&tb);
                        }
                        if ca.is_monster() && cb.is_monster() {
                            if ca.level != cb.level {
                                return cb.level.cmp(&ca.level); // higher level first
                            }
                        }
                        ca.name.cmp(&cb.name)
                    }
                    _ => a.cmp(&b),
                }
            });
        };

        sort_section(&mut self.deck.main);
        sort_section(&mut self.deck.extra);
        sort_section(&mut self.deck.side);
        self.status_message = Some(("Sorted deck by official categories".into(), false));
    }

    /// Draw a test 5-card opening hand.
    pub fn draw_test_hand(&mut self) {
        if self.deck.main.len() < 5 {
            self.status_message = Some(("Main Deck must have at least 5 cards to draw a test hand".into(), true));
            return;
        }
        let mut shuffled = self.deck.main.clone();
        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x12345678_9abcdef0);
        let mut rng = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        for i in (1..shuffled.len()).rev() {
            let j = (rng() as usize) % (i + 1);
            shuffled.swap(i, j);
        }
        self.test_hand_cards = shuffled.into_iter().take(5).collect();
        self.test_hand_open = true;
    }

    /// Draw 1 additional card in test hand.
    pub fn draw_one_more(&mut self) {
        if self.test_hand_cards.len() < self.deck.main.len() {
            let used: std::collections::HashSet<usize> = self.test_hand_cards.iter().map(|&c| c as usize).collect();
            for &code in &self.deck.main {
                if !used.contains(&(code as usize)) {
                    self.test_hand_cards.push(code);
                    break;
                }
            }
        }
    }

    /// Save the current deck to `decks/<deck_name>.ydk`.
    pub fn save_to_ydk(&mut self) {
        let name = self.deck_name.trim();
        if name.is_empty() {
            self.status_message = Some(("Deck name cannot be empty".into(), true));
            return;
        }
        let dir = Path::new("decks");
        if let Err(e) = std::fs::create_dir_all(dir) {
            self.status_message = Some((format!("Failed to create decks dir: {e}"), true));
            return;
        }
        let safe_name: String = name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' || c == ' ' { c } else { '_' })
            .collect();
        let path = dir.join(format!("{safe_name}.ydk"));
        self.deck.name = name.to_string();
        match self.deck.save_ydk(&path) {
            Ok(_) => {
                self.status_message = Some((format!("Saved deck to {}", path.display()), false));
            }
            Err(e) => {
                self.status_message = Some((format!("Failed to save deck: {e}"), true));
            }
        }
    }

    /// Load a deck from a `.ydk` file.
    pub fn load_from_ydk(&mut self, path: &Path) {
        match Deck::load_ydk(path) {
            Ok(loaded) => {
                self.deck_name = loaded.name.clone();
                self.deck = loaded;
                self.status_message = Some((
                    format!(
                        "Loaded '{}' (Main: {}, Extra: {}, Side: {})",
                        self.deck_name,
                        self.deck.main.len(),
                        self.deck.extra.len(),
                        self.deck.side.len()
                    ),
                    false,
                ));
            }
            Err(e) => {
                self.status_message = Some((format!("Failed to load deck: {e}"), true));
            }
        }
    }
}

/// Scans the `decks/` folder for `.ydk` files.
fn scan_deck_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let dir = Path::new("decks");
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e == "ydk").unwrap_or(false) {
                files.push(p);
            }
        }
    }
    files.sort();
    files
}

/// Renders the complete Master Duel Deck Builder UI.
pub fn render_deck_builder(
    ui: &mut Ui,
    state: &mut DeckBuilderState,
    db: &CardDatabase,
    banlists: &BanlistRegistry,
    rel_idx: &ReleaseDateIndex,
    start_duel_requested: &mut bool,
) {
    let screen_w = ui.available_width();
    let screen_h = ui.available_height();

    // 1. TOP HEADER BAR: Deck Name, Format Legality, Save/Load, and Actions
    ui.horizontal(|ui| {
        ui.heading(RichText::new("🃏 DECK BUILDER").color(ACCENT_GOLD).size(20.0).strong());
        ui.separator();

        // Deck Name input with modern styling
        ui.label(RichText::new("Deck:").color(TEXT_MUTED));
        ui.add(egui::TextEdit::singleline(&mut state.deck_name).desired_width(140.0));
        if ui.button("💾 Save").clicked() {
            state.save_to_ydk();
        }

        // Preset / Load Dropdown
        egui::ComboBox::from_id_salt("load_deck_combo")
            .selected_text("📂 Load...")
            .width(130.0)
            .show_ui(ui, |ui| {
                if ui.selectable_label(false, "Ishizu Tearlaments (2023 TCG)").clicked() {
                    state.deck = ygopro_data::builtin::tearlaments_ishizu_deck();
                    state.deck_name = state.deck.name.clone();
                    state.status_message = Some(("Loaded Ishizu Tearlaments fixture".into(), false));
                }
                let deck_files = scan_deck_files();
                if !deck_files.is_empty() {
                    ui.separator();
                    for path in &deck_files {
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown");
                        if ui.selectable_label(false, stem).clicked() {
                            state.load_from_ydk(path);
                        }
                    }
                }
            });

        ui.separator();

        // Banlist Selector
        ui.label(RichText::new("Banlist:").color(TEXT_MUTED));
        egui::ComboBox::from_id_salt("banlist_combo")
            .selected_text(RichText::new(&state.selected_banlist).color(ACCENT_CYAN))
            .width(150.0)
            .show_ui(ui, |ui| {
                for name in banlists.names() {
                    ui.selectable_value(&mut state.selected_banlist, name.clone(), name);
                }
            });

        // Date Display
        ui.label(RichText::new(format!("📅 {}", state.target_date)).color(TEXT_MUTED));

        ui.separator();

        // Real-time Legality Check Badge
        let query = BanlistQuery::tcg(state.target_date);
        let validator = DeckValidator::new(db).with_release_dates(rel_idx);
        if let Ok(active) = banlists.resolve(&query) {
            let report = validator.validate(&state.deck, &active);
            if report.is_legal {
                ui.label(RichText::new("✔ LEGAL").color(ACCENT_GREEN).strong());
            } else {
                let count = report.violations.len();
                if ui.button(RichText::new(format!("⚠ ILLEGAL ({count})")).color(ACCENT_RED).strong()).clicked() {
                    state.violations_dialog_open = true;
                }
            }
        }

        // Action Buttons on Right
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(RichText::new("⚔️ Duel with Deck").color(Color32::WHITE).strong()).clicked() {
                *start_duel_requested = true;
            }
            if ui.button("🎲 Test Hand").clicked() {
                state.draw_test_hand();
            }
            if ui.button("✨ Sort Deck").clicked() {
                state.sort_deck(db);
            }
            if ui.button("📤 Export").clicked() {
                state.export_dialog_open = true;
            }
            if ui.button("📥 Import").clicked() {
                state.import_dialog_open = true;
            }
            if ui.button("🧹 Clear").clicked() {
                state.deck.main.clear();
                state.deck.extra.clear();
                state.deck.side.clear();
                state.status_message = Some(("Cleared deck".into(), false));
            }
        });
    });

    ui.separator();

    // 2. DECK STATISTICS BAR (Monsters / Spells / Traps ratio progress bar)
    let monsters_count = state.deck.main.iter().filter(|&&c| db.get(c).map(|cd| cd.is_monster()).unwrap_or(false)).count();
    let spells_count = state.deck.main.iter().filter(|&&c| db.get(c).map(|cd| cd.is_spell()).unwrap_or(false)).count();
    let traps_count = state.deck.main.iter().filter(|&&c| db.get(c).map(|cd| cd.is_trap()).unwrap_or(false)).count();
    let total_main = state.deck.main.len();

    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("Main: {total_main}/60")).strong().color(TEXT_PRIMARY));
        ui.label(RichText::new(format!("Extra: {}/15", state.deck.extra.len())).color(ACCENT_PURPLE));
        ui.label(RichText::new(format!("Side: {}/15", state.deck.side.len())).color(ACCENT_CYAN));
        ui.separator();

        ui.label(RichText::new(format!("🐉 Monsters: {monsters_count}")).color(COLOR_EFFECT_MONSTER));
        ui.label(RichText::new(format!("✨ Spells: {spells_count}")).color(COLOR_SPELL));
        ui.label(RichText::new(format!("📜 Traps: {traps_count}")).color(COLOR_TRAP));

        if let Some((msg, is_err)) = &state.status_message {
            let color = if *is_err { ACCENT_RED } else { ACCENT_GREEN };
            ui.separator();
            ui.label(RichText::new(msg).color(color));
        }
    });

    ui.separator();

    // 3. THREE-COLUMN LAYOUT: Inspector (Left), Deck Grid (Center), Catalog & Search (Right)
    let inspector_w = 260.0;
    let catalog_w = 380.0;
    let deck_w = (screen_w - inspector_w - catalog_w - 30.0).max(420.0);

    ui.horizontal(|ui| {
        // --- COLUMN 1: MASTER DUEL CARD INSPECTOR ---
        ui.allocate_ui(Vec2::new(inspector_w, screen_h - 90.0), |ui| {
            ui.vertical(|ui| {
                let mut card_to_add_main = None;
                let mut card_to_add_side = None;
                let mut card_to_remove = None;

                if let Some(card) = &state.selected_card {
                    draw_card_item(ui, card, inspector_w - 10.0, 310.0, false);
                    ui.add_space(6.0);

                    // Add / Remove action buttons
                    ui.horizontal(|ui| {
                        let btn_label = if card.is_extra_deck_monster() { "➕ Extra" } else { "➕ Main" };
                        if ui.button(btn_label).clicked() {
                            card_to_add_main = Some(card.clone());
                        }
                        if ui.button("➕ Side").clicked() {
                            card_to_add_side = Some(card.clone());
                        }
                        if state.deck.card_count(card.code) > 0 {
                            if ui.button("✖ Remove").clicked() {
                                card_to_remove = Some(card.code);
                            }
                        }
                    });

                    ui.add_space(4.0);
                    // Unique id_salt for inspector scroll
                    ScrollArea::vertical()
                        .id_salt("inspector_desc_scroll")
                        .max_height(screen_h - 470.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new(&card.name).strong().color(TEXT_PRIMARY));
                            ui.label(RichText::new(format!("Passcode: {:08}", card.code)).color(TEXT_MUTED).size(11.0));
                            if card.is_monster() {
                                let attr = attribute_label(card.attribute);
                                ui.label(
                                    RichText::new(format!(
                                        "[{}] Level/Rank: {} | ATK: {} / DEF: {}",
                                        attr, card.level, card.attack, card.defense
                                    ))
                                    .color(ACCENT_GOLD),
                                );
                            }
                            ui.separator();
                            ui.label(RichText::new(&card.desc).size(12.0).color(TEXT_PRIMARY));
                        });
                } else {
                    ui.add_space(40.0);
                    ui.label(RichText::new("Select any card in catalog or deck to inspect full details.").italics().color(TEXT_MUTED));
                }

                if let Some(card) = card_to_add_main {
                    state.add_card(&card);
                }
                if let Some(card) = card_to_add_side {
                    state.add_card_to_side(&card);
                }
                if let Some(code) = card_to_remove {
                    if state.deck.main.contains(&code) {
                        state.remove_card(code, 0);
                    } else if state.deck.extra.contains(&code) {
                        state.remove_card(code, 1);
                    } else {
                        state.remove_card(code, 2);
                    }
                }
            });
        });

        ui.separator();

        // --- COLUMN 2: ACTIVE DECK ZONES (MAIN, EXTRA, SIDE) ---
        ui.allocate_ui(Vec2::new(deck_w, screen_h - 90.0), |ui| {
            // Unique id_salt for deck zones scroll
            ScrollArea::vertical().id_salt("deck_zones_scroll").show(ui, |ui| {
                let card_w = 66.0;
                let card_h = 94.0;
                let cols = ((deck_w - 20.0) / (card_w + 6.0)).max(1.0) as usize;

                // 2.1 MAIN DECK SECTION
                ui.horizontal(|ui| {
                    ui.heading(RichText::new(format!("Main Deck ({}/60)", state.deck.main.len())).size(15.0).color(TEXT_PRIMARY));
                    ui.label(RichText::new("(Right-click card to remove)").color(TEXT_MUTED).size(11.0));
                });

                let mut to_remove_main = None;
                egui::Grid::new("main_deck_grid").spacing(Vec2::new(6.0, 6.0)).show(ui, |ui| {
                    for (i, &code) in state.deck.main.iter().enumerate() {
                        if let Some(card) = db.get(code) {
                            let resp = draw_card_item(ui, card, card_w, card_h, false);
                            if resp.clicked() {
                                state.selected_card = Some(card.clone());
                            }
                            if resp.secondary_clicked() {
                                to_remove_main = Some(code);
                            }
                        }
                        if (i + 1) % cols == 0 {
                            ui.end_row();
                        }
                    }
                });
                if let Some(code) = to_remove_main {
                    state.remove_card(code, 0);
                }

                ui.add_space(14.0);
                ui.separator();

                // 2.2 EXTRA DECK SECTION
                ui.horizontal(|ui| {
                    ui.heading(RichText::new(format!("Extra Deck ({}/15)", state.deck.extra.len())).size(15.0).color(ACCENT_PURPLE));
                });

                let mut to_remove_extra = None;
                egui::Grid::new("extra_deck_grid").spacing(Vec2::new(6.0, 6.0)).show(ui, |ui| {
                    for (i, &code) in state.deck.extra.iter().enumerate() {
                        if let Some(card) = db.get(code) {
                            let resp = draw_card_item(ui, card, card_w, card_h, false);
                            if resp.clicked() {
                                state.selected_card = Some(card.clone());
                            }
                            if resp.secondary_clicked() {
                                to_remove_extra = Some(code);
                            }
                        }
                        if (i + 1) % cols == 0 {
                            ui.end_row();
                        }
                    }
                });
                if let Some(code) = to_remove_extra {
                    state.remove_card(code, 1);
                }

                ui.add_space(14.0);
                ui.separator();

                // 2.3 SIDE DECK SECTION
                ui.horizontal(|ui| {
                    ui.heading(RichText::new(format!("Side Deck ({}/15)", state.deck.side.len())).size(15.0).color(ACCENT_CYAN));
                });

                let mut to_remove_side = None;
                egui::Grid::new("side_deck_grid").spacing(Vec2::new(6.0, 6.0)).show(ui, |ui| {
                    for (i, &code) in state.deck.side.iter().enumerate() {
                        if let Some(card) = db.get(code) {
                            let resp = draw_card_item(ui, card, card_w, card_h, false);
                            if resp.clicked() {
                                state.selected_card = Some(card.clone());
                            }
                            if resp.secondary_clicked() {
                                to_remove_side = Some(code);
                            }
                        }
                        if (i + 1) % cols == 0 {
                            ui.end_row();
                        }
                    }
                });
                if let Some(code) = to_remove_side {
                    state.remove_card(code, 2);
                }
            });
        });

        ui.separator();

        // --- COLUMN 3: CARD CATALOG, LIVE SEARCH & MULTI-FILTER ---
        ui.allocate_ui(Vec2::new(catalog_w, screen_h - 90.0), |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(RichText::new("Card Catalog").size(15.0).color(TEXT_PRIMARY));
                    ui.label(RichText::new("(Left-click inspect, Right-click add)").color(TEXT_MUTED).size(11.0));
                });

                // Search Box with clear button
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(egui::TextEdit::singleline(&mut state.search_query).desired_width(catalog_w - 70.0));
                    if ui.button("✖").clicked() {
                        state.search_query.clear();
                    }
                });

                // Category Filter Tabs Row 1
                ui.horizontal_wrapped(|ui| {
                    if ui.selectable_label(state.filter_type == 0, "All").clicked() {
                        state.filter_type = 0;
                        state.catalog_page = 0;
                    }
                    if ui.selectable_label(state.filter_type == TYPE_MONSTER, "Monster").clicked() {
                        state.filter_type = TYPE_MONSTER;
                        state.catalog_page = 0;
                    }
                    if ui.selectable_label(state.filter_type == TYPE_SPELL, "Spell").clicked() {
                        state.filter_type = TYPE_SPELL;
                        state.catalog_page = 0;
                    }
                    if ui.selectable_label(state.filter_type == TYPE_TRAP, "Trap").clicked() {
                        state.filter_type = TYPE_TRAP;
                        state.catalog_page = 0;
                    }
                    if ui.selectable_label(state.filter_type == TYPE_FUSION, "Fusion").clicked() {
                        state.filter_type = TYPE_FUSION;
                        state.catalog_page = 0;
                    }
                    if ui.selectable_label(state.filter_type == TYPE_SYNCHRO, "Synchro").clicked() {
                        state.filter_type = TYPE_SYNCHRO;
                        state.catalog_page = 0;
                    }
                    if ui.selectable_label(state.filter_type == TYPE_XYZ, "Xyz").clicked() {
                        state.filter_type = TYPE_XYZ;
                        state.catalog_page = 0;
                    }
                    if ui.selectable_label(state.filter_type == TYPE_LINK, "Link").clicked() {
                        state.filter_type = TYPE_LINK;
                        state.catalog_page = 0;
                    }
                });

                // Attribute & Level Filter Row 2
                ui.horizontal_wrapped(|ui| {
                    egui::ComboBox::from_id_salt("attribute_filter_combo")
                        .selected_text(if state.filter_attribute == 0 { "Attribute: All" } else { attribute_label(state.filter_attribute) })
                        .width(110.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.filter_attribute, 0, "All");
                            ui.selectable_value(&mut state.filter_attribute, ATTRIBUTE_DARK, "DARK");
                            ui.selectable_value(&mut state.filter_attribute, ATTRIBUTE_LIGHT, "LIGHT");
                            ui.selectable_value(&mut state.filter_attribute, ATTRIBUTE_WATER, "WATER");
                            ui.selectable_value(&mut state.filter_attribute, ATTRIBUTE_FIRE, "FIRE");
                            ui.selectable_value(&mut state.filter_attribute, ATTRIBUTE_EARTH, "EARTH");
                            ui.selectable_value(&mut state.filter_attribute, ATTRIBUTE_WIND, "WIND");
                            ui.selectable_value(&mut state.filter_attribute, ATTRIBUTE_DIVINE, "DIVINE");
                        });

                    let lvl_str = if state.filter_level == 0 { "Level: All".to_string() } else { format!("★ Level {}", state.filter_level) };
                    egui::ComboBox::from_id_salt("level_filter_combo")
                        .selected_text(&lvl_str)
                        .width(90.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.filter_level, 0, "All");
                            for lvl in 1..=12 {
                                ui.selectable_value(&mut state.filter_level, lvl, format!("★ {lvl}"));
                            }
                        });

                    if state.filter_type != 0 || state.filter_attribute != 0 || state.filter_level != 0 || !state.search_query.is_empty() {
                        if ui.button("Reset Filters").clicked() {
                            state.filter_type = 0;
                            state.filter_attribute = 0;
                            state.filter_level = 0;
                            state.search_query.clear();
                            state.catalog_page = 0;
                        }
                    }
                });

                ui.separator();

                // Catalog Query Execution
                let results: Vec<Card> = if let Ok(passcode) = state.search_query.trim().parse::<u32>() {
                    if let Some(card) = db.get(passcode) {
                        vec![card.clone()]
                    } else {
                        Vec::new()
                    }
                } else {
                    let mut filter = CardFilter::default();
                    if !state.search_query.is_empty() {
                        filter.name_contains = Some(state.search_query.clone());
                    }
                    if state.filter_type != 0 {
                        filter.type_any = state.filter_type;
                    }
                    if state.filter_attribute != 0 {
                        filter.attribute_any = state.filter_attribute;
                    }

                    let raw: Vec<Card> = if state.search_query.is_empty() && state.filter_type == 0 && state.filter_attribute == 0 {
                        db.iter().take(300).cloned().collect()
                    } else {
                        db.query(&filter).into_iter().take(300).cloned().collect()
                    };

                    if state.filter_level != 0 {
                        raw.into_iter().filter(|c| c.level == state.filter_level).collect()
                    } else {
                        raw
                    }
                };

                let total_results = results.len();
                let start_idx = state.catalog_page * state.page_size;
                let page_cards: Vec<Card> = results.into_iter().skip(start_idx).take(state.page_size).collect();

                // Pagination Controls
                ui.horizontal(|ui| {
                    if ui.button("◀ Prev").clicked() && state.catalog_page > 0 {
                        state.catalog_page -= 1;
                    }
                    ui.label(RichText::new(format!("Page {} ({} results)", state.catalog_page + 1, total_results)).color(TEXT_MUTED));
                    if ui.button("Next ▶").clicked() && (start_idx + state.page_size) < total_results {
                        state.catalog_page += 1;
                    }
                });

                // Unique id_salt for catalog grid scroll
                ScrollArea::vertical().id_salt("catalog_search_scroll").show(ui, |ui| {
                    let cat_w = 64.0;
                    let cat_h = 90.0;
                    let cat_cols = ((catalog_w - 20.0) / (cat_w + 6.0)).max(1.0) as usize;

                    egui::Grid::new("catalog_cards_grid").spacing(Vec2::new(6.0, 6.0)).show(ui, |ui| {
                        for (i, card) in page_cards.iter().enumerate() {
                            let resp = draw_card_item(ui, card, cat_w, cat_h, false);
                            if resp.clicked() {
                                state.selected_card = Some(card.clone());
                            }
                            if resp.secondary_clicked() {
                                state.add_card(card);
                            }
                            if (i + 1) % cat_cols == 0 {
                                ui.end_row();
                            }
                        }
                    });
                });
            });
        });
    });

    // --- TEST OPENING HAND MODAL (Master Duel Draw 5) ---
    if state.test_hand_open {
        egui::Window::new("🎲 Test Opening Hand Simulator")
            .fixed_size(Vec2::new(560.0, 240.0))
            .show(ui.ctx(), |ui| {
                ui.label(RichText::new(format!("Drew {} cards from Main Deck:", state.test_hand_cards.len())).color(TEXT_PRIMARY));
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    for &code in &state.test_hand_cards {
                        if let Some(card) = db.get(code) {
                            draw_card_item(ui, card, 75.0, 105.0, false);
                        }
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("🎲 Redraw Hand (5)").clicked() {
                        state.draw_test_hand();
                    }
                    if ui.button("➕ Draw 1 More").clicked() {
                        state.draw_one_more();
                    }
                    if ui.button("Close").clicked() {
                        state.test_hand_open = false;
                    }
                });
            });
    }

    // --- UNIVERSAL IMPORT DIALOG ---
    if state.import_dialog_open {
        egui::Window::new("📥 Universal Deck Import")
            .fixed_size(Vec2::new(520.0, 420.0))
            .show(ui.ctx(), |ui| {
                ui.label("Paste any deck format: .ydk contents, ydke:// URL, or plaintext card list:");
                ui.text_edit_multiline(&mut state.import_text);
                ui.horizontal(|ui| {
                    if ui.button("Import & Auto-Detect").clicked() {
                        let text = state.import_text.trim();
                        let result = if text.starts_with("ydke://") {
                            Deck::from_ydke(text).map_err(|e| format!("{e}"))
                        } else if text.contains("#main") || text.contains("#extra") || text.contains("!side") {
                            Deck::parse_ydk(text).map_err(|e| format!("{e}"))
                        } else {
                            parse_plaintext_deck(text, db).map_err(|e| format!("{e}"))
                        };

                        match result {
                            Ok(imported) => {
                                state.deck = imported;
                                state.deck_name = if state.deck.name.is_empty() { "Imported Deck".into() } else { state.deck.name.clone() };
                                state.status_message = Some((
                                    format!(
                                        "Imported deck: {} Main, {} Extra, {} Side",
                                        state.deck.main.len(),
                                        state.deck.extra.len(),
                                        state.deck.side.len()
                                    ),
                                    false,
                                ));
                                state.import_dialog_open = false;
                            }
                            Err(e) => {
                                state.status_message = Some((format!("Import failed: {e}"), true));
                            }
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        state.import_dialog_open = false;
                    }
                });
            });
    }

    // --- EXPORT / SHARE DIALOG ---
    if state.export_dialog_open {
        egui::Window::new("📤 Export / Share Deck")
            .fixed_size(Vec2::new(520.0, 420.0))
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut state.export_format, 0, "YDK File Format");
                    ui.selectable_value(&mut state.export_format, 1, "YDKE URL");
                    ui.selectable_value(&mut state.export_format, 2, "Card Name Plaintext");
                });

                ui.separator();

                let mut export_text = match state.export_format {
                    0 => state.deck.to_ydk(),
                    1 => state.deck.to_ydke(),
                    _ => {
                        let mut pt = String::new();
                        pt.push_str(&format!("== MAIN DECK ({} cards) ==\n", state.deck.main.len()));
                        for &c in &state.deck.main {
                            if let Some(card) = db.get(c) {
                                pt.push_str(&format!("{}\n", card.name));
                            }
                        }
                        if !state.deck.extra.is_empty() {
                            pt.push_str(&format!("\n== EXTRA DECK ({} cards) ==\n", state.deck.extra.len()));
                            for &c in &state.deck.extra {
                                if let Some(card) = db.get(c) {
                                    pt.push_str(&format!("{}\n", card.name));
                                }
                            }
                        }
                        if !state.deck.side.is_empty() {
                            pt.push_str(&format!("\n== SIDE DECK ({} cards) ==\n", state.deck.side.len()));
                            for &c in &state.deck.side {
                                if let Some(card) = db.get(c) {
                                    pt.push_str(&format!("{}\n", card.name));
                                }
                            }
                        }
                        pt
                    }
                };

                ui.text_edit_multiline(&mut export_text);

                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        state.export_dialog_open = false;
                    }
                });
            });
    }

    // --- FORMAT LEGALITY VIOLATIONS DIALOG ---
    if state.violations_dialog_open {
        egui::Window::new("⚠ Format Legality Breakdown")
            .fixed_size(Vec2::new(480.0, 320.0))
            .show(ui.ctx(), |ui| {
                let query = BanlistQuery::tcg(state.target_date);
                let validator = DeckValidator::new(db).with_release_dates(rel_idx);
                if let Ok(active) = banlists.resolve(&query) {
                    let report = validator.validate(&state.deck, &active);
                    if report.is_legal {
                        ui.label(RichText::new("✔ This deck is fully legal under the selected format and date.").color(ACCENT_GREEN));
                    } else {
                        ui.label(RichText::new(format!("Deck has {} legality violation(s):", report.violations.len())).color(ACCENT_RED).strong());
                        ui.separator();
                        // Unique id_salt for violations list scroll
                        ScrollArea::vertical().id_salt("violations_breakdown_scroll").show(ui, |ui| {
                            for v in &report.violations {
                                ui.label(format!("• {v:?}"));
                            }
                        });
                    }
                }
                ui.separator();
                if ui.button("Close").clicked() {
                    state.violations_dialog_open = false;
                }
            });
    }
}
