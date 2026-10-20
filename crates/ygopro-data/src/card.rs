//! A single card row (`datas` ⨝ `texts`).

use crate::constants::*;

/// Fully decoded card record. Fields mirror `cards.cdb` with the packed
/// columns (`level`, `setcode`, link markers) already unpacked.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Card {
    pub code: u32,
    /// Non-zero for alternate artworks (alias == base passcode) **and** for
    /// cards that are "always treated as" another card. See
    /// [`CardDatabase::canonical_code`](crate::CardDatabase::canonical_code).
    pub alias: u32,
    /// Up to four 16-bit archetype codes (zeros removed).
    pub setcodes: Vec<u16>,
    /// Scope flags (`OT_*`).
    pub ot: u32,
    /// Type flags (`TYPE_*`).
    pub type_flags: u32,
    /// Level / Rank / Link rating.
    pub level: u32,
    pub lscale: u32,
    pub rscale: u32,
    /// `-2` encodes "?".
    pub attack: i32,
    /// `-2` encodes "?". Zero for Link monsters (their DEF column stores markers).
    pub defense: i32,
    pub link_marker: u32,
    pub race: u64,
    pub attribute: u32,
    pub category: u64,
    pub name: String,
    pub desc: String,
    /// `str1`..`str16` (effect prompt strings), trailing empties trimmed.
    pub strings: Vec<String>,
}

impl Card {
    /// Minimal constructor, handy for tests and synthetic cards.
    pub fn new(code: u32, name: impl Into<String>, type_flags: u32) -> Self {
        Card {
            code,
            name: name.into(),
            type_flags,
            ot: OT_OCG_TCG,
            ..Default::default()
        }
    }

    /// Decode the packed `level` column: low byte = level, byte 2 = right
    /// scale, byte 3 = left scale.
    pub fn unpack_level(raw: u32) -> (u32, u32, u32) {
        (raw & 0xff, (raw >> 24) & 0xff, (raw >> 16) & 0xff)
    }

    /// Decode the packed `setcode` column into up to four archetype codes.
    pub fn unpack_setcodes(raw: u64) -> Vec<u16> {
        (0..4)
            .map(|i| ((raw >> (i * 16)) & 0xffff) as u16)
            .filter(|&s| s != 0)
            .collect()
    }

    #[inline]
    pub fn is_type(&self, mask: u32) -> bool {
        self.type_flags & mask != 0
    }
    #[inline]
    pub fn is_monster(&self) -> bool {
        self.is_type(TYPE_MONSTER)
    }
    #[inline]
    pub fn is_spell(&self) -> bool {
        self.is_type(TYPE_SPELL)
    }
    #[inline]
    pub fn is_trap(&self) -> bool {
        self.is_type(TYPE_TRAP)
    }
    /// Fusion / Synchro / Xyz / Link monster.
    #[inline]
    pub fn is_extra_deck_monster(&self) -> bool {
        self.is_monster() && self.is_type(TYPE_EXTRA_DECK)
    }
    /// Tokens, Skills and Action cards can never be put in a deck.
    #[inline]
    pub fn is_deckable(&self) -> bool {
        !self.is_type(TYPE_TOKEN | TYPE_SKILL | TYPE_ACTION)
    }
    #[inline]
    pub fn is_official(&self) -> bool {
        self.ot & OT_OCG_TCG != 0
    }
    #[inline]
    pub fn is_rush(&self) -> bool {
        self.ot & OT_RUSH != 0
    }

    /// ocgcore `is_setcode` semantics: the low 12 bits select the archetype,
    /// the high 4 bits select a sub-archetype that must also be present.
    pub fn is_setcode(&self, set: u16) -> bool {
        self.setcodes
            .iter()
            .any(|&s| (s & 0x0fff) == (set & 0x0fff) && (s & set) == set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack_packed_columns() {
        // Tearlaments Kashtira: setcode 0x18a0182 => Tearlaments (0x182) + Kashtira (0x18a)
        assert_eq!(Card::unpack_setcodes(0x18a0182), vec![0x182, 0x18a]);
        // Pendulum: L scale 4, R scale 4, level 7
        assert_eq!(Card::unpack_level(0x0404_0007), (7, 4, 4));
    }

    #[test]
    fn setcode_semantics() {
        let mut c = Card::new(1, "x", TYPE_MONSTER);
        c.setcodes = vec![0x1182];
        assert!(c.is_setcode(0x182)); // base archetype matches sub-archetype card
        assert!(c.is_setcode(0x1182));
        c.setcodes = vec![0x182];
        assert!(!c.is_setcode(0x1182)); // base card is not a member of the sub-archetype
    }
}
