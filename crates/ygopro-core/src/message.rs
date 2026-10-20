//! Strongly typed engine messages parsed from ocgcore's binary message stream.

use ocgcore_sys::*;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardLocation {
    pub controller: u8,
    pub location: u8,
    pub sequence: u32,
    pub position: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CmdCardItem {
    pub code: u32,
    pub controller: u8,
    pub location: u8,
    pub sequence: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CmdActivatableItem {
    pub code: u32,
    pub controller: u8,
    pub location: u8,
    pub sequence: u32,
    pub desc: u64,
    pub client_mode: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CmdAttackableItem {
    pub code: u32,
    pub controller: u8,
    pub location: u8,
    pub sequence: u8,
    pub direct_attackable: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectCardItem {
    pub code: u32,
    pub location: CardLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectChainItem {
    pub code: u32,
    pub location: CardLocation,
    pub desc: u64,
    pub client_mode: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreMessage {
    Retry,
    Hint { kind: u8, player: u8, value: u64 },
    Waiting,
    Start {
        playertype: u8,
        lp0: u32,
        lp1: u32,
        deck0: u16,
        deck1: u16,
        extra0: u16,
        extra1: u16,
    },
    Win { player: u8, reason: u8 },
    SelectBattleCmd {
        player: u8,
        activatable: Vec<CmdActivatableItem>,
        attackable: Vec<CmdAttackableItem>,
        can_m2: bool,
        can_ep: bool,
    },
    SelectIdleCmd {
        player: u8,
        summonable: Vec<CmdCardItem>,
        spsummonable: Vec<CmdCardItem>,
        reposable: Vec<CmdCardItem>,
        msetable: Vec<CmdCardItem>,
        ssetable: Vec<CmdCardItem>,
        activatable: Vec<CmdActivatableItem>,
        can_bp: bool,
        can_ep: bool,
        can_shuffle: bool,
    },
    SelectEffectYesNo {
        player: u8,
        code: u32,
        location: CardLocation,
        desc: u64,
    },
    SelectYesNo { player: u8, desc: u64 },
    SelectOption { player: u8, options: Vec<u64> },
    SelectCard {
        player: u8,
        cancelable: bool,
        min: usize,
        max: usize,
        cards: Vec<SelectCardItem>,
    },
    SelectChain {
        player: u8,
        spe_count: u8,
        forced: bool,
        hint0: u32,
        hint1: u32,
        chains: Vec<SelectChainItem>,
    },
    SelectPlace { player: u8, count: u8, flag: u32 },
    SelectDisfield { player: u8, count: u8, flag: u32 },
    SelectPosition { player: u8, code: u32, positions: u8 },
    SelectTribute {
        player: u8,
        cancelable: bool,
        min: usize,
        max: usize,
        cards: Vec<SelectCardItem>,
    },
    SelectUnselectCard {
        player: u8,
        finishable: bool,
        cancelable: bool,
        min: usize,
        max: usize,
        selectable: Vec<SelectCardItem>,
        unselectable: Vec<SelectCardItem>,
    },
    ConfirmDecktop { player: u8, cards: Vec<u32> },
    ConfirmCards { player: u8, cards: Vec<SelectCardItem> },
    ShuffleDeck { player: u8 },
    ShuffleHand { player: u8, cards: Vec<u32> },
    NewTurn { player: u8 },
    NewPhase { phase: u16 },
    Move {
        code: u32,
        from_con: u8,
        from_loc: u8,
        from_seq: u8,
        from_pos: u8,
        to_con: u8,
        to_loc: u8,
        to_seq: u8,
        to_pos: u8,
        reason: u32,
    },
    PosChange {
        code: u32,
        con: u8,
        loc: u8,
        seq: u8,
        prev_pos: u8,
        current_pos: u8,
    },
    Set {
        code: u32,
        con: u8,
        loc: u8,
        seq: u8,
        pos: u8,
    },
    Swap {
        code1: u32,
        con1: u8,
        loc1: u8,
        seq1: u8,
        pos1: u8,
        code2: u32,
        con2: u8,
        loc2: u8,
        seq2: u8,
        pos2: u8,
    },
    Summoning {
        code: u32,
        con: u8,
        loc: u8,
        seq: u8,
        pos: u8,
    },
    Summoned,
    SpSummoning {
        code: u32,
        con: u8,
        loc: u8,
        seq: u8,
        pos: u8,
    },
    SpSummoned,
    FlipSummoning {
        code: u32,
        con: u8,
        loc: u8,
        seq: u8,
        pos: u8,
    },
    FlipSummoned,
    Chaining {
        code: u32,
        con: u8,
        loc: u8,
        seq: u8,
        tc: u32,
        desc: u64,
        chain_link: u8,
    },
    Chained { chain_link: u8 },
    ChainSolving { chain_link: u8 },
    ChainSolved { chain_link: u8 },
    ChainEnd,
    ChainNegated { chain_link: u8 },
    ChainDisabled { chain_link: u8 },
    CardSelected { player: u8, cards: Vec<u32> },
    Draw { player: u8, count: u8, cards: Vec<u32> },
    Damage { player: u8, amount: u32 },
    Recover { player: u8, amount: u32 },
    LpUpdate { player: u8, lp: u32 },
    PayLpCost { player: u8, amount: u32 },
    Attack {
        attacker_code: u32,
        attacker_con: u8,
        attacker_loc: u8,
        attacker_seq: u8,
        attacker_pos: u8,
        target_con: u8,
        target_loc: u8,
        target_seq: u8,
        target_pos: u8,
    },
    AttackDisabled,
    DamageStepStart,
    DamageStepEnd,
    TossCoin { player: u8, results: Vec<u8> },
    TossDice { player: u8, results: Vec<u8> },
    AnnounceRace { player: u8, count: u8, available: u64 },
    AnnounceAttrib { player: u8, count: u8, available: u32 },
    AnnounceCard { player: u8, options: Vec<u64> },
    AnnounceNumber { player: u8, options: Vec<u64> },
    Unknown { msg_type: u8, data: Vec<u8> },
}

struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Reader { data, offset: 0 }
    }

    #[allow(dead_code)]
    fn has_remaining(&self) -> bool {
        self.offset < self.data.len()
    }

    fn read_u8(&mut self) -> Result<u8> {
        if self.offset + 1 > self.data.len() {
            return Err(Error::InvalidMessage("unexpected EOF reading u8".into()));
        }
        let b = self.data[self.offset];
        self.offset += 1;
        Ok(b)
    }

    fn read_u16(&mut self) -> Result<u16> {
        if self.offset + 2 > self.data.len() {
            return Err(Error::InvalidMessage("unexpected EOF reading u16".into()));
        }
        let val = u16::from_le_bytes([self.data[self.offset], self.data[self.offset + 1]]);
        self.offset += 2;
        Ok(val)
    }

    fn read_u32(&mut self) -> Result<u32> {
        if self.offset + 4 > self.data.len() {
            return Err(Error::InvalidMessage("unexpected EOF reading u32".into()));
        }
        let val = u32::from_le_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ]);
        self.offset += 4;
        Ok(val)
    }

    fn read_u64(&mut self) -> Result<u64> {
        if self.offset + 8 > self.data.len() {
            return Err(Error::InvalidMessage("unexpected EOF reading u64".into()));
        }
        let val = u64::from_le_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
            self.data[self.offset + 4],
            self.data[self.offset + 5],
            self.data[self.offset + 6],
            self.data[self.offset + 7],
        ]);
        self.offset += 8;
        Ok(val)
    }

    fn read_loc_info(&mut self) -> Result<CardLocation> {
        Ok(CardLocation {
            controller: self.read_u8()?,
            location: self.read_u8()?,
            sequence: self.read_u32()?,
            position: self.read_u32()?,
        })
    }

    fn remaining_bytes(&self) -> Vec<u8> {
        self.data[self.offset..].to_vec()
    }
}

