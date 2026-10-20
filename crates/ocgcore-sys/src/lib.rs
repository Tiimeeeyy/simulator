//! Unsafe C/C++ FFI bindings to the ygopro ocgcore rules engine.
//!
//! Statically links against `ocgcore` and Lua 5.4.

#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::{c_char, c_int, c_void};

pub type OCG_Duel = *mut c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OCG_LogTypes {
    Error = 0,
    FromScript = 1,
    ForDebug = 2,
    Undefined = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OCG_DuelCreationStatus {
    Success = 0,
    NoOutput = 1,
    NotCreated = 2,
    NullDataReader = 3,
    NullScriptReader = 4,
    IncompatibleLuaApi = 5,
    NullRngSeed = 6,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OCG_DuelStatus {
    End = 0,
    Awaiting = 1,
    Continue = 2,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct OCG_CardData {
    pub code: u32,
    pub alias: u32,
    pub setcodes: *mut u16,
    pub type_: u32,
    pub level: u32,
    pub attribute: u32,
    pub race: u64,
    pub attack: i32,
    pub defense: i32,
    pub lscale: u32,
    pub rscale: u32,
    pub link_marker: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct OCG_Player {
    pub startingLP: u32,
    pub startingDrawCount: u32,
    pub drawCountPerTurn: u32,
}

pub type OCG_DataReader =
    Option<unsafe extern "C" fn(payload: *mut c_void, code: u32, data: *mut OCG_CardData)>;
pub type OCG_DataReaderDone =
    Option<unsafe extern "C" fn(payload: *mut c_void, data: *mut OCG_CardData)>;
pub type OCG_ScriptReader = Option<
    unsafe extern "C" fn(payload: *mut c_void, duel: OCG_Duel, name: *const c_char) -> c_int,
>;
pub type OCG_LogHandler =
    Option<unsafe extern "C" fn(payload: *mut c_void, string: *const c_char, log_type: c_int)>;

#[repr(C)]
pub struct OCG_DuelOptions {
    pub seed: [u64; 4],
    pub flags: u64,
    pub team1: OCG_Player,
    pub team2: OCG_Player,
    pub cardReader: OCG_DataReader,
    pub payload1: *mut c_void,
    pub scriptReader: OCG_ScriptReader,
    pub payload2: *mut c_void,
    pub logHandler: OCG_LogHandler,
    pub payload3: *mut c_void,
    pub cardReaderDone: OCG_DataReaderDone,
    pub payload4: *mut c_void,
    pub enableUnsafeLibraries: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct OCG_NewCardInfo {
    pub team: u8,
    pub duelist: u8,
    pub code: u32,
    pub con: u8,
    pub loc: u32,
    pub seq: u32,
    pub pos: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct OCG_QueryInfo {
    pub flags: u32,
    pub con: u8,
    pub loc: u32,
    pub seq: u32,
    pub overlay_seq: u32,
}

// ---------------------------------------------------------------------------
// Engine Constants
// ---------------------------------------------------------------------------

pub const LOCATION_DECK: u32 = 0x01;
pub const LOCATION_HAND: u32 = 0x02;
pub const LOCATION_MZONE: u32 = 0x04;
pub const LOCATION_SZONE: u32 = 0x08;
pub const LOCATION_GRAVE: u32 = 0x10;
pub const LOCATION_REMOVED: u32 = 0x20;
pub const LOCATION_EXTRA: u32 = 0x40;
pub const LOCATION_OVERLAY: u32 = 0x80;
pub const LOCATION_ONFIELD: u32 = LOCATION_MZONE | LOCATION_SZONE;

pub const POS_FACEUP_ATTACK: u32 = 0x1;
pub const POS_FACEDOWN_ATTACK: u32 = 0x2;
pub const POS_FACEUP_DEFENSE: u32 = 0x4;
pub const POS_FACEDOWN_DEFENSE: u32 = 0x8;
pub const POS_FACEUP: u32 = POS_FACEUP_ATTACK | POS_FACEUP_DEFENSE;
pub const POS_FACEDOWN: u32 = POS_FACEDOWN_ATTACK | POS_FACEDOWN_DEFENSE;
pub const POS_ATTACK: u32 = POS_FACEUP_ATTACK | POS_FACEDOWN_ATTACK;
pub const POS_DEFENSE: u32 = POS_FACEUP_DEFENSE | POS_FACEDOWN_DEFENSE;

// Duel modes
pub const DUEL_TEST_MODE: u64 = 0x01;
pub const DUEL_ATTACK_FIRST_TURN: u64 = 0x02;
pub const DUEL_USE_TRAPS_IN_NEW_CHAIN: u64 = 0x04;
pub const DUEL_6_STEP_BATLLE_STEP: u64 = 0x08;
pub const DUEL_PSEUDO_SHUFFLE: u64 = 0x10;
pub const DUEL_TRIGGER_WHEN_PRIVATE_KNOWLEDGE: u64 = 0x20;
pub const DUEL_SIMPLE_AI: u64 = 0x40;
pub const DUEL_RELAY: u64 = 0x80;
pub const DUEL_OCG_OBSOLETE_IGNITION: u64 = 0x100;
pub const DUEL_1ST_TURN_DRAW: u64 = 0x200;
pub const DUEL_1_FACEUP_FIELD: u64 = 0x400;
pub const DUEL_PZONE: u64 = 0x800;
pub const DUEL_SEPARATE_PZONE: u64 = 0x1000;
pub const DUEL_EMZONE: u64 = 0x2000;
pub const DUEL_FSX_MMZONE: u64 = 0x4000;
pub const DUEL_TRAP_MONSTERS_NOT_USE_ZONE: u64 = 0x8000;
pub const DUEL_RETURN_TO_DECK_TRIGGERS: u64 = 0x10000;
pub const DUEL_TRIGGER_ONLY_IN_LOCATION: u64 = 0x20000;

pub const DUEL_MODE_MR5: u64 = DUEL_PZONE
    | DUEL_EMZONE
    | DUEL_FSX_MMZONE
    | DUEL_TRAP_MONSTERS_NOT_USE_ZONE
    | DUEL_TRIGGER_ONLY_IN_LOCATION;

// Message codes
pub const MSG_RETRY: u8 = 1;
pub const MSG_HINT: u8 = 2;
pub const MSG_WAITING: u8 = 3;
pub const MSG_START: u8 = 4;
pub const MSG_WIN: u8 = 5;
pub const MSG_UPDATE_DATA: u8 = 6;
pub const MSG_UPDATE_CARD: u8 = 7;
pub const MSG_REQUEST_DECK: u8 = 8;
pub const MSG_SELECT_BATTLECMD: u8 = 10;
pub const MSG_SELECT_IDLECMD: u8 = 11;
pub const MSG_SELECT_EFFECTYN: u8 = 12;
pub const MSG_SELECT_YESNO: u8 = 13;
pub const MSG_SELECT_OPTION: u8 = 14;
pub const MSG_SELECT_CARD: u8 = 15;
pub const MSG_SELECT_CHAIN: u8 = 16;
pub const MSG_SELECT_PLACE: u8 = 18;
pub const MSG_SELECT_POSITION: u8 = 19;
pub const MSG_SELECT_TRIBUTE: u8 = 20;
pub const MSG_SORT_CHAIN: u8 = 21;
pub const MSG_SELECT_COUNTER: u8 = 22;
pub const MSG_SELECT_SUM: u8 = 23;
pub const MSG_SELECT_DISFIELD: u8 = 24;
pub const MSG_SORT_CARD: u8 = 25;
pub const MSG_SELECT_UNSELECT_CARD: u8 = 26;
pub const MSG_CONFIRM_DECKTOP: u8 = 30;
pub const MSG_CONFIRM_CARDS: u8 = 31;
pub const MSG_SHUFFLE_DECK: u8 = 32;
pub const MSG_SHUFFLE_HAND: u8 = 33;
pub const MSG_REFRESH_DECK: u8 = 34;
pub const MSG_SWAP_GRAVE_DECK: u8 = 35;
pub const MSG_SHUFFLE_SET_CARD: u8 = 36;
pub const MSG_REVERSE_DECK: u8 = 37;
pub const MSG_DECK_TOP: u8 = 38;
pub const MSG_SHUFFLE_EXTRA: u8 = 39;
pub const MSG_NEW_TURN: u8 = 40;
pub const MSG_NEW_PHASE: u8 = 41;
pub const MSG_CONFIRM_EXTRATOP: u8 = 42;
pub const MSG_MOVE: u8 = 50;
pub const MSG_POS_CHANGE: u8 = 53;
pub const MSG_SET: u8 = 54;
pub const MSG_SWAP: u8 = 55;
pub const MSG_FIELD_DISABLED: u8 = 56;
pub const MSG_SUMMONING: u8 = 60;
pub const MSG_SUMMONED: u8 = 61;
pub const MSG_SPSUMMONING: u8 = 62;
pub const MSG_SPSUMMONED: u8 = 63;
pub const MSG_FLIPSUMMONING: u8 = 64;
pub const MSG_FLIPSUMMONED: u8 = 65;
pub const MSG_CHAINING: u8 = 70;
pub const MSG_CHAINED: u8 = 71;
pub const MSG_CHAIN_SOLVING: u8 = 72;
pub const MSG_CHAIN_SOLVED: u8 = 73;
pub const MSG_CHAIN_END: u8 = 74;
pub const MSG_CHAIN_NEGATED: u8 = 75;
pub const MSG_CHAIN_DISABLED: u8 = 76;
pub const MSG_CARD_SELECTED: u8 = 80;
pub const MSG_RANDOM_SELECTED: u8 = 81;
pub const MSG_BECOME_TARGET: u8 = 83;
pub const MSG_DRAW: u8 = 90;
pub const MSG_DAMAGE: u8 = 91;
pub const MSG_RECOVER: u8 = 92;
pub const MSG_EQUIP: u8 = 93;
pub const MSG_LPUPDATE: u8 = 94;
pub const MSG_UNEQUIP: u8 = 95;
pub const MSG_CARD_TARGET: u8 = 96;
pub const MSG_CANCEL_TARGET: u8 = 97;
pub const MSG_PAY_LPCOST: u8 = 100;
pub const MSG_ADD_COUNTER: u8 = 101;
pub const MSG_REMOVE_COUNTER: u8 = 102;
pub const MSG_ATTACK: u8 = 110;
pub const MSG_BATTLE: u8 = 111;
pub const MSG_ATTACK_DISABLED: u8 = 112;
pub const MSG_DAMAGE_STEP_START: u8 = 113;
pub const MSG_DAMAGE_STEP_END: u8 = 114;
pub const MSG_MISSED_EFFECT: u8 = 120;
pub const MSG_BE_CHAIN_TARGET: u8 = 121;
pub const MSG_CREATE_RELATION: u8 = 122;
pub const MSG_RELEASE_RELATION: u8 = 123;
pub const MSG_TOSS_COIN: u8 = 130;
pub const MSG_TOSS_DICE: u8 = 131;
pub const MSG_ROCK_PAPER_SCISSORS: u8 = 132;
pub const MSG_HAND_RES: u8 = 133;
pub const MSG_ANNOUNCE_RACE: u8 = 140;
pub const MSG_ANNOUNCE_ATTRIB: u8 = 141;
pub const MSG_ANNOUNCE_CARD: u8 = 142;
pub const MSG_ANNOUNCE_NUMBER: u8 = 143;
pub const MSG_CARD_HINT: u8 = 160;
pub const MSG_TAG_SWAP: u8 = 161;
pub const MSG_RELOAD_FIELD: u8 = 162;
pub const MSG_AI_NAME: u8 = 163;
pub const MSG_SHOW_HINT: u8 = 164;
pub const MSG_PLAYER_HINT: u8 = 165;
pub const MSG_MATCH_KILL: u8 = 170;
pub const MSG_CUSTOM_MSG: u8 = 180;
pub const MSG_REMOVE_CARDS: u8 = 190;

// Query flags
pub const QUERY_CODE: u32 = 0x1;
pub const QUERY_POSITION: u32 = 0x2;
pub const QUERY_ALIAS: u32 = 0x4;
pub const QUERY_TYPE: u32 = 0x8;
pub const QUERY_LEVEL: u32 = 0x10;
pub const QUERY_RANK: u32 = 0x20;
pub const QUERY_ATTRIBUTE: u32 = 0x40;
pub const QUERY_RACE: u32 = 0x80;
pub const QUERY_ATTACK: u32 = 0x100;
pub const QUERY_DEFENSE: u32 = 0x200;
pub const QUERY_BASE_ATTACK: u32 = 0x400;
pub const QUERY_BASE_DEFENSE: u32 = 0x800;
pub const QUERY_REASON: u32 = 0x1000;
pub const QUERY_REASON_CARD: u32 = 0x2000;
pub const QUERY_EQUIP_CARD: u32 = 0x4000;
pub const QUERY_TARGET_CARD: u32 = 0x8000;
pub const QUERY_OVERLAY_CARD: u32 = 0x10000;
pub const QUERY_COUNTERS: u32 = 0x20000;
pub const QUERY_OWNER: u32 = 0x40000;
pub const QUERY_STATUS: u32 = 0x80000;
pub const QUERY_LSCALE: u32 = 0x200000;
pub const QUERY_RSCALE: u32 = 0x400000;
pub const QUERY_LINK: u32 = 0x800000;

// ---------------------------------------------------------------------------
// C API Functions
// ---------------------------------------------------------------------------

unsafe extern "C" {
    pub fn OCG_GetVersion(major: *mut c_int, minor: *mut c_int);
    pub fn OCG_CreateDuel(
        out_ocg_duel: *mut OCG_Duel,
        options_ptr: *const OCG_DuelOptions,
    ) -> c_int;
    pub fn OCG_DestroyDuel(ocg_duel: OCG_Duel);
    pub fn OCG_DuelNewCard(ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo);
    pub fn OCG_StartDuel(ocg_duel: OCG_Duel);
    pub fn OCG_DuelProcess(ocg_duel: OCG_Duel) -> c_int;
    pub fn OCG_DuelGetMessage(ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void;
    pub fn OCG_DuelSetResponse(ocg_duel: OCG_Duel, buffer: *const c_void, length: u32);
    pub fn OCG_LoadScript(
        ocg_duel: OCG_Duel,
        buffer: *const c_char,
        length: u32,
        name: *const c_char,
    ) -> c_int;
    pub fn OCG_DuelQueryCount(ocg_duel: OCG_Duel, team: u8, loc: u32) -> u32;
    pub fn OCG_DuelQuery(
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void;
    pub fn OCG_DuelQueryLocation(
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void;
    pub fn OCG_DuelQueryField(ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocg_version() {
        let mut major = 0;
        let mut minor = 0;
        unsafe {
            OCG_GetVersion(&mut major, &mut minor);
        }
        assert!(major >= 11, "Expected ocgcore version >= 11, got {major}.{minor}");
    }
}
