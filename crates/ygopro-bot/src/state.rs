//! Board state tracker and HOPT flag recorder.

use std::collections::HashSet;
use ocgcore_sys::*;
use ygopro_core::CoreMessage;

/// Tracks local game state and once-per-turn effect usage for AI heuristics.
#[derive(Debug, Clone)]
pub struct BotBoardState {
    pub player_id: u8,
    pub turn_count: u32,
    pub current_phase: u16,
    pub my_lp: u32,
    pub opp_lp: u32,
    pub normal_summon_used: bool,
    /// Card codes that have activated once-per-turn effects this turn.
    pub used_hopt: HashSet<u32>,
    /// Hand cards for our player.
    pub my_hand: Vec<u32>,
    /// Graveyard cards for our player.
    pub my_grave: Vec<u32>,
    /// Graveyard cards for opponent.
    pub opp_grave: Vec<u32>,
    /// Monster zones for our player (seq -> code).
    pub my_mzone: Vec<Option<u32>>,
    /// Monster zones for opponent (seq -> code).
    pub opp_mzone: Vec<Option<u32>>,
    /// Spell/trap zones for our player (seq -> code).
    pub my_szone: Vec<Option<u32>>,
    /// Spell/trap zones for opponent (seq -> code).
    pub opp_szone: Vec<Option<u32>>,
}

impl BotBoardState {
    pub fn new(player_id: u8) -> Self {
        Self {
            player_id,
            turn_count: 0,
            current_phase: 0,
            my_lp: 8000,
            opp_lp: 8000,
            normal_summon_used: false,
            used_hopt: HashSet::new(),
            my_hand: Vec::new(),
            my_grave: Vec::new(),
            opp_grave: Vec::new(),
            my_mzone: vec![None; 7],
            opp_mzone: vec![None; 7],
            my_szone: vec![None; 8],
            opp_szone: vec![None; 8],
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.player_id);
    }

    pub fn is_hopt_available(&self, code: u32) -> bool {
        !self.used_hopt.contains(&code)
    }

    pub fn mark_hopt_used(&mut self, code: u32) {
        self.used_hopt.insert(code);
    }

    pub fn update(&mut self, msg: &CoreMessage) {
        match msg {
            CoreMessage::Start { lp0, lp1, .. } => {
                if self.player_id == 0 {
                    self.my_lp = *lp0;
                    self.opp_lp = *lp1;
                } else {
                    self.my_lp = *lp1;
                    self.opp_lp = *lp0;
                }
            }
            CoreMessage::NewTurn { player: _ } => {
                self.turn_count += 1;
                self.used_hopt.clear();
                self.normal_summon_used = false;
            }
            CoreMessage::NewPhase { phase } => {
                self.current_phase = *phase;
            }
            CoreMessage::Draw { player, cards, .. } => {
                if *player == self.player_id {
                    for &c in cards {
                        if c != 0 {
                            self.my_hand.push(c);
                        }
                    }
                }
            }
            CoreMessage::Summoned | CoreMessage::FlipSummoned => {
                self.normal_summon_used = true;
            }
            CoreMessage::Chaining { code, con, .. } => {
                if *con == self.player_id {
                    self.used_hopt.insert(*code);
                }
            }
            CoreMessage::Damage { player, amount } => {
                if *player == self.player_id {
                    self.my_lp = self.my_lp.saturating_sub(*amount);
                } else {
                    self.opp_lp = self.opp_lp.saturating_sub(*amount);
                }
            }
            CoreMessage::Recover { player, amount } => {
                if *player == self.player_id {
                    self.my_lp = self.my_lp.saturating_add(*amount);
                } else {
                    self.opp_lp = self.opp_lp.saturating_add(*amount);
                }
            }
            CoreMessage::LpUpdate { player, lp } => {
                if *player == self.player_id {
                    self.my_lp = *lp;
                } else {
                    self.opp_lp = *lp;
                }
            }
            CoreMessage::Move {
                code,
                from_con,
                from_loc,
                from_seq: _,
                to_con,
                to_loc,
                to_seq: _,
                ..
            } => {
                let code = *code;
                let from_con = *from_con;
                let from_loc = *from_loc as u32;
                let to_con = *to_con;
                let to_loc = *to_loc as u32;

                // Remove from previous location if tracked
                if from_con == self.player_id {
                    if from_loc == LOCATION_HAND {
                        if let Some(pos) = self.my_hand.iter().position(|&c| c == code) {
                            self.my_hand.remove(pos);
                        }
                    } else if from_loc == LOCATION_GRAVE {
                        if let Some(pos) = self.my_grave.iter().position(|&c| c == code) {
                            self.my_grave.remove(pos);
                        }
                    }
                } else if from_con == (1 - self.player_id) {
                    if from_loc == LOCATION_GRAVE {
                        if let Some(pos) = self.opp_grave.iter().position(|&c| c == code) {
                            self.opp_grave.remove(pos);
                        }
                    }
                }

                // Add to new location
                if to_con == self.player_id {
                    if to_loc == LOCATION_HAND && code != 0 {
                        self.my_hand.push(code);
                    } else if to_loc == LOCATION_GRAVE && code != 0 {
                        self.my_grave.push(code);
                    }
                } else if to_con == (1 - self.player_id) {
                    if to_loc == LOCATION_GRAVE && code != 0 {
                        self.opp_grave.push(code);
                    }
                }
            }
            _ => {}
        }
    }
}
