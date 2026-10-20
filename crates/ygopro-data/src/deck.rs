//! Deck representation, .ydk file parser/serializer, and `ydke://` URL support.

use std::collections::HashMap;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::engine::general_purpose::URL_SAFE as BASE64_URL_SAFE;

use crate::database::CardDatabase;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeckSection {
    Main,
    Extra,
    Side,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Deck {
    pub name: String,
    pub main: Vec<u32>,
    pub extra: Vec<u32>,
    pub side: Vec<u32>,
}

impl Deck {
    pub fn new(name: impl Into<String>) -> Self {
        Deck {
            name: name.into(),
            main: Vec::new(),
            extra: Vec::new(),
            side: Vec::new(),
        }
    }

    pub fn total_count(&self) -> usize {
        self.main.len() + self.extra.len() + self.side.len()
    }

    pub fn is_empty(&self) -> bool {
        self.main.is_empty() && self.extra.is_empty() && self.side.is_empty()
    }

    pub fn section(&self, section: DeckSection) -> &[u32] {
        match section {
            DeckSection::Main => &self.main,
            DeckSection::Extra => &self.extra,
            DeckSection::Side => &self.side,
        }
    }

    pub fn section_mut(&mut self, section: DeckSection) -> &mut Vec<u32> {
        match section {
            DeckSection::Main => &mut self.main,
            DeckSection::Extra => &mut self.extra,
            DeckSection::Side => &mut self.side,
        }
    }

    pub fn all_cards(&self) -> impl Iterator<Item = u32> + '_ {
        self.main.iter().chain(&self.extra).chain(&self.side).copied()
    }

    /// Count copies of each canonical card code across Main, Extra, and Side.
    pub fn canonical_counts(&self, db: &CardDatabase) -> HashMap<u32, usize> {
        let mut counts = HashMap::new();
        for code in self.all_cards() {
            let canonical = db.canonical_code(code);
            *counts.entry(canonical).or_insert(0) += 1;
        }
        counts
    }

    /// Count exact passcode copies in a given section.
    pub fn count_in_section(&self, section: DeckSection, code: u32) -> usize {
        self.section(section).iter().filter(|&&c| c == code).count()
    }

    /// Parse a .ydk file format string.
    pub fn parse_ydk(text: &str) -> Result<Self> {
        let mut deck = Deck::default();
        let mut current_section = DeckSection::Main;

        for (line_idx, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('#') {
                if line.eq_ignore_ascii_case("#main") {
                    current_section = DeckSection::Main;
                } else if line.eq_ignore_ascii_case("#extra") {
                    current_section = DeckSection::Extra;
                }
                // Comments like #created by... are ignored
                continue;
            }
            if line.starts_with('!') {
                if line.eq_ignore_ascii_case("!side") {
                    current_section = DeckSection::Side;
                }
                continue;
            }

            // Expect integer passcode
            match line.parse::<u32>() {
                Ok(code) => {
                    deck.section_mut(current_section).push(code);
                }
                Err(_) => {
                    return Err(Error::Parse {
                        line: line_idx + 1,
                        message: format!("invalid passcode in ydk: '{line}'"),
                    });
                }
            }
        }

        Ok(deck)
    }

    pub fn load_ydk(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        let mut deck = Self::parse_ydk(&content)?;
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            deck.name = stem.to_string();
        }
        Ok(deck)
    }

    /// Serialize to standard .ydk format.
    pub fn to_ydk(&self) -> String {
        let mut out = String::new();
        out.push_str("#created by ygopro\n");
        out.push_str("#main\n");
        for &code in &self.main {
            out.push_str(&format!("{code}\n"));
        }
        out.push_str("#extra\n");
        for &code in &self.extra {
            out.push_str(&format!("{code}\n"));
        }
        out.push_str("!side\n");
        for &code in &self.side {
            out.push_str(&format!("{code}\n"));
        }
        out
    }

    pub fn save_ydk(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        std::fs::write(path, self.to_ydk()).map_err(|e| Error::io(path, e))
    }

    /// Encode deck to `ydke://` URL string.
    pub fn to_ydke(&self) -> String {
        fn encode_section(cards: &[u32]) -> String {
            let mut bytes = Vec::with_capacity(cards.len() * 4);
            for &c in cards {
                bytes.extend_from_slice(&c.to_le_bytes());
            }
            BASE64_STANDARD.encode(&bytes)
        }

        format!(
            "ydke://{}!{}!{}!",
            encode_section(&self.main),
            encode_section(&self.extra),
            encode_section(&self.side)
        )
    }

    /// Parse a `ydke://` URL string.
    pub fn from_ydke(url: &str) -> Result<Self> {
        let stripped = url
            .trim()
            .strip_prefix("ydke://")
            .ok_or_else(|| Error::InvalidYdke("missing 'ydke://' prefix".into()))?;

        let mut parts = stripped.split('!');
        let main_b64 = parts.next().unwrap_or("");
        let extra_b64 = parts.next().unwrap_or("");
        let side_b64 = parts.next().unwrap_or("");

        fn decode_section(b64: &str) -> Result<Vec<u32>> {
            if b64.is_empty() {
                return Ok(Vec::new());
            }
            let bytes = BASE64_STANDARD
                .decode(b64)
                .or_else(|_| BASE64_URL_SAFE.decode(b64))
                .map_err(|e| Error::InvalidYdke(format!("base64 decode error: {e}")))?;

            if bytes.len() % 4 != 0 {
                return Err(Error::InvalidYdke("invalid byte length for u32 codes".into()));
            }

            let mut codes = Vec::with_capacity(bytes.len() / 4);
            for chunk in bytes.chunks_exact(4) {
                let code = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                codes.push(code);
            }
            Ok(codes)
        }

        Ok(Deck {
            name: "YDKE Deck".into(),
            main: decode_section(main_b64)?,
            extra: decode_section(extra_b64)?,
            side: decode_section(side_b64)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ydk_roundtrip() {
        let mut d = Deck::new("Test");
        d.main = vec![73956664, 4928565];
        d.extra = vec![84330567];
        d.side = vec![102380];

        let ydk = d.to_ydk();
        let parsed = Deck::parse_ydk(&ydk).unwrap();
        assert_eq!(parsed.main, d.main);
        assert_eq!(parsed.extra, d.extra);
        assert_eq!(parsed.side, d.side);
    }

    #[test]
    fn ydke_roundtrip() {
        let mut d = Deck::new("Test");
        d.main = vec![73956664, 4928565];
        d.extra = vec![84330567];
        d.side = vec![102380];

        let url = d.to_ydke();
        assert!(url.starts_with("ydke://"));
        let parsed = Deck::from_ydke(&url).unwrap();
        assert_eq!(parsed.main, d.main);
        assert_eq!(parsed.extra, d.extra);
        assert_eq!(parsed.side, d.side);
    }
}
