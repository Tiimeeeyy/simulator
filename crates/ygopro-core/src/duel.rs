//! Safe wrapper around an active `ocgcore` duel simulation.

use std::cell::RefCell;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::sync::Arc;

use ocgcore_sys::*;
use ygopro_data::{CardDatabase, Deck};

use crate::error::{Error, Result};
use crate::message::{parse_message_stream, CoreMessage};
use crate::response::PlayerResponse;
use crate::script::ScriptLoader;

thread_local! {
    static SETCODES_BUF: RefCell<[u16; 16]> = const { RefCell::new([0u16; 16]) };
}

/// Duel configuration options.
#[derive(Debug, Clone)]
pub struct DuelConfig {
    pub seed: [u64; 4],
    pub flags: u64,
    pub team1_lp: u32,
    pub team1_hand: u32,
    pub team1_draw: u32,
    pub team2_lp: u32,
    pub team2_hand: u32,
    pub team2_draw: u32,
    pub enable_unsafe_libraries: bool,
}

impl Default for DuelConfig {
    fn default() -> Self {
        Self {
            seed: [0x1234_5678, 0x9abc_def0, 0x1357_9bdf, 0x2468_ace0],
            flags: DUEL_MODE_MR5,
            team1_lp: 8000,
            team1_hand: 5,
            team1_draw: 1,
            team2_lp: 8000,
            team2_hand: 5,
            team2_draw: 1,
            enable_unsafe_libraries: false,
        }
    }
}

/// The state returned by the simulation after stepping the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessResult {
    /// Duel is waiting for player input.
    Awaiting,
    /// Engine can continue processing immediately.
    Continue,
    /// Duel has reached a conclusion.
    End,
}

/// Heap-allocated context passed through C void* payloads.
struct DuelContext {
    db: Arc<CardDatabase>,
    script_loader: Arc<ScriptLoader>,
}

unsafe extern "C" fn card_reader_callback(
    payload: *mut c_void,
    code: u32,
    data: *mut OCG_CardData,
) {
    if payload.is_null() || data.is_null() {
        return;
    }
    let ctx = unsafe { &*(payload as *const DuelContext) };
    let out = unsafe { &mut *data };

    if let Some(card) = ctx.db.get(code) {
        out.code = card.code;
        out.alias = card.alias;
        out.type_ = card.type_flags;
        out.level = card.level;
        out.attribute = card.attribute;
        out.race = card.race;
        out.attack = card.attack;
        out.defense = card.defense;
        out.lscale = card.lscale;
        out.rscale = card.rscale;
        out.link_marker = card.link_marker;

        SETCODES_BUF.with(|buf| {
            let mut b = buf.borrow_mut();
            let count = card.setcodes.len().min(15);
            for i in 0..count {
                b[i] = card.setcodes[i];
            }
            b[count] = 0;
            out.setcodes = b.as_mut_ptr();
        });
    } else {
        out.code = code;
        out.alias = 0;
        out.setcodes = std::ptr::null_mut();
        out.type_ = 0;
        out.level = 0;
        out.attribute = 0;
        out.race = 0;
        out.attack = 0;
        out.defense = 0;
        out.lscale = 0;
        out.rscale = 0;
        out.link_marker = 0;
    }
}

unsafe extern "C" fn card_reader_done_callback(
    _payload: *mut c_void,
    _data: *mut OCG_CardData,
) {
}

unsafe extern "C" fn script_reader_callback(
    payload: *mut c_void,
    duel: OCG_Duel,
    name: *const c_char,
) -> c_int {
    if payload.is_null() || name.is_null() {
        return 0;
    }
    let ctx = unsafe { &*(payload as *const DuelContext) };
    let c_str = unsafe { CStr::from_ptr(name) };
    let name_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    if let Some(bytes) = ctx.script_loader.read_script(name_str) {
        let ret = unsafe {
            OCG_LoadScript(
                duel,
                bytes.as_ptr() as *const c_char,
                bytes.len() as u32,
                name,
            )
        };
        return ret;
    }

    0
}

unsafe extern "C" fn log_handler_callback(
    _payload: *mut c_void,
    string: *const c_char,
    log_type: c_int,
) {
    if string.is_null() {
        return;
    }
    let msg = match unsafe { CStr::from_ptr(string) }.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    match log_type {
        0 => tracing::error!(target: "ocgcore", "{msg}"),
        1 => tracing::info!(target: "ocgcore_script", "{msg}"),
        2 => tracing::debug!(target: "ocgcore", "{msg}"),
        _ => tracing::trace!(target: "ocgcore", "{msg}"),
    }
}

/// A safe, RAII wrapper over an `OCG_Duel` instance.
pub struct Duel {
    handle: OCG_Duel,
    context: *mut DuelContext,
    is_started: bool,
    is_ended: bool,
}

unsafe impl Send for Duel {}

