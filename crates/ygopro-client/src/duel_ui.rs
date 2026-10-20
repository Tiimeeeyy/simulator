//! Master Duel inspired interactive Duel Board, Action Coordinator, and Chain Toggle system.

use std::sync::Arc;
use bevy_egui::egui::{
    self, Color32, Layout, RichText, ScrollArea, Stroke, Ui, Vec2,
};
use ocgcore_sys::*;
use ygopro_bot::{GenericAgent, TearlamentsPolicy};
use ygopro_core::{
    BattleCmdChoice, CoreMessage, Duel, DuelConfig, IdleCmdChoice, PlayerResponse, ScriptLoader,
};
use ygopro_data::{CardDatabase, Deck};
use ygopro_host::{DuelSession, PlayerController, StepOutcome};

use crate::card_widget::draw_card_item;
use crate::theme::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BotOpponentType {
    #[default]
    IshizuTearlaments,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivationToggle {
    #[default]
    Auto,
    On,
    Off,
}

pub struct DuelUiState {
    pub session: Option<DuelSession>,
    pub bot_type: BotOpponentType,
    pub activation_toggle: ActivationToggle,
    pub duel_log: Vec<String>,
    pub auto_step: bool,
    pub active_prompt: Option<CoreMessage>,
    pub selected_hand_code: Option<u32>,
    pub winner: Option<u8>,
    pub my_lp: u32,
    pub opp_lp: u32,
    pub turn_count: u32,
    pub current_phase: u16,
    pub my_hand: Vec<u32>,
    pub opp_hand_count: usize,
    pub my_mzone: Vec<Option<u32>>,
    pub opp_mzone: Vec<Option<u32>>,
    pub my_szone: Vec<Option<u32>>,
    pub opp_szone: Vec<Option<u32>>,
    pub my_grave: Vec<u32>,
    pub opp_grave: Vec<u32>,
    pub active_chains: Vec<(u32, u8, u32)>, // (code, player, chain_link)
    pub view_pile: Option<(String, Vec<u32>)>, // title, cards
}

impl DuelUiState {
    pub fn new() -> Self {
        Self {
            session: None,
            bot_type: BotOpponentType::IshizuTearlaments,
            activation_toggle: ActivationToggle::Auto,
            duel_log: Vec::new(),
            auto_step: true,
            active_prompt: None,
            selected_hand_code: None,
            winner: None,
            my_lp: 8000,
            opp_lp: 8000,
            turn_count: 1,
            current_phase: 1,
            my_hand: Vec::new(),
            opp_hand_count: 5,
            my_mzone: vec![None; 5],
            opp_mzone: vec![None; 5],
            my_szone: vec![None; 5],
            opp_szone: vec![None; 5],
            my_grave: Vec::new(),
            opp_grave: Vec::new(),
            active_chains: Vec::new(),
            view_pile: None,
        }
    }

    /// Initialize a new match between human and selected bot.
    pub fn start_match(
        &mut self,
        deck: &Deck,
        db: Arc<CardDatabase>,
        script_loader: Arc<ScriptLoader>,
    ) {
        let config = DuelConfig::default();
        let mut duel = match Duel::create(config, Arc::clone(&db), Arc::clone(&script_loader)) {
            Ok(d) => d,
            Err(e) => {
                self.duel_log.push(format!("Failed to create duel: {e}"));
                return;
            }
        };

        // Player 0 = Human, Player 1 = Bot
        duel.load_deck(0, deck);
        let bot_deck = ygopro_data::builtin::tearlaments_ishizu_deck();
        duel.load_deck(1, &bot_deck);

        let p0 = PlayerController::Human { name: "Player".into() };
        let p1: PlayerController = match self.bot_type {
            BotOpponentType::IshizuTearlaments => {
                PlayerController::Bot(Box::new(TearlamentsPolicy::new(1)))
            }
            BotOpponentType::Generic => {
                PlayerController::Bot(Box::new(GenericAgent::new(1)))
            }
        };

        let session = DuelSession::new(duel, db, p0, p1);
        self.session = Some(session);
        self.duel_log.clear();
        self.duel_log.push("⚔️ Duel started! Master Duel ruleset active.".into());
        self.my_lp = 8000;
        self.opp_lp = 8000;
        self.my_hand.clear();
        self.opp_hand_count = 5;
        self.my_mzone = vec![None; 5];
        self.opp_mzone = vec![None; 5];
        self.my_szone = vec![None; 5];
        self.opp_szone = vec![None; 5];
        self.my_grave.clear();
        self.opp_grave.clear();
        self.active_chains.clear();
        self.winner = None;
    }

    /// Process incoming engine messages and update UI state.
    pub fn handle_messages(&mut self, msgs: &[CoreMessage], db: &CardDatabase) {
        for m in msgs {
            match m {
                CoreMessage::NewTurn { player } => {
                    self.turn_count += 1;
                    let p_str = if *player == 0 { "Player" } else { "Opponent" };
                    self.duel_log.push(format!("--- Turn {} (Turn Player: {p_str}) ---", self.turn_count));
                }
                CoreMessage::NewPhase { phase } => {
                    self.current_phase = *phase;
                }
                CoreMessage::Draw { player, cards, .. } => {
                    if *player == 0 {
                        for &c in cards {
                            if c != 0 {
                                self.my_hand.push(c);
                                let name = db.get(c).map(|cd| cd.name.as_str()).unwrap_or("Card");
                                self.duel_log.push(format!("Player drew '{name}'"));
                            }
                        }
                    } else {
                        self.opp_hand_count += cards.len();
                        self.duel_log.push(format!("Opponent drew {} card(s)", cards.len()));
                    }
                }
                CoreMessage::Chaining { code, con, chain_link, .. } => {
                    let name = db.get(*code).map(|c| c.name.as_str()).unwrap_or("Card");
                    let p_str = if *con == 0 { "Player" } else { "Opponent" };
                    self.active_chains.push((*code, *con, *chain_link as u32));
                    self.duel_log.push(format!("⚡ Chain Link {chain_link}: {p_str} activated '{name}'"));
                }
                CoreMessage::ChainSolving { chain_link } => {
                    self.duel_log.push(format!("✨ Resolving Chain Link {chain_link}..."));
                }
                CoreMessage::ChainEnd => {
                    self.active_chains.clear();
                }
                CoreMessage::Damage { player, amount } => {
                    if *player == 0 {
                        self.my_lp = self.my_lp.saturating_sub(*amount);
                        self.duel_log.push(format!("💥 Player took {amount} damage (LP: {})", self.my_lp));
                    } else {
                        self.opp_lp = self.opp_lp.saturating_sub(*amount);
                        self.duel_log.push(format!("💥 Opponent took {amount} damage (LP: {})", self.opp_lp));
                    }
                }
                CoreMessage::Recover { player, amount } => {
                    if *player == 0 {
                        self.my_lp = self.my_lp.saturating_add(*amount);
                    } else {
                        self.opp_lp = self.opp_lp.saturating_add(*amount);
                    }
                }
                CoreMessage::Win { player, reason: _ } => {
                    self.winner = Some(*player);
                    let victor = if *player == 0 { "Player" } else { "Opponent" };
                    self.duel_log.push(format!("🏆 Duel Ended! {victor} Wins!"));
                }
                CoreMessage::Move {
                    code,
                    from_con,
                    from_loc,
                    from_seq: _,
                    to_con,
                    to_loc,
                    to_seq,
                    ..
                } => {
                    let code = *code;
                    let from_con = *from_con;
                    let from_loc = *from_loc as u32;
                    let to_con = *to_con;
                    let to_loc = *to_loc as u32;
                    let to_seq = *to_seq as usize;

                    // Remove from hand if moved
                    if from_con == 0 && from_loc == LOCATION_HAND {
                        if let Some(pos) = self.my_hand.iter().position(|&c| c == code) {
                            self.my_hand.remove(pos);
                        }
                    } else if from_con == 1 && from_loc == LOCATION_HAND {
                        self.opp_hand_count = self.opp_hand_count.saturating_sub(1);
                    }

                    // Add to field
                    if to_con == 0 {
                        if to_loc == LOCATION_MZONE && to_seq < 5 {
                            self.my_mzone[to_seq] = Some(code);
                        } else if to_loc == LOCATION_SZONE && to_seq < 5 {
                            self.my_szone[to_seq] = Some(code);
                        } else if to_loc == LOCATION_GRAVE {
                            self.my_grave.push(code);
                        }
                    } else {
                        if to_loc == LOCATION_MZONE && to_seq < 5 {
                            self.opp_mzone[to_seq] = Some(code);
                        } else if to_loc == LOCATION_SZONE && to_seq < 5 {
                            self.opp_szone[to_seq] = Some(code);
                        } else if to_loc == LOCATION_GRAVE {
                            self.opp_grave.push(code);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn render_duel_screen(
    ui: &mut Ui,
    state: &mut DuelUiState,
    db: &CardDatabase,
    return_to_menu: &mut bool,
) {
    let screen_w = ui.available_width();
    let screen_h = ui.available_height();

    // Advance engine step if auto-stepping and no prompt is awaiting input
    if state.auto_step && state.active_prompt.is_none() && state.session.is_some() && state.winner.is_none() {
        if let Some(ref mut session) = state.session {
            match session.step() {
                Ok(outcome) => match outcome {
                    StepOutcome::Continue(msgs) => {
                        state.handle_messages(&msgs, db);
                    }
                    StepOutcome::BotActed { response, messages, .. } => {
                        state.handle_messages(&messages, db);
                        state.duel_log.push(format!("🤖 Bot response: {response:?}"));
                    }
                    StepOutcome::AwaitingHuman { prompt, messages, .. } => {
                        state.handle_messages(&messages, db);
                        state.active_prompt = Some(*prompt);
                    }
                    StepOutcome::Ended { winner, messages } => {
                        state.handle_messages(&messages, db);
                        state.winner = winner;
                    }
                },
                Err(e) => {
                    state.duel_log.push(format!("Engine error: {e}"));
                }
            }
        }
    }

    // 1. TOP HEADER: MASTER DUEL LIFE POINTS GAUGES & PHASE BAR
    ui.horizontal(|ui| {
        // Player LP Gauge
        ui.horizontal(|ui| {
            ui.label(RichText::new("👤 YOU").color(ACCENT_BLUE).strong());
            let lp_color = if state.my_lp > 4000 { ACCENT_GREEN } else if state.my_lp > 2000 { ACCENT_GOLD } else { ACCENT_RED };
            ui.label(RichText::new(format!("LP {}", state.my_lp)).size(18.0).color(lp_color).strong());
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        // Master Duel Phase Strip: [ DP ] [ SP ] [ M1 ] [ BP ] [ M2 ] [ EP ]
        let phases = [
            (1, "DP", "Draw Phase"),
            (2, "SP", "Standby Phase"),
            (4, "M1", "Main Phase 1"),
            (8, "BP", "Battle Phase"),
            (16, "M2", "Main Phase 2"),
            (512, "EP", "End Phase"),
        ];

        ui.label(RichText::new(format!("TURN {}", state.turn_count)).color(TEXT_MUTED).strong());
        ui.add_space(8.0);

        for (mask, short, name) in phases {
            let is_current = (state.current_phase & mask) != 0;
            let (bg, text_col) = if is_current {
                (ACCENT_CYAN, Color32::BLACK)
            } else {
                (Color32::from_rgb(26, 32, 48), TEXT_MUTED)
            };

            let badge = egui::Button::new(RichText::new(short).strong().color(text_col))
                .fill(bg)
                .corner_radius(3.0);
            ui.add(badge).on_hover_text(name);
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        // Opponent LP Gauge
        ui.horizontal(|ui| {
            ui.label(RichText::new("🤖 AI").color(ACCENT_RED).strong());
            let opp_lp_col = if state.opp_lp > 4000 { ACCENT_GREEN } else if state.opp_lp > 2000 { ACCENT_GOLD } else { ACCENT_RED };
            ui.label(RichText::new(format!("LP {}", state.opp_lp)).size(18.0).color(opp_lp_col).strong());
        });

        // Top Right Controls (Leave duel & Manual step)
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Leave Match").clicked() {
                *return_to_menu = true;
            }
            if ui.button("Step ▶").clicked() && state.active_prompt.is_none() && state.session.is_some() {
                if let Some(ref mut session) = state.session {
                    if let Ok(outcome) = session.step() {
                        match outcome {
                            StepOutcome::Continue(msgs) => state.handle_messages(&msgs, db),
                            StepOutcome::BotActed { messages, .. } => state.handle_messages(&messages, db),
                            StepOutcome::AwaitingHuman { prompt, messages, .. } => {
                                state.handle_messages(&messages, db);
                                state.active_prompt = Some(*prompt);
                            }
                            StepOutcome::Ended { winner, messages } => {
                                state.handle_messages(&messages, db);
                                state.winner = winner;
                            }
                        }
                    }
                }
            }
            ui.checkbox(&mut state.auto_step, "Auto-Step");
        });
    });

    ui.separator();

    // Active Chain Links Display (Master Duel Chain Stack)
    if !state.active_chains.is_empty() {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚡ ACTIVE CHAIN:").color(ACCENT_GOLD).strong());
            for (code, con, link) in &state.active_chains {
                let name = db.get(*code).map(|c| c.name.as_str()).unwrap_or("Card");
                let badge_col = if *con == 0 { ACCENT_BLUE } else { ACCENT_RED };
                ui.label(RichText::new(format!("[Link {link}: {name}]")).color(badge_col).strong());
            }
        });
        ui.separator();
    }

    let field_w = screen_w - 340.0;
    let log_w = 320.0;

    ui.horizontal(|ui| {
        // --- DUEL FIELD BOARD (LEFT) ---
        ui.allocate_ui(Vec2::new(field_w, screen_h - 75.0), |ui| {
            ui.vertical(|ui| {
                // 1. OPPONENT STATUS & HAND
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Opponent (AI)").strong().color(ACCENT_RED));
                    ui.label(RichText::new(format!("Hand: {} cards", state.opp_hand_count)).color(TEXT_MUTED));
                    if ui.button(format!("GY ({})", state.opp_grave.len())).clicked() {
                        state.view_pile = Some(("Opponent Graveyard".into(), state.opp_grave.clone()));
                    }
                });

                // Opponent facedown cards in hand
                ui.horizontal(|ui| {
                    for _ in 0..state.opp_hand_count.min(12) {
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(45.0, 65.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 3.0, Color32::from_rgb(130, 40, 40));
                        ui.painter().rect_stroke(rect, 3.0, Stroke::new(1.0, Color32::from_black_alpha(180)), egui::StrokeKind::Inside);
                    }
                });

                ui.add_space(6.0);

                // 2. OPPONENT FIELD (S/T row, then Monster row)
                let zone_w = 70.0;
                let zone_h = 96.0;

                // Opponent Spell/Trap Zones
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Opp S/T:").color(TEXT_MUTED).size(11.0));
                    for seq in (0..5).rev() {
                        let code = state.opp_szone[seq];
                        render_zone_slot(ui, code, zone_w, zone_h, db);
                    }
                });

                ui.add_space(4.0);

                // Opponent Monster Zones
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Opp MZone:").color(TEXT_MUTED).size(11.0));
                    for seq in (0..5).rev() {
                        let code = state.opp_mzone[seq];
                        render_zone_slot(ui, code, zone_w, zone_h, db);
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // 3. PLAYER FIELD (Monster row, then S/T row)
                // Player Monster Zones
                ui.horizontal(|ui| {
                    ui.label(RichText::new("You MZone:").color(TEXT_MUTED).size(11.0));
                    for seq in 0..5 {
                        let code = state.my_mzone[seq];
                        render_zone_slot(ui, code, zone_w, zone_h, db);
                    }
                });

                ui.add_space(4.0);

                // Player Spell/Trap Zones
                ui.horizontal(|ui| {
                    ui.label(RichText::new("You S/T:").color(TEXT_MUTED).size(11.0));
                    for seq in 0..5 {
                        let code = state.my_szone[seq];
                        render_zone_slot(ui, code, zone_w, zone_h, db);
                    }
                });

                ui.add_space(8.0);
                ui.separator();

                // 4. PLAYER HAND & MASTER DUEL ACTIVATION TOGGLE
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Your Hand ({} cards):", state.my_hand.len())).color(TEXT_PRIMARY).strong());
                    if ui.button(format!("GY ({})", state.my_grave.len())).clicked() {
                        state.view_pile = Some(("Your Graveyard".into(), state.my_grave.clone()));
                    }

                    // MASTER DUEL ACTIVATION TOGGLE (Bottom-right control)
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (toggle_txt, toggle_col) = match state.activation_toggle {
                            ActivationToggle::Auto => ("⚡ CHAIN: AUTO", ACCENT_CYAN),
                            ActivationToggle::On => ("⚡ CHAIN: ON (ALWAYS ASK)", ACCENT_GOLD),
                            ActivationToggle::Off => ("⚡ CHAIN: OFF", TEXT_MUTED),
                        };
                        if ui.button(RichText::new(toggle_txt).color(toggle_col).strong()).clicked() {
                            state.activation_toggle = match state.activation_toggle {
                                ActivationToggle::Auto => ActivationToggle::On,
                                ActivationToggle::On => ActivationToggle::Off,
                                ActivationToggle::Off => ActivationToggle::Auto,
                            };
                            state.duel_log.push(format!("Activation Toggle set to {:?}", state.activation_toggle));
                        }
                    });
                });

                ui.add_space(4.0);

                // Player Hand Cards with lift on hover
                ScrollArea::horizontal().id_salt("duel_hand_cards_scroll").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let hand_w = 72.0;
                        let hand_h = 100.0;
                        for &code in &state.my_hand {
                            if let Some(card) = db.get(code) {
                                let is_selected = state.selected_hand_code == Some(code);
                                let resp = draw_card_item(ui, card, hand_w, hand_h, is_selected);
                                if resp.clicked() {
                                    state.selected_hand_code = if is_selected { None } else { Some(code) };
                                }
                            }
                        }
                    });
                });
            });
        });

        ui.separator();

        // --- DUEL LOG & CARD INSPECTOR (RIGHT) ---
        ui.allocate_ui(Vec2::new(log_w, screen_h - 75.0), |ui| {
            ui.vertical(|ui| {
                ui.heading(RichText::new("Combat & Action Log").size(14.0).color(TEXT_MUTED));
                ScrollArea::vertical().id_salt("duel_combat_log_scroll").max_height(screen_h - 130.0).show(ui, |ui| {
                    for entry in state.duel_log.iter().rev() {
                        ui.label(RichText::new(entry).size(11.0).color(TEXT_PRIMARY));
                    }
                });
            });
        });
    });

    // --- PILE INSPECTOR MODAL (Graveyard / Banished) ---
    let mut close_pile = false;
    if let Some((title, cards)) = &state.view_pile {
        egui::Window::new(RichText::new(title).strong().color(ACCENT_GOLD))
            .fixed_size(Vec2::new(500.0, 380.0))
            .show(ui.ctx(), |ui| {
                ui.label(format!("Total entries: {}", cards.len()));
                ScrollArea::vertical().id_salt("pile_modal_scroll").show(ui, |ui| {
                    egui::Grid::new("pile_cards_grid").spacing(Vec2::new(6.0, 6.0)).show(ui, |ui| {
                        for (i, &code) in cards.iter().enumerate() {
                            if let Some(card) = db.get(code) {
                                draw_card_item(ui, card, 64.0, 90.0, false);
                            }
                            if (i + 1) % 6 == 0 {
                                ui.end_row();
                            }
                        }
                    });
                });
                if ui.button("Close").clicked() {
                    close_pile = true;
                }
            });
    }
    if close_pile {
        state.view_pile = None;
    }

    // --- MASTER DUEL INTERACTIVE ACTION MODAL ---
    if let Some(prompt) = state.active_prompt.clone() {
        egui::Window::new(RichText::new("⚡ ACTION REQUIRED").color(ACCENT_GOLD).size(15.0).strong())
            .fixed_size(Vec2::new(450.0, 280.0))
            .show(ui.ctx(), |ui| {
                match &prompt {
                    CoreMessage::SelectIdleCmd {
                        summonable,
                        spsummonable,
                        activatable,
                        ssetable,
                        can_bp,
                        can_ep,
                        ..
                    } => {
                        ui.heading(RichText::new("Main Phase: Choose Action").color(ACCENT_CYAN).size(14.0));
                        ui.label(RichText::new("Select an action or transition phase:").color(TEXT_MUTED));
                        ui.add_space(4.0);

                        ScrollArea::vertical().id_salt("idle_cmd_actions_scroll").max_height(160.0).show(ui, |ui| {
                            // Normal Summon
                            for (i, s) in summonable.iter().enumerate() {
                                let name = db.get(s.code).map(|c| c.name.as_str()).unwrap_or("Monster");
                                if ui.button(RichText::new(format!("⚔️ Normal Summon: {name}")).color(ACCENT_GOLD)).clicked() {
                                    send_response(state, PlayerResponse::IdleCmd(IdleCmdChoice::Summon(i)));
                                }
                            }
                            // Special Summon
                            for (i, s) in spsummonable.iter().enumerate() {
                                let name = db.get(s.code).map(|c| c.name.as_str()).unwrap_or("Monster");
                                if ui.button(RichText::new(format!("✨ Special Summon: {name}")).color(ACCENT_PURPLE)).clicked() {
                                    send_response(state, PlayerResponse::IdleCmd(IdleCmdChoice::SpSummon(i)));
                                }
                            }
                            // Activatable Spell/Effect
                            for (i, a) in activatable.iter().enumerate() {
                                let name = db.get(a.code).map(|c| c.name.as_str()).unwrap_or("Card");
                                if ui.button(RichText::new(format!("⚡ Activate: {name}")).color(ACCENT_CYAN)).clicked() {
                                    send_response(state, PlayerResponse::IdleCmd(IdleCmdChoice::Activate(i)));
                                }
                            }
                            // Set Spell/Trap
                            for (i, s) in ssetable.iter().enumerate() {
                                let name = db.get(s.code).map(|c| c.name.as_str()).unwrap_or("Card");
                                if ui.button(RichText::new(format!("📜 Set: {name}")).color(TEXT_PRIMARY)).clicked() {
                                    send_response(state, PlayerResponse::IdleCmd(IdleCmdChoice::SetSpell(i)));
                                }
                            }
                        });

                        ui.separator();
                        ui.horizontal(|ui| {
                            if *can_bp && ui.button(RichText::new("⚔️ To Battle Phase ▶").color(ACCENT_RED).strong()).clicked() {
                                send_response(state, PlayerResponse::IdleCmd(IdleCmdChoice::ToBattlePhase));
                            }
                            if *can_ep && ui.button(RichText::new("⏭ To End Phase").color(TEXT_MUTED)).clicked() {
                                send_response(state, PlayerResponse::IdleCmd(IdleCmdChoice::ToEndPhase));
                            }
                        });
                    }

                    CoreMessage::SelectBattleCmd {
                        attackable,
                        activatable,
                        can_ep,
                        ..
                    } => {
                        ui.heading(RichText::new("Battle Phase: Declare Attack").color(ACCENT_RED).size(14.0));
                        for (i, a) in attackable.iter().enumerate() {
                            let name = db.get(a.code).map(|c| c.name.as_str()).unwrap_or("Monster");
                            if ui.button(RichText::new(format!("⚔️ Attack with: {name}")).color(ACCENT_RED).strong()).clicked() {
                                send_response(state, PlayerResponse::BattleCmd(BattleCmdChoice::Attack(i)));
                            }
                        }
                        for (i, a) in activatable.iter().enumerate() {
                            let name = db.get(a.code).map(|c| c.name.as_str()).unwrap_or("Card");
                            if ui.button(format!("⚡ Activate: {name}")).clicked() {
                                send_response(state, PlayerResponse::BattleCmd(BattleCmdChoice::Activate(i)));
                            }
                        }
                        ui.separator();
                        ui.horizontal(|ui| {
                            if *can_ep && ui.button("⏭ To End Phase").clicked() {
                                send_response(state, PlayerResponse::BattleCmd(BattleCmdChoice::ToEndPhase));
                            } else if ui.button("▶ To Main Phase 2").clicked() {
                                send_response(state, PlayerResponse::BattleCmd(BattleCmdChoice::ToMainPhase2));
                            }
                        });
                    }

                    CoreMessage::SelectChain { chains, forced, .. } => {
                        ui.heading(RichText::new("⚡ Chain Window Open").color(ACCENT_GOLD).size(14.0));
                        if chains.is_empty() {
                            ui.label("No chainable quick effects.");
                            if ui.button("Pass Priority").clicked() {
                                send_response(state, PlayerResponse::Chain(None));
                            }
                        } else {
                            ui.label("Respond to activation:");
                            ScrollArea::vertical().id_salt("chain_responses_scroll").max_height(140.0).show(ui, |ui| {
                                for (i, c) in chains.iter().enumerate() {
                                    let name = db.get(c.code).map(|cd| cd.name.as_str()).unwrap_or("Card");
                                    if ui.button(RichText::new(format!("⚡ Chain: {name}")).color(ACCENT_GOLD)).clicked() {
                                        send_response(state, PlayerResponse::Chain(Some(i)));
                                    }
                                }
                            });
                            if !*forced && ui.button("Pass Priority").clicked() {
                                send_response(state, PlayerResponse::Chain(None));
                            }
                        }
                    }

                    CoreMessage::SelectEffectYesNo { code, .. } => {
                        let name = db.get(*code).map(|c| c.name.as_str()).unwrap_or("Card");
                        ui.label(RichText::new(format!("Activate the effect of '{name}'?")).strong());
                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Yes").color(ACCENT_GREEN).strong()).clicked() {
                                send_response(state, PlayerResponse::EffectYesNo(true));
                            }
                            if ui.button(RichText::new("No").color(ACCENT_RED)).clicked() {
                                send_response(state, PlayerResponse::EffectYesNo(false));
                            }
                        });
                    }

                    CoreMessage::SelectYesNo { .. } => {
                        ui.label("Confirm action?");
                        ui.horizontal(|ui| {
                            if ui.button("Yes").clicked() {
                                send_response(state, PlayerResponse::YesNo(true));
                            }
                            if ui.button("No").clicked() {
                                send_response(state, PlayerResponse::YesNo(false));
                            }
                        });
                    }

                    CoreMessage::SelectCard { cards, min, max, cancelable, .. } => {
                        ui.label(format!("Select {min} to {max} card(s):"));
                        ScrollArea::vertical().id_salt("select_card_targets_scroll").max_height(140.0).show(ui, |ui| {
                            for (i, c) in cards.iter().enumerate() {
                                let name = db.get(c.code).map(|cd| cd.name.as_str()).unwrap_or("Card");
                                if ui.button(format!("{i}: {name}")).clicked() {
                                    send_response(state, PlayerResponse::CardIndices(vec![i]));
                                }
                            }
                        });
                        if *cancelable && ui.button("Cancel").clicked() {
                            send_response(state, PlayerResponse::Cancel);
                        }
                    }

                    CoreMessage::SelectPosition { positions, .. } => {
                        ui.label("Select Battle Position:");
                        let pos = *positions as u32;
                        ui.horizontal(|ui| {
                            if (pos & POS_FACEUP_ATTACK) != 0 && ui.button("⚔️ Attack Position").clicked() {
                                send_response(state, PlayerResponse::Position(POS_FACEUP_ATTACK));
                            }
                            if (pos & POS_FACEUP_DEFENSE) != 0 && ui.button("🛡️ Defense Position").clicked() {
                                send_response(state, PlayerResponse::Position(POS_FACEUP_DEFENSE));
                            }
                        });
                    }

                    CoreMessage::SelectPlace { count, flag, .. } => {
                        ui.label(format!("Select {count} field zone(s) to place card:"));
                        let mut chosen = Vec::new();
                        // Find first available zone
                        for seq in 0..5 {
                            let to_check = (1u32 << seq) << 8;
                            if (flag & to_check) == 0 {
                                chosen.push((0, LOCATION_SZONE as u8, seq as u8));
                                break;
                            }
                        }
                        if chosen.is_empty() {
                            for seq in 0..5 {
                                let to_check = 1u32 << seq;
                                if (flag & to_check) == 0 {
                                    chosen.push((0, LOCATION_MZONE as u8, seq as u8));
                                    break;
                                }
                            }
                        }
                        if ui.button("Place Card").clicked() {
                            send_response(state, PlayerResponse::SelectPlace(chosen));
                        }
                    }

                    _ => {
                        ui.label("Awaiting engine decision...");
                        if ui.button("Continue").clicked() {
                            send_response(state, PlayerResponse::Cancel);
                        }
                    }
                }
            });
    }
}

fn send_response(state: &mut DuelUiState, resp: PlayerResponse) {
    if let Some(ref mut session) = state.session {
        let _ = session.submit_human_response(0, &resp);
        state.active_prompt = None;
    }
}

fn render_zone_slot(ui: &mut Ui, card_code: Option<u32>, width: f32, height: f32, db: &CardDatabase) {
    if let Some(code) = card_code {
        if let Some(card) = db.get(code) {
            draw_card_item(ui, card, width, height, false);
            return;
        }
    }

    // Empty Zone Slot with metallic grid bevel
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
    ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(18, 23, 34));
    ui.painter().rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, Color32::from_rgb(38, 48, 70)),
        bevy_egui::egui::StrokeKind::Inside,
    );
}
