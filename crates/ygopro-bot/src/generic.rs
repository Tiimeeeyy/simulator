//! General-purpose fallback bot for standard decks.

use ocgcore_sys::*;
use ygopro_core::{
    BattleCmdChoice, CoreMessage, IdleCmdChoice, PlayerResponse,
};
use ygopro_data::CardDatabase;

use crate::agent::DuelAgent;
use crate::state::BotBoardState;

/// General-purpose fallback bot that plays basic beatdown / standard cards.
pub struct GenericAgent {
    player_id: u8,
    state: BotBoardState,
}

impl GenericAgent {
    pub fn new(player_id: u8) -> Self {
        Self {
            player_id,
            state: BotBoardState::new(player_id),
        }
    }
}

impl DuelAgent for GenericAgent {
    fn name(&self) -> &str {
        "Generic-AI"
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
                ssetable,
                activatable,
                can_bp,
                can_ep,
                ..
            } => {
                if !spsummonable.is_empty() {
                    return PlayerResponse::IdleCmd(IdleCmdChoice::SpSummon(0));
                }
                if !summonable.is_empty() && !self.state.normal_summon_used {
                    return PlayerResponse::IdleCmd(IdleCmdChoice::Summon(0));
                }
                if !activatable.is_empty() {
                    return PlayerResponse::IdleCmd(IdleCmdChoice::Activate(0));
                }
                if !ssetable.is_empty() {
                    return PlayerResponse::IdleCmd(IdleCmdChoice::SetSpell(0));
                }
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

            CoreMessage::SelectChain { chains, forced, .. } => {
                if *forced {
                    PlayerResponse::Chain(Some(0))
                } else if !chains.is_empty() {
                    PlayerResponse::Chain(Some(0))
                } else {
                    PlayerResponse::Chain(None)
                }
            }

            CoreMessage::SelectCard { min, cards, cancelable, .. } => {
                let count = (*min as usize).max(1).min(cards.len());
                if cards.is_empty() {
                    if *cancelable {
                        PlayerResponse::Cancel
                    } else {
                        PlayerResponse::CardIndices(vec![])
                    }
                } else {
                    let indices = (0..count).collect();
                    PlayerResponse::CardIndices(indices)
                }
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
                } else {
                    PlayerResponse::Position(pos)
                }
            }
            CoreMessage::SelectOption { .. } => PlayerResponse::Option(0),
            CoreMessage::AnnounceAttrib { available, .. } => {
                PlayerResponse::AnnounceAttribute(*available)
            }
            CoreMessage::AnnounceRace { available, .. } => PlayerResponse::AnnounceRace(*available),
            CoreMessage::AnnounceCard { .. } => PlayerResponse::AnnounceCard(73956664),
            CoreMessage::AnnounceNumber { options, .. } => {
                PlayerResponse::AnnounceNumber(0.min(options.len().saturating_sub(1)))
            }
            CoreMessage::SelectPlace { player, count, flag } => {
                let mut chosen = Vec::new();
                for seq in 0..5 {
                    if chosen.len() >= *count as usize {
                        break;
                    }
                    let to_check = (1u32 << seq) << 8;
                    if (flag & to_check) == 0 {
                        chosen.push((*player, LOCATION_SZONE as u8, seq as u8));
                    }
                }
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