impl CoreMessage {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(Error::InvalidMessage("empty message buffer".into()));
        }
        let mut r = Reader::new(bytes);
        let msg_type = r.read_u8()?;

        let msg = match msg_type {
            MSG_RETRY => CoreMessage::Retry,
            MSG_HINT => CoreMessage::Hint {
                kind: r.read_u8()?,
                player: r.read_u8()?,
                value: r.read_u64()?,
            },
            MSG_WAITING => CoreMessage::Waiting,
            MSG_START => CoreMessage::Start {
                playertype: r.read_u8()?,
                lp0: r.read_u32()?,
                lp1: r.read_u32()?,
                deck0: r.read_u16()?,
                deck1: r.read_u16()?,
                extra0: r.read_u16()?,
                extra1: r.read_u16()?,
            },
            MSG_WIN => CoreMessage::Win {
                player: r.read_u8()?,
                reason: r.read_u8()?,
            },
            MSG_SELECT_BATTLECMD => {
                let player = r.read_u8()?;
                let act_count = r.read_u32()? as usize;
                let mut activatable = Vec::with_capacity(act_count);
                for _ in 0..act_count {
                    activatable.push(CmdActivatableItem {
                        code: r.read_u32()?,
                        controller: r.read_u8()?,
                        location: r.read_u8()?,
                        sequence: r.read_u32()?,
                        desc: r.read_u64()?,
                        client_mode: r.read_u8()?,
                    });
                }
                let atk_count = r.read_u32()? as usize;
                let mut attackable = Vec::with_capacity(atk_count);
                for _ in 0..atk_count {
                    attackable.push(CmdAttackableItem {
                        code: r.read_u32()?,
                        controller: r.read_u8()?,
                        location: r.read_u8()?,
                        sequence: r.read_u8()?,
                        direct_attackable: r.read_u8()?,
                    });
                }
                let can_m2 = r.read_u8()? != 0;
                let can_ep = r.read_u8()? != 0;
                CoreMessage::SelectBattleCmd {
                    player,
                    activatable,
                    attackable,
                    can_m2,
                    can_ep,
                }
            }
            MSG_SELECT_IDLECMD => {
                let player = r.read_u8()?;
                let read_cards = |r: &mut Reader| -> Result<Vec<CmdCardItem>> {
                    let count = r.read_u32()? as usize;
                    let mut list = Vec::with_capacity(count);
                    for _ in 0..count {
                        list.push(CmdCardItem {
                            code: r.read_u32()?,
                            controller: r.read_u8()?,
                            location: r.read_u8()?,
                            sequence: r.read_u32()?,
                        });
                    }
                    Ok(list)
                };

                let summonable = read_cards(&mut r)?;
                let spsummonable = read_cards(&mut r)?;
                let reposable = read_cards(&mut r)?;
                let msetable = read_cards(&mut r)?;
                let ssetable = read_cards(&mut r)?;

                let act_count = r.read_u32()? as usize;
                let mut activatable = Vec::with_capacity(act_count);
                for _ in 0..act_count {
                    activatable.push(CmdActivatableItem {
                        code: r.read_u32()?,
                        controller: r.read_u8()?,
                        location: r.read_u8()?,
                        sequence: r.read_u32()?,
                        desc: r.read_u64()?,
                        client_mode: r.read_u8()?,
                    });
                }

                let can_bp = r.read_u8()? != 0;
                let can_ep = r.read_u8()? != 0;
                let can_shuffle = r.read_u8()? != 0;

                CoreMessage::SelectIdleCmd {
                    player,
                    summonable,
                    spsummonable,
                    reposable,
                    msetable,
                    ssetable,
                    activatable,
                    can_bp,
                    can_ep,
                    can_shuffle,
                }
            }
            MSG_SELECT_EFFECTYN => CoreMessage::SelectEffectYesNo {
                player: r.read_u8()?,
                code: r.read_u32()?,
                location: r.read_loc_info()?,
                desc: r.read_u64()?,
            },
            MSG_SELECT_YESNO => CoreMessage::SelectYesNo {
                player: r.read_u8()?,
                desc: r.read_u64()?,
            },
            MSG_SELECT_OPTION => {
                let player = r.read_u8()?;
                let count = r.read_u8()? as usize;
                let mut options = Vec::with_capacity(count);
                for _ in 0..count {
                    options.push(r.read_u64()?);
                }
                CoreMessage::SelectOption { player, options }
            }
            MSG_SELECT_CARD => {
                let player = r.read_u8()?;
                let cancelable = r.read_u8()? != 0;
                let min = r.read_u32()? as usize;
                let max = r.read_u32()? as usize;
                let count = r.read_u32()? as usize;
                let mut cards = Vec::with_capacity(count);
                for _ in 0..count {
                    cards.push(SelectCardItem {
                        code: r.read_u32()?,
                        location: r.read_loc_info()?,
                    });
                }
                CoreMessage::SelectCard {
                    player,
                    cancelable,
                    min,
                    max,
                    cards,
                }
            }
            MSG_SELECT_CHAIN => {
                let player = r.read_u8()?;
                let spe_count = r.read_u8()?;
                let forced = r.read_u8()? != 0;
                let hint0 = r.read_u32()?;
                let hint1 = r.read_u32()?;
                let count = r.read_u32()? as usize;
                let mut chains = Vec::with_capacity(count);
                for _ in 0..count {
                    chains.push(SelectChainItem {
                        code: r.read_u32()?,
                        location: r.read_loc_info()?,
                        desc: r.read_u64()?,
                        client_mode: r.read_u8()?,
                    });
                }
                CoreMessage::SelectChain {
                    player,
                    spe_count,
                    forced,
                    hint0,
                    hint1,
                    chains,
                }
            }
            MSG_SELECT_POSITION => CoreMessage::SelectPosition {
                player: r.read_u8()?,
                code: r.read_u32()?,
                positions: r.read_u8()?,
            },
            MSG_SELECT_TRIBUTE => {
                let player = r.read_u8()?;
                let cancelable = r.read_u8()? != 0;
                let min = r.read_u32()? as usize;
                let max = r.read_u32()? as usize;
                let count = r.read_u32()? as usize;
                let mut cards = Vec::with_capacity(count);
                for _ in 0..count {
                    cards.push(SelectCardItem {
                        code: r.read_u32()?,
                        location: r.read_loc_info()?,
                    });
                }
                CoreMessage::SelectTribute {
                    player,
                    cancelable,
                    min,
                    max,
                    cards,
                }
            }
            MSG_SELECT_UNSELECT_CARD => {
                let player = r.read_u8()?;
                let finishable = r.read_u8()? != 0;
                let cancelable = r.read_u8()? != 0;
                let min = r.read_u32()? as usize;
                let max = r.read_u32()? as usize;
                let sel_count = r.read_u32()? as usize;
                let mut selectable = Vec::with_capacity(sel_count);
                for _ in 0..sel_count {
                    selectable.push(SelectCardItem {
                        code: r.read_u32()?,
                        location: r.read_loc_info()?,
                    });
                }
                let unsel_count = r.read_u32()? as usize;
                let mut unselectable = Vec::with_capacity(unsel_count);
                for _ in 0..unsel_count {
                    unselectable.push(SelectCardItem {
                        code: r.read_u32()?,
                        location: r.read_loc_info()?,
                    });
                }
                CoreMessage::SelectUnselectCard {
                    player,
                    finishable,
                    cancelable,
                    min,
                    max,
                    selectable,
                    unselectable,
                }
            }
            MSG_SHUFFLE_DECK => CoreMessage::ShuffleDeck { player: r.read_u8()? },
            MSG_SHUFFLE_HAND => {
                let player = r.read_u8()?;
                let count = r.read_u32()? as usize;
                let mut cards = Vec::with_capacity(count);
                for _ in 0..count {
                    cards.push(r.read_u32()?);
                }
                CoreMessage::ShuffleHand { player, cards }
            }
            MSG_NEW_TURN => CoreMessage::NewTurn { player: r.read_u8()? },
            MSG_NEW_PHASE => CoreMessage::NewPhase { phase: r.read_u16()? },
            MSG_MOVE => CoreMessage::Move {
                code: r.read_u32()?,
                from_con: r.read_u8()?,
                from_loc: r.read_u8()?,
                from_seq: r.read_u8()?,
                from_pos: r.read_u8()?,
                to_con: r.read_u8()?,
                to_loc: r.read_u8()?,
                to_seq: r.read_u8()?,
                to_pos: r.read_u8()?,
                reason: r.read_u32()?,
            },
            MSG_POS_CHANGE => CoreMessage::PosChange {
                code: r.read_u32()?,
                con: r.read_u8()?,
                loc: r.read_u8()?,
                seq: r.read_u8()?,
                prev_pos: r.read_u8()?,
                current_pos: r.read_u8()?,
            },
            MSG_SET => CoreMessage::Set {
                code: r.read_u32()?,
                con: r.read_u8()?,
                loc: r.read_u8()?,
                seq: r.read_u8()?,
                pos: r.read_u8()?,
            },
            MSG_SUMMONING => CoreMessage::Summoning {
                code: r.read_u32()?,
                con: r.read_u8()?,
                loc: r.read_u8()?,
                seq: r.read_u8()?,
                pos: r.read_u8()?,
            },
            MSG_SUMMONED => CoreMessage::Summoned,
            MSG_SPSUMMONING => CoreMessage::SpSummoning {
                code: r.read_u32()?,
                con: r.read_u8()?,
                loc: r.read_u8()?,
                seq: r.read_u8()?,
                pos: r.read_u8()?,
            },
            MSG_SPSUMMONED => CoreMessage::SpSummoned,
            MSG_FLIPSUMMONING => CoreMessage::FlipSummoning {
                code: r.read_u32()?,
                con: r.read_u8()?,
                loc: r.read_u8()?,
                seq: r.read_u8()?,
                pos: r.read_u8()?,
            },
            MSG_FLIPSUMMONED => CoreMessage::FlipSummoned,
            MSG_CHAINING => CoreMessage::Chaining {
                code: r.read_u32()?,
                con: r.read_u8()?,
                loc: r.read_u8()?,
                seq: r.read_u8()?,
                tc: r.read_u32()?,
                desc: r.read_u64()?,
                chain_link: r.read_u8()?,
            },
            MSG_CHAINED => CoreMessage::Chained { chain_link: r.read_u8()? },
            MSG_CHAIN_SOLVING => CoreMessage::ChainSolving { chain_link: r.read_u8()? },
            MSG_CHAIN_SOLVED => CoreMessage::ChainSolved { chain_link: r.read_u8()? },
            MSG_CHAIN_END => CoreMessage::ChainEnd,
            MSG_CHAIN_NEGATED => CoreMessage::ChainNegated { chain_link: r.read_u8()? },
            MSG_CHAIN_DISABLED => CoreMessage::ChainDisabled { chain_link: r.read_u8()? },
            MSG_CARD_SELECTED => {
                let player = r.read_u8()?;
                let count = r.read_u32()? as usize;
                let mut cards = Vec::with_capacity(count);
                for _ in 0..count {
                    cards.push(r.read_u32()?);
                }
                CoreMessage::CardSelected { player, cards }
            }
            MSG_DRAW => {
                let player = r.read_u8()?;
                let count = r.read_u8()?;
                let mut cards = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    cards.push(r.read_u32()?);
                }
                CoreMessage::Draw { player, count, cards }
            }
            MSG_DAMAGE => CoreMessage::Damage {
                player: r.read_u8()?,
                amount: r.read_u32()?,
            },
            MSG_RECOVER => CoreMessage::Recover {
                player: r.read_u8()?,
                amount: r.read_u32()?,
            },
            MSG_LPUPDATE => CoreMessage::LpUpdate {
                player: r.read_u8()?,
                lp: r.read_u32()?,
            },
            MSG_PAY_LPCOST => CoreMessage::PayLpCost {
                player: r.read_u8()?,
                amount: r.read_u32()?,
            },
            MSG_ATTACK => CoreMessage::Attack {
                attacker_code: r.read_u32()?,
                attacker_con: r.read_u8()?,
                attacker_loc: r.read_u8()?,
                attacker_seq: r.read_u8()?,
                attacker_pos: r.read_u8()?,
                target_con: r.read_u8()?,
                target_loc: r.read_u8()?,
                target_seq: r.read_u8()?,
                target_pos: r.read_u8()?,
            },
            MSG_ATTACK_DISABLED => CoreMessage::AttackDisabled,
            MSG_DAMAGE_STEP_START => CoreMessage::DamageStepStart,
            MSG_DAMAGE_STEP_END => CoreMessage::DamageStepEnd,
            MSG_TOSS_COIN => {
                let player = r.read_u8()?;
                let count = r.read_u8()? as usize;
                let mut results = Vec::with_capacity(count);
                for _ in 0..count {
                    results.push(r.read_u8()?);
                }
                CoreMessage::TossCoin { player, results }
            }
            MSG_TOSS_DICE => {
                let player = r.read_u8()?;
                let count = r.read_u8()? as usize;
                let mut results = Vec::with_capacity(count);
                for _ in 0..count {
                    results.push(r.read_u8()?);
                }
                CoreMessage::TossDice { player, results }
            }
            MSG_ANNOUNCE_RACE => CoreMessage::AnnounceRace {
                player: r.read_u8()?,
                count: r.read_u8()?,
                available: r.read_u64()?,
            },
            MSG_ANNOUNCE_ATTRIB => CoreMessage::AnnounceAttrib {
                player: r.read_u8()?,
                count: r.read_u8()?,
                available: r.read_u32()?,
            },
            MSG_ANNOUNCE_CARD => {
                let player = r.read_u8()?;
                let count = r.read_u8()? as usize;
                let mut options = Vec::with_capacity(count);
                for _ in 0..count {
                    options.push(r.read_u64()?);
                }
                CoreMessage::AnnounceCard { player, options }
            }
            MSG_ANNOUNCE_NUMBER => {
                let player = r.read_u8()?;
                let count = r.read_u8()? as usize;
                let mut options = Vec::with_capacity(count);
                for _ in 0..count {
                    options.push(r.read_u64()?);
                }
                CoreMessage::AnnounceNumber { player, options }
            }
            other => CoreMessage::Unknown {
                msg_type: other,
                data: r.remaining_bytes(),
            },
        };

        Ok(msg)
    }

    /// `true` if this message represents an interactive prompt awaiting a player decision.
    pub fn is_prompt(&self) -> bool {
        matches!(
            self,
            CoreMessage::SelectIdleCmd { .. }
                | CoreMessage::SelectBattleCmd { .. }
                | CoreMessage::SelectCard { .. }
                | CoreMessage::SelectChain { .. }
                | CoreMessage::SelectEffectYesNo { .. }
                | CoreMessage::SelectYesNo { .. }
                | CoreMessage::SelectOption { .. }
                | CoreMessage::SelectPosition { .. }
                | CoreMessage::SelectTribute { .. }
                | CoreMessage::SelectUnselectCard { .. }
                | CoreMessage::SelectPlace { .. }
                | CoreMessage::SelectDisfield { .. }
                | CoreMessage::AnnounceRace { .. }
                | CoreMessage::AnnounceAttrib { .. }
                | CoreMessage::AnnounceCard { .. }
                | CoreMessage::AnnounceNumber { .. }
        )
    }

    /// The player ID (0 or 1) that needs to respond, if this is a prompt.
    pub fn prompt_player(&self) -> Option<u8> {
        match self {
            CoreMessage::SelectIdleCmd { player, .. }
            | CoreMessage::SelectBattleCmd { player, .. }
            | CoreMessage::SelectCard { player, .. }
            | CoreMessage::SelectChain { player, .. }
            | CoreMessage::SelectEffectYesNo { player, .. }
            | CoreMessage::SelectYesNo { player, .. }
            | CoreMessage::SelectOption { player, .. }
            | CoreMessage::SelectPosition { player, .. }
            | CoreMessage::SelectTribute { player, .. }
            | CoreMessage::SelectUnselectCard { player, .. }
            | CoreMessage::SelectPlace { player, .. }
            | CoreMessage::SelectDisfield { player, .. }
            | CoreMessage::AnnounceRace { player, .. }
            | CoreMessage::AnnounceAttrib { player, .. }
            | CoreMessage::AnnounceCard { player, .. }
            | CoreMessage::AnnounceNumber { player, .. } => Some(*player),
            _ => None,
        }
    }
}

/// Parse a stream of length-prefixed messages as returned by `OCG_DuelGetMessage`.
pub fn parse_message_stream(mut bytes: &[u8]) -> Result<Vec<CoreMessage>> {
    let mut messages = Vec::new();
    while !bytes.is_empty() {
        if bytes.len() < 4 {
            return Err(Error::InvalidMessage(
                "truncated message length header".into(),
            ));
        }
        let size = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
        bytes = &bytes[4..];
        if bytes.len() < size {
            return Err(Error::InvalidMessage(format!(
                "expected {size} bytes for message, but only {} bytes remain",
                bytes.len()
            )));
        }
        let msg_bytes = &bytes[..size];
        bytes = &bytes[size..];
        if !msg_bytes.is_empty() {
            let msg = CoreMessage::decode(msg_bytes)?;
            messages.push(msg);
        }
    }
    Ok(messages)
}