impl Duel {
    /// Create and initialize a new duel.
    pub fn create(
        config: DuelConfig,
        db: Arc<CardDatabase>,
        script_loader: Arc<ScriptLoader>,
    ) -> Result<Self> {
        let context = Box::new(DuelContext {
            db,
            script_loader: Arc::clone(&script_loader),
        });
        let context_ptr = Box::into_raw(context);

        let options = OCG_DuelOptions {
            seed: config.seed,
            flags: config.flags,
            team1: OCG_Player {
                startingLP: config.team1_lp,
                startingDrawCount: config.team1_hand,
                drawCountPerTurn: config.team1_draw,
            },
            team2: OCG_Player {
                startingLP: config.team2_lp,
                startingDrawCount: config.team2_hand,
                drawCountPerTurn: config.team2_draw,
            },
            cardReader: Some(card_reader_callback),
            payload1: context_ptr as *mut c_void,
            scriptReader: Some(script_reader_callback),
            payload2: context_ptr as *mut c_void,
            logHandler: Some(log_handler_callback),
            payload3: context_ptr as *mut c_void,
            cardReaderDone: Some(card_reader_done_callback),
            payload4: context_ptr as *mut c_void,
            enableUnsafeLibraries: if config.enable_unsafe_libraries { 1 } else { 0 },
        };

        let mut handle: OCG_Duel = std::ptr::null_mut();
        let status = unsafe { OCG_CreateDuel(&mut handle, &options) };
        if status != OCG_DuelCreationStatus::Success as c_int || handle.is_null() {
            unsafe {
                drop(Box::from_raw(context_ptr));
            }
            return Err(Error::DuelCreationFailed(status));
        }

        // Load baseline scripts: constant.lua and utility.lua
        if let Some(bytes) = script_loader.read_script("constant.lua") {
            let c_name = CString::new("constant.lua").unwrap();
            unsafe {
                OCG_LoadScript(
                    handle,
                    bytes.as_ptr() as *const c_char,
                    bytes.len() as u32,
                    c_name.as_ptr(),
                );
            }
        }
        if let Some(bytes) = script_loader.read_script("utility.lua") {
            let c_name = CString::new("utility.lua").unwrap();
            unsafe {
                OCG_LoadScript(
                    handle,
                    bytes.as_ptr() as *const c_char,
                    bytes.len() as u32,
                    c_name.as_ptr(),
                );
            }
        }

        Ok(Duel {
            handle,
            context: context_ptr,
            is_started: false,
            is_ended: false,
        })
    }

    /// Add a new card into the duel before starting.
    pub fn new_card(
        &mut self,
        team: u8,
        code: u32,
        location: u32,
        sequence: u32,
        position: u32,
    ) {
        let info = OCG_NewCardInfo {
            team,
            duelist: 0,
            code,
            con: team,
            loc: location,
            seq: sequence,
            pos: position,
        };
        unsafe {
            OCG_DuelNewCard(self.handle, &info);
        }
    }

    /// Populate Main Deck and Extra Deck for a player.
    pub fn load_deck(&mut self, player: u8, deck: &Deck) {
        for (i, &code) in deck.main.iter().enumerate() {
            self.new_card(
                player,
                code,
                LOCATION_DECK,
                i as u32,
                POS_FACEDOWN_DEFENSE,
            );
        }
        for (i, &code) in deck.extra.iter().enumerate() {
            self.new_card(
                player,
                code,
                LOCATION_EXTRA,
                i as u32,
                POS_FACEDOWN_DEFENSE,
            );
        }
    }

    /// Start the duel simulation state machine.
    pub fn start(&mut self) {
        if !self.is_started {
            self.is_started = true;
            unsafe {
                OCG_StartDuel(self.handle);
            }
        }
    }

    /// Step the duel simulation.
    pub fn process(&mut self) -> Result<ProcessResult> {
        if self.is_ended {
            return Err(Error::DuelAlreadyEnded);
        }
        let status = unsafe { OCG_DuelProcess(self.handle) };
        if status == OCG_DuelStatus::Awaiting as c_int {
            Ok(ProcessResult::Awaiting)
        } else if status == OCG_DuelStatus::Continue as c_int {
            Ok(ProcessResult::Continue)
        } else {
            self.is_ended = true;
            Ok(ProcessResult::End)
        }
    }

    /// Retrieve raw binary messages generated by the last process cycle.
    pub fn get_raw_messages(&mut self) -> Vec<u8> {
        let mut len: u32 = 0;
        let ptr = unsafe { OCG_DuelGetMessage(self.handle, &mut len) };
        if ptr.is_null() || len == 0 {
            return Vec::new();
        }
        let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        slice.to_vec()
    }

    /// Retrieve and decode all messages generated by the last process cycle.
    pub fn get_messages(&mut self) -> Result<Vec<CoreMessage>> {
        let bytes = self.get_raw_messages();
        parse_message_stream(&bytes)
    }

    /// Send a player response to the duel engine.
    pub fn set_response(&mut self, response: &PlayerResponse) {
        let bytes = response.to_bytes();
        unsafe {
            OCG_DuelSetResponse(
                self.handle,
                bytes.as_ptr() as *const c_void,
                bytes.len() as u32,
            );
        }
    }

    /// Check number of cards in a specific zone.
    pub fn query_count(&self, team: u8, location: u32) -> u32 {
        unsafe { OCG_DuelQueryCount(self.handle, team, location) }
    }

    /// Query the complete field state byte buffer.
    pub fn query_field(&self) -> Vec<u8> {
        let mut len: u32 = 0;
        let ptr = unsafe { OCG_DuelQueryField(self.handle, &mut len) };
        if ptr.is_null() || len == 0 {
            return Vec::new();
        }
        let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        slice.to_vec()
    }

    pub fn is_started(&self) -> bool {
        self.is_started
    }

    pub fn is_ended(&self) -> bool {
        self.is_ended
    }
}

impl Drop for Duel {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                OCG_DestroyDuel(self.handle);
            }
        }
        if !self.context.is_null() {
            unsafe {
                drop(Box::from_raw(self.context));
            }
        }
    }
}
