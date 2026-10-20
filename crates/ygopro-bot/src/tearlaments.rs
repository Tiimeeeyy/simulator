//! Dedicated AI policy for Ishizu Tearlaments.
//!
//! Implements fusion summoning prioritization, milling heuristics,
//! and graveyard shuffler management.

use ocgcore_sys::*;
use ygopro_core::{
    BattleCmdChoice, CoreMessage, IdleCmdChoice, PlayerResponse, SelectCardItem, SelectChainItem,
};
use ygopro_data::CardDatabase;

use crate::agent::DuelAgent;
use crate::state::BotBoardState;

// Key Passcodes
pub const CARD_REINOHEART: u32 = 73956664;
pub const CARD_SCHEIREN: u32 = 572850;
pub const CARD_HAVNIS: u32 = 37961969;
pub const CARD_MERRLI: u32 = 74078255;
pub const CARD_TEAR_KASHTIRA: u32 = 4928565;
pub const CARD_KELBEK: u32 = 25926710;
pub const CARD_AGIDO: u32 = 62320425;
pub const CARD_KELDO: u32 = 63542003;
pub const CARD_MUDORA: u32 = 99937011;
pub const CARD_FENRIR: u32 = 32909498;
pub const CARD_DIVINER: u32 = 92919429;
pub const CARD_PERLEREINO: u32 = 77103950;
pub const CARD_SCREAM: u32 = 6767771;
pub const CARD_GRIEF: u32 = 33878367;
pub const CARD_FOOLISH: u32 = 81439173;
pub const CARD_FOOLISH_GOODS: u32 = 35726888;
pub const CARD_SULLIEK: u32 = 74920585;
pub const CARD_CRYME: u32 = 1329620;
pub const CARD_METANOISE: u32 = 38436986;
pub const CARD_TRIVIKARMA: u32 = 7436169;
pub const CARD_SUPER_POLY: u32 = 48130397;
pub const CARD_CALLED_BY: u32 = 24224830;

// Extra deck
pub const CARD_RULKALLOS: u32 = 84330567;
pub const CARD_KALEIDO_HEART: u32 = 28226490;
pub const CARD_DRAGOSTAPELIA: u32 = 69946549;
pub const CARD_WINDA: u32 = 94977269;
pub const CARD_MUDRAGON: u32 = 54757758;
pub const CARD_GARURA: u32 = 11765832;
pub const CARD_BARONNE: u32 = 84815190;
pub const CARD_REDOER: u32 = 55285840;

/// Specialized AI for post-PHHY Ishizu Tearlaments.
pub struct TearlamentsPolicy {
    player_id: u8,
    state: BotBoardState,
}

impl TearlamentsPolicy {
    pub fn new(player_id: u8) -> Self {
        Self {
            player_id,
            state: BotBoardState::new(player_id),
        }
    }

    /// Evaluates which card in hand is best to discard for Scheiren or cost.
    fn best_discard_card(&self, candidates: &[SelectCardItem]) -> usize {
        let score = |code: u32| -> i32 {
            match code {
                CARD_AGIDO | CARD_KELBEK => 100,
                CARD_MERRLI | CARD_HAVNIS | CARD_SCHEIREN => 90,
                CARD_TRIVIKARMA => 85,
                CARD_SULLIEK | CARD_CRYME | CARD_METANOISE => 80,
                CARD_KELDO | CARD_MUDORA => 75,
                CARD_TEAR_KASHTIRA => 70,
                _ => 10,
            }
        };

        candidates
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| score(c.code))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Evaluates which card to add from deck to hand (Perlereino search, Sulliek search).
    fn best_search_target(&self, candidates: &[SelectCardItem]) -> usize {
        let score = |code: u32| -> i32 {
            match code {
                CARD_PERLEREINO => 110,
                CARD_REINOHEART if !self.state.normal_summon_used => 100,
                CARD_SCHEIREN if self.state.is_hopt_available(CARD_SCHEIREN) => 90,
                CARD_HAVNIS => 85,
                CARD_TEAR_KASHTIRA => 80,
                CARD_FENRIR => 75,
                CARD_SULLIEK => 70,
                _ => 10,
            }
        };

        candidates
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| score(c.code))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
}

impl DuelAgent for TearlamentsPolicy {
    fn name(&self) -> &str {
        "Ishizu-Tearlaments-AI"
    }

    fn player_id(&self) -> u8 {
        self.player_id
    }

    fn reset(&mut self) {
        self.state.reset();
    }

    fn on_message(&mut self, msg: &CoreMessage) {
        self.state.update(msg);
    }

