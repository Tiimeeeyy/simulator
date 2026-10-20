//! `strings.conf` parser (system strings, victory reasons, counters, setnames).
//!
//! ```text
//! !system 1050 Monster
//! !counter 0x1 Spell Counter
//! !setname 0x182 Tearlaments
//! ```

use std::collections::HashMap;
use std::path::Path;

use crate::error::{Error, Result};

#[derive(Debug, Default, Clone)]
pub struct StringsConf {
    pub system: HashMap<u32, String>,
    pub victory: HashMap<u32, String>,
    pub counter: HashMap<u32, String>,
    /// Raw setname strings; EDOPro may append alternate spellings after `|`.
    pub setname: HashMap<u16, String>,
}

fn parse_num(s: &str) -> Option<u32> {
    match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(hex) => u32::from_str_radix(hex, 16).ok(),
        None => s.parse().ok(),
    }
}

impl StringsConf {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        Ok(Self::parse(&text))
    }

    /// Parse and merge into a fresh table. Unknown directives are ignored;
    /// later duplicates override earlier ones (locale overlays).
    pub fn parse(text: &str) -> Self {
        let mut s = StringsConf::default();
        s.merge_str(text);
        s
    }

    pub fn merge_str(&mut self, text: &str) {
        for line in text.lines() {
            let Some(rest) = line.trim_end_matches('\r').strip_prefix('!') else { continue };
            let mut parts = rest.splitn(3, ' ');
            let (Some(kind), Some(id), Some(value)) = (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let Some(id) = parse_num(id) else { continue };
            let value = value.trim().to_string();
            match kind {
                "system" => {
                    self.system.insert(id, value);
                }
                "victory" => {
                    self.victory.insert(id, value);
                }
                "counter" => {
                    self.counter.insert(id, value);
                }
                "setname" => {
                    self.setname.insert(id as u16, value);
                }
                _ => {}
            }
        }
    }

    /// Primary archetype name (text before the first `|`).
    pub fn setname(&self, code: u16) -> Option<&str> {
        self.setname.get(&code).map(|s| s.split('|').next().unwrap_or(s).trim())
    }

    pub fn system(&self, id: u32) -> Option<&str> {
        self.system.get(&id).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_sections() {
        let s = StringsConf::parse(
            "# comment\n!system 1050 Monster\r\n!victory 0x1 Exodia\n!counter 0x1 Spell Counter\n\
             !setname 0x46 Polymerization|Fusion\n!setname 0x182 Tearlaments\n!bogus 1 x\n",
        );
        assert_eq!(s.system(1050), Some("Monster"));
        assert_eq!(s.victory[&1], "Exodia");
        assert_eq!(s.counter[&1], "Spell Counter");
        assert_eq!(s.setname(0x46), Some("Polymerization"));
        assert_eq!(s.setname(0x182), Some("Tearlaments"));
    }
}
