//! PlayerResponse types and binary response serialization for ocgcore.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleCmdChoice {
    Summon(usize),
    SpSummon(usize),
    Repos(usize),
    SetMonster(usize),
    SetSpell(usize),
    Activate(usize),
    ToBattlePhase,
    ToEndPhase,
    ShuffleHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleCmdChoice {
    Activate(usize),
    Attack(usize),
    ToMainPhase2,
    ToEndPhase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerResponse {
    IdleCmd(IdleCmdChoice),
    BattleCmd(BattleCmdChoice),
    EffectYesNo(bool),
    YesNo(bool),
    Option(u32),
    Position(u32),
    Chain(Option<usize>),
    CardIndices(Vec<usize>),
    SelectUnselect(Option<usize>),
    Cancel,
    AnnounceRace(u64),
    AnnounceAttribute(u32),
    AnnounceCard(u32),
    AnnounceNumber(usize),
    SelectPlace(Vec<(u8, u8, u8)>),
    Raw(Vec<u8>),
}

impl PlayerResponse {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            PlayerResponse::IdleCmd(choice) => {
                let val: u32 = match choice {
                    IdleCmdChoice::Summon(s) => ((*s as u32) << 16) | 0,
                    IdleCmdChoice::SpSummon(s) => ((*s as u32) << 16) | 1,
                    IdleCmdChoice::Repos(s) => ((*s as u32) << 16) | 2,
                    IdleCmdChoice::SetMonster(s) => ((*s as u32) << 16) | 3,
                    IdleCmdChoice::SetSpell(s) => ((*s as u32) << 16) | 4,
                    IdleCmdChoice::Activate(s) => ((*s as u32) << 16) | 5,
                    IdleCmdChoice::ToBattlePhase => 6,
                    IdleCmdChoice::ToEndPhase => 7,
                    IdleCmdChoice::ShuffleHand => 8,
                };
                val.to_le_bytes().to_vec()
            }
            PlayerResponse::BattleCmd(choice) => {
                let val: u32 = match choice {
                    BattleCmdChoice::Activate(s) => ((*s as u32) << 16) | 0,
                    BattleCmdChoice::Attack(s) => ((*s as u32) << 16) | 1,
                    BattleCmdChoice::ToMainPhase2 => 2,
                    BattleCmdChoice::ToEndPhase => 3,
                };
                val.to_le_bytes().to_vec()
            }
            PlayerResponse::EffectYesNo(val) => {
                let num: i32 = if *val { 1 } else { 0 };
                num.to_le_bytes().to_vec()
            }
            PlayerResponse::YesNo(val) => {
                let num: i32 = if *val { 1 } else { 0 };
                num.to_le_bytes().to_vec()
            }
            PlayerResponse::Option(idx) => (*idx as i32).to_le_bytes().to_vec(),
            PlayerResponse::Position(pos) => (*pos as i32).to_le_bytes().to_vec(),
            PlayerResponse::Chain(opt) => {
                let val: i32 = match opt {
                    Some(idx) => *idx as i32,
                    None => -1,
                };
                val.to_le_bytes().to_vec()
            }
            PlayerResponse::Cancel => (-1i32).to_le_bytes().to_vec(),
            PlayerResponse::CardIndices(indices) => {
                // Type 0: [0: i32, size: u32, idx0: u32, idx1: u32, ...]
                let mut bytes = Vec::with_capacity(8 + indices.len() * 4);
                bytes.extend_from_slice(&(0i32).to_le_bytes());
                bytes.extend_from_slice(&(indices.len() as u32).to_le_bytes());
                for &idx in indices {
                    bytes.extend_from_slice(&(idx as u32).to_le_bytes());
                }
                bytes
            }
            PlayerResponse::SelectUnselect(opt) => {
                match opt {
                    Some(idx) => {
                        let mut bytes = Vec::with_capacity(8);
                        bytes.extend_from_slice(&(1i32).to_le_bytes());
                        bytes.extend_from_slice(&(*idx as i32).to_le_bytes());
                        bytes
                    }
                    None => (-1i32).to_le_bytes().to_vec(),
                }
            }
            PlayerResponse::AnnounceRace(mask) => mask.to_le_bytes().to_vec(),
            PlayerResponse::AnnounceAttribute(mask) => mask.to_le_bytes().to_vec(),
            PlayerResponse::AnnounceCard(code) => (*code as i32).to_le_bytes().to_vec(),
            PlayerResponse::AnnounceNumber(idx) => (*idx as i32).to_le_bytes().to_vec(),
            PlayerResponse::SelectPlace(zones) => {
                let mut bytes = Vec::with_capacity(zones.len() * 3);
                for &(con, loc, seq) in zones {
                    bytes.push(con);
                    bytes.push(loc);
                    bytes.push(seq);
                }
                bytes
            }
            PlayerResponse::Raw(bytes) => bytes.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_cmd_encoding() {
        assert_eq!(
            PlayerResponse::IdleCmd(IdleCmdChoice::ToBattlePhase).to_bytes(),
            6u32.to_le_bytes()
        );
        assert_eq!(
            PlayerResponse::IdleCmd(IdleCmdChoice::ToEndPhase).to_bytes(),
            7u32.to_le_bytes()
        );
        let summon_1 = PlayerResponse::IdleCmd(IdleCmdChoice::Summon(1)).to_bytes();
        assert_eq!(summon_1, (1u32 << 16).to_le_bytes());
    }

    #[test]
    fn test_chain_encoding() {
        assert_eq!(PlayerResponse::Chain(None).to_bytes(), (-1i32).to_le_bytes());
        assert_eq!(PlayerResponse::Chain(Some(2)).to_bytes(), 2i32.to_le_bytes());
    }
}