    fn respond(&mut self, prompt: &CoreMessage, _db: &CardDatabase) -> PlayerResponse {
        match prompt {
            CoreMessage::SelectIdleCmd {
                summonable,
                spsummonable,
                msetable: _,
                ssetable,
                activatable,
                can_bp,
                can_ep,
                ..
            } => {
                // 1. Spells priority (Perlereino -> Foolish -> Goods -> Scream -> Grief)
                for (i, act) in activatable.iter().enumerate() {
                    if act.code == CARD_PERLEREINO && self.state.is_hopt_available(CARD_PERLEREINO) {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::Activate(i));
                    }
                    if act.code == CARD_FOOLISH {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::Activate(i));
                    }
                    if act.code == CARD_FOOLISH_GOODS {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::Activate(i));
                    }
                    if act.code == CARD_SCREAM {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::Activate(i));
                    }
                    if act.code == CARD_GRIEF {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::Activate(i));
                    }
                }

                // 2. Special Summons (Fenrir -> Tear Kashtira -> Scheiren)
                for (i, sp) in spsummonable.iter().enumerate() {
                    if sp.code == CARD_FENRIR {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::SpSummon(i));
                    }
                    if sp.code == CARD_TEAR_KASHTIRA {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::SpSummon(i));
                    }
                }
                for (i, act) in activatable.iter().enumerate() {
                    if act.code == CARD_SCHEIREN && self.state.is_hopt_available(CARD_SCHEIREN) {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::Activate(i));
                    }
                }

                // 3. Normal Summons (Reinoheart -> Diviner -> Merrli -> Fenrir)
                if !self.state.normal_summon_used && !summonable.is_empty() {
                    if let Some(pos) = summonable.iter().position(|s| s.code == CARD_REINOHEART) {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::Summon(pos));
                    }
                    if let Some(pos) = summonable.iter().position(|s| s.code == CARD_DIVINER) {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::Summon(pos));
                    }
                    if let Some(pos) = summonable.iter().position(|s| s.code == CARD_MERRLI) {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::Summon(pos));
                    }
                    return PlayerResponse::IdleCmd(IdleCmdChoice::Summon(0));
                }

                // 4. Other activatable effects
                if !activatable.is_empty() {
                    return PlayerResponse::IdleCmd(IdleCmdChoice::Activate(0));
                }

                // 5. Set Traps / Spells
                for (i, ss) in ssetable.iter().enumerate() {
                    if matches!(
                        ss.code,
                        CARD_SULLIEK | CARD_CRYME | CARD_METANOISE | CARD_SUPER_POLY | CARD_CALLED_BY
                    ) {
                        return PlayerResponse::IdleCmd(IdleCmdChoice::SetSpell(i));
                    }
                }

                // 6. Transition Phase
                if *can_bp {
                    PlayerResponse::IdleCmd(IdleCmdChoice::ToBattlePhase)
                } else if *can_ep {
                    PlayerResponse::IdleCmd(IdleCmdChoice::ToEndPhase)
                } else {
                    PlayerResponse::IdleCmd(IdleCmdChoice::ToEndPhase)
                }
            }

            CoreMessage::SelectBattleCmd {
                attackable,
                can_ep,
                activatable,
                ..
            } => {
                if !activatable.is_empty() {
                    return PlayerResponse::BattleCmd(BattleCmdChoice::Activate(0));
                }
                if !attackable.is_empty() {
                    return PlayerResponse::BattleCmd(BattleCmdChoice::Attack(0));
                }
                if *can_ep {
                    PlayerResponse::BattleCmd(BattleCmdChoice::ToEndPhase)
                } else {
                    PlayerResponse::BattleCmd(BattleCmdChoice::ToMainPhase2)
                }
            }

            CoreMessage::SelectChain {
                chains,
                forced,
                ..
            } => {
                if *forced || !chains.is_empty() {
                    let score_chain = |item: &SelectChainItem| -> i32 {
                        match item.code {
                            CARD_MERRLI | CARD_HAVNIS | CARD_SCHEIREN => 100,
                            CARD_AGIDO | CARD_KELBEK => 90,
                            CARD_REINOHEART | CARD_PERLEREINO | CARD_SULLIEK => 85,
                            CARD_RULKALLOS | CARD_BARONNE | CARD_DRAGOSTAPELIA | CARD_CRYME => 80,
                            CARD_KELDO | CARD_MUDORA => 70,
                            _ => 50,
                        }
                    };

                    let best_idx = chains
                        .iter()
                        .enumerate()
                        .max_by_key(|(_, c)| score_chain(c))
                        .map(|(i, _)| i)
                        .unwrap_or(0);

                    PlayerResponse::Chain(Some(best_idx))
                } else {
                    PlayerResponse::Chain(None)
                }
            }

            CoreMessage::SelectCard {
                min,
                cards,
                cancelable,
                ..
            } => {
                let count = (*min as usize).max(1).min(cards.len());
                if cards.is_empty() {
                    return if *cancelable {
                        PlayerResponse::Cancel
                    } else {
                        PlayerResponse::CardIndices(vec![])
                    };
                }

                if cards.iter().all(|c| c.location.location == LOCATION_HAND as u8) {
                    let best = self.best_discard_card(cards);
                    let mut chosen = vec![best];
                    for i in 0..cards.len() {
                        if chosen.len() >= count {
                            break;
                        }
                        if !chosen.contains(&i) {
                            chosen.push(i);
                        }
                    }
                    return PlayerResponse::CardIndices(chosen);
                }

                if cards.iter().all(|c| c.location.location == LOCATION_DECK as u8) {
                    let best = self.best_search_target(cards);
                    let mut chosen = vec![best];
                    for i in 0..cards.len() {
                        if chosen.len() >= count {
                            break;
                        }
                        if !chosen.contains(&i) {
                            chosen.push(i);
                        }
                    }
                    return PlayerResponse::CardIndices(chosen);
                }

                let indices = (0..count).collect();
                PlayerResponse::CardIndices(indices)
            }

            CoreMessage::SelectUnselectCard {
                finishable,
                selectable,
                cancelable,
                ..
            } => {
                if *finishable {
                    PlayerResponse::SelectUnselect(None)
                } else if !selectable.is_empty() {
                    PlayerResponse::SelectUnselect(Some(0))
                } else if *cancelable {
                    PlayerResponse::Cancel
                } else {
                    PlayerResponse::SelectUnselect(None)
                }
            }

            CoreMessage::SelectEffectYesNo { .. } => PlayerResponse::EffectYesNo(true),
            CoreMessage::SelectYesNo { .. } => PlayerResponse::YesNo(true),

            CoreMessage::SelectPosition { positions, .. } => {
                let pos = *positions as u32;
                if (pos & POS_FACEUP_ATTACK) != 0 {
                    PlayerResponse::Position(POS_FACEUP_ATTACK)
                } else if (pos & POS_FACEUP_DEFENSE) != 0 {
                    PlayerResponse::Position(POS_FACEUP_DEFENSE)
                } else {
                    PlayerResponse::Position(pos)
                }
            }

            CoreMessage::SelectOption { .. } => PlayerResponse::Option(0),

            CoreMessage::AnnounceAttrib { available, .. } => {
                let dark = 0x20;
                let water = 0x02;
                if (*available & dark) != 0 {
                    PlayerResponse::AnnounceAttribute(dark)
                } else if (*available & water) != 0 {
                    PlayerResponse::AnnounceAttribute(water)
                } else {
                    PlayerResponse::AnnounceAttribute(*available)
                }
            }

            CoreMessage::AnnounceRace { available, .. } => {
                let aqua = 0x40;
                if (*available & aqua) != 0 {
                    PlayerResponse::AnnounceRace(aqua)
                } else {
                    PlayerResponse::AnnounceRace(*available)
                }
            }

            CoreMessage::AnnounceCard { .. } => {
                PlayerResponse::AnnounceCard(CARD_REINOHEART)
            }

            CoreMessage::AnnounceNumber { options, .. } => {
                PlayerResponse::AnnounceNumber(0.min(options.len().saturating_sub(1)))
            }

            CoreMessage::SelectPlace { player, count, flag } => {
                let mut chosen = Vec::new();
                // Check SZONE (zones 0..5)
                for seq in 0..5 {
                    if chosen.len() >= *count as usize {
                        break;
                    }
                    let to_check = (1u32 << seq) << 8;
                    if (flag & to_check) == 0 {
                        chosen.push((*player, LOCATION_SZONE as u8, seq as u8));
                    }
                }
                // Check MZONE (zones 0..5)
                for seq in 0..5 {
                    if chosen.len() >= *count as usize {
                        break;
                    }
                    let to_check = 1u32 << seq;
                    if (flag & to_check) == 0 {
                        chosen.push((*player, LOCATION_MZONE as u8, seq as u8));
                    }
                }
                PlayerResponse::SelectPlace(chosen)
            }

            CoreMessage::SelectDisfield { player, count, flag } => {
                let mut chosen = Vec::new();
                for seq in 0..5 {
                    if chosen.len() >= *count as usize {
                        break;
                    }
                    let to_check = 1u32 << seq;
                    if (flag & to_check) == 0 {
                        chosen.push((*player, LOCATION_MZONE as u8, seq as u8));
                    }
                }
                PlayerResponse::SelectPlace(chosen)
            }

            _ => PlayerResponse::Cancel,
        }
    }
}
