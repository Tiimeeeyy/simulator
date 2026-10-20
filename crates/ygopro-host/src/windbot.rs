//! Local TCP loopback adapter for WindBot integration.
//!
//! Exposes a standard YGOPro protocol socket (`127.0.0.1:port`) allowing external
//! WindBot instances (C# / Python / C++) to connect as Player 1 or 2.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use crate::error::{HostError, Result};

// Protocol Message Codes
pub const STOC_GAME_MSG: u8 = 0x01;
pub const STOC_JOIN_GAME: u8 = 0x20;
pub const STOC_DUEL_START: u8 = 0x21;
pub const STOC_TYPE_CHANGE: u8 = 0x22;

pub const CTOS_RESPONSE: u8 = 0x01;
pub const CTOS_PLAYER_INFO: u8 = 0x10;

/// Network adapter for an external WindBot process connected via loopback TCP.
pub struct WindBotBridge {
    stream: TcpStream,
}

impl WindBotBridge {
    /// Bind a TCP listener on `127.0.0.1:port` (use `0` for ephemeral port).
    pub fn bind(addr: &str) -> std::io::Result<TcpListener> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(false)?;
        Ok(listener)
    }

    /// Accept a WindBot client connection and perform the initial handshake.
    pub fn accept(listener: &TcpListener) -> Result<Self> {
        let (mut stream, _) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        stream.set_nodelay(true)?;

        // 1. Read CTOS_PLAYER_INFO (or whatever initial packet bot sends)
        let mut header = [0u8; 2];
        if stream.read_exact(&mut header).is_ok() {
            let len = u16::from_le_bytes(header) as usize;
            if len > 0 {
                let mut buf = vec![0u8; len];
                let _ = stream.read_exact(&mut buf);
            }
        }

        // 2. Send STOC_JOIN_GAME
        let join_payload = [0u8; 18];
        Self::write_packet(&mut stream, STOC_JOIN_GAME, &join_payload)?;

        // 3. Send STOC_TYPE_CHANGE (host = 0x01, player1 = 0x01)
        let type_payload = [0x01u8];
        Self::write_packet(&mut stream, STOC_TYPE_CHANGE, &type_payload)?;

        // 4. Send STOC_DUEL_START
        Self::write_packet(&mut stream, STOC_DUEL_START, &[])?;

        Ok(Self { stream })
    }

    /// Helper to write a framed packet: `[u16 length][u8 proto_code][payload]`
    fn write_packet(stream: &mut TcpStream, proto: u8, payload: &[u8]) -> std::io::Result<()> {
        let total_len = (1 + payload.len()) as u16;
        stream.write_all(&total_len.to_le_bytes())?;
        stream.write_all(&[proto])?;
        if !payload.is_empty() {
            stream.write_all(payload)?;
        }
        stream.flush()?;
        Ok(())
    }

    /// Broadcast game messages buffer to the connected WindBot.
    pub fn send_game_messages(&mut self, raw_messages: &[u8]) -> Result<()> {
        if raw_messages.is_empty() {
            return Ok(());
        }
        Self::write_packet(&mut self.stream, STOC_GAME_MSG, raw_messages)
            .map_err(HostError::Io)?;
        Ok(())
    }

    /// Read player response sent from WindBot (`CTOS_RESPONSE`).
    pub fn read_response(&mut self) -> Result<Vec<u8>> {
        let mut header = [0u8; 2];
        self.stream.read_exact(&mut header).map_err(HostError::Io)?;
        let len = u16::from_le_bytes(header) as usize;
        if len == 0 {
            return Ok(Vec::new());
        }

        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf).map_err(HostError::Io)?;

        // First byte is proto code (CTOS_RESPONSE = 0x01)
        if !buf.is_empty() && buf[0] == CTOS_RESPONSE {
            Ok(buf[1..].to_vec())
        } else {
            Ok(buf)
        }
    }
}
