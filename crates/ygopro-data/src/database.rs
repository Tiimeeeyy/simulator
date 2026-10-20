//! In-memory card database loaded from one or more `cards.cdb` files.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use crate::card::Card;
use crate::constants::*;
use crate::error::{Error, Result};

/// Normalise a card name for matching: lowercase, typographic quotes/dashes
/// folded, and everything that is not alphanumeric removed.
///
/// `"Droll & Lock Bird"` → `"drolllockbird"`, `"Kaleido-Heart"` → `"kaleidoheart"`.
pub fn normalize_name(name: &str) -> String {
    name.chars()
        .flat_map(char::to_lowercase)
        .filter(|c| c.is_alphanumeric())
        .collect()
}

// ---------------------------------------------------------------------------
// Bitset index
// ---------------------------------------------------------------------------

/// Dense bitset over card indices (14k cards ⇒ ~230 words per set).
#[derive(Debug, Clone, Default)]
pub(crate) struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    fn with_len(len: usize) -> Self {
        BitSet { words: vec![0; len.div_ceil(64)] }
    }
    fn full(len: usize) -> Self {
        let mut s = Self::with_len(len);
        for i in 0..len {
            s.insert(i);
        }
        s
    }
    fn insert(&mut self, i: usize) {
        self.words[i / 64] |= 1 << (i % 64);
    }
    fn and_assign(&mut self, other: &BitSet) {
        self.words.iter_mut().zip(&other.words).for_each(|(a, b)| *a &= b);
    }
    fn and_not_assign(&mut self, other: &BitSet) {
        self.words.iter_mut().zip(&other.words).for_each(|(a, b)| *a &= !b);
    }
    fn or_assign(&mut self, other: &BitSet) {
        self.words.iter_mut().zip(&other.words).for_each(|(a, b)| *a |= b);
    }
    fn iter_ones(&self) -> impl Iterator<Item = usize> + '_ {
        self.words.iter().enumerate().flat_map(|(wi, &w)| {
            let mut w = w;
            std::iter::from_fn(move || {
                if w == 0 {
                    return None;
                }
                let tz = w.trailing_zeros() as usize;
                w &= w - 1;
                Some(wi * 64 + tz)
            })
        })
    }
}

/// Precomputed per-bit indices so filters are a handful of word-wise ANDs.
#[derive(Debug, Default)]
struct CardIndex {
    len: usize,
    by_type: Vec<BitSet>,      // 32 type bits
    by_attribute: Vec<BitSet>, // 8 attribute bits
    by_race: Vec<BitSet>,      // 32 race bits
    by_scope: Vec<BitSet>,     // 16 scope bits
    alternate_art: BitSet,
}

impl CardIndex {
    fn build(cards: &[Card], is_alt: &[bool]) -> Self {
        let len = cards.len();
        let mk = |n: usize| (0..n).map(|_| BitSet::with_len(len)).collect::<Vec<_>>();
        let mut idx = CardIndex {
            len,
            by_type: mk(32),
            by_attribute: mk(8),
            by_race: mk(32),
            by_scope: mk(16),
            alternate_art: BitSet::with_len(len),
        };
        for (i, c) in cards.iter().enumerate() {
            for b in 0..32 {
                if c.type_flags >> b & 1 == 1 {
                    idx.by_type[b].insert(i);
                }
                if c.race >> b & 1 == 1 {
                    idx.by_race[b].insert(i);
                }
            }
            for b in 0..8 {
                if c.attribute >> b & 1 == 1 {
                    idx.by_attribute[b].insert(i);
                }
            }
            for b in 0..16 {
                if c.ot >> b & 1 == 1 {
                    idx.by_scope[b].insert(i);
                }
            }
            if is_alt[i] {
                idx.alternate_art.insert(i);
            }
        }
        idx
    }

    fn any_of(&self, sets: &[BitSet], mask: u64) -> BitSet {
        let mut out = BitSet::with_len(self.len);
        for (b, set) in sets.iter().enumerate() {
            if mask >> b & 1 == 1 {
                out.or_assign(set);
            }
        }
        out
    }
}

/// Declarative card filter evaluated against the bitset indices.
///
/// Zero masks mean "don't care".
#[derive(Debug, Clone, Default)]
pub struct CardFilter {
    /// Card must have **all** of these type bits.
    pub type_all: u32,
    /// Card must have **at least one** of these type bits.
    pub type_any: u32,
    /// Card must have **none** of these type bits.
    pub type_none: u32,
    pub attribute_any: u32,
    pub race_any: u64,
    /// Scope (`OT_*`) bits, at least one must match.
    pub scope_any: u32,
    /// Include alternate-artwork duplicates (default: hidden).
    pub include_alternate_arts: bool,
    /// Case/punctuation-insensitive substring match on the name.
    pub name_contains: Option<String>,
}

/// Result of a fuzzy name lookup.
#[derive(Debug, Clone, PartialEq)]
pub struct FuzzyMatch {
    pub code: u32,
    pub score: f64,
    /// Second-best distinct card, useful to flag ambiguous matches.
    pub runner_up: Option<(u32, f64)>,
}

// ---------------------------------------------------------------------------
// Database
// ---------------------------------------------------------------------------

/// All cards, resident in memory, sorted by passcode.
#[derive(Debug, Default)]
pub struct CardDatabase {
    cards: Vec<Card>,
    by_code: HashMap<u32, u32>,
    /// normalised name → card indices, best candidate first.
    by_name: HashMap<String, Vec<u32>>,
    /// (normalised name, best index) for every distinct deckable name; fuzzy search space.
    name_keys: Vec<(String, u32)>,
    index: CardIndex,
}

const SELECT_ALL: &str = "SELECT d.id, d.ot, d.alias, d.setcode, d.type, d.atk, d.def, d.level, \
     d.race, d.attribute, d.category, t.name, t.desc, \
     t.str1, t.str2, t.str3, t.str4, t.str5, t.str6, t.str7, t.str8, \
     t.str9, t.str10, t.str11, t.str12, t.str13, t.str14, t.str15, t.str16 \
     FROM datas d JOIN texts t ON t.id = d.id";

impl CardDatabase {
    /// Load a single `cards.cdb`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_many([path])
    }

    /// Load several databases; rows in later files override earlier ones
    /// (e.g. `cards.cdb` then `prerelease-*.cdb`).
    pub fn open_many<P: AsRef<Path>>(paths: impl IntoIterator<Item = P>) -> Result<Self> {
        let mut rows: HashMap<u32, Card> = HashMap::new();
        for path in paths {
            let path = path.as_ref();
            if !path.is_file() {
                return Err(Error::io(
                    path,
                    std::io::Error::new(std::io::ErrorKind::NotFound, "card database not found"),
                ));
            }
            let conn = Connection::open_with_flags(
                path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )?;
            for card in Self::read_rows(&conn)? {
                rows.insert(card.code, card);
            }
            tracing::debug!(path = %path.display(), total = rows.len(), "loaded card database");
        }
        Ok(Self::from_cards(rows.into_values()))
    }

    fn read_rows(conn: &Connection) -> Result<Vec<Card>> {
        let mut stmt = conn.prepare(SELECT_ALL)?;
        let it = stmt.query_map([], |r| {
            let int = |i: usize| -> rusqlite::Result<i64> { Ok(r.get::<_, Option<i64>>(i)?.unwrap_or(0)) };
            let text = |i: usize| -> rusqlite::Result<String> {
                Ok(r.get::<_, Option<String>>(i)?.unwrap_or_default())
            };
            let type_flags = int(4)? as u32;
            let (level, lscale, rscale) = Card::unpack_level(int(7)? as u32);
            let raw_def = int(6)? as i32;
            let is_link = type_flags & TYPE_LINK != 0;
            let mut strings = (13..29).map(text).collect::<rusqlite::Result<Vec<_>>>()?;
            while strings.last().is_some_and(String::is_empty) {
                strings.pop();
            }
            Ok(Card {
                code: int(0)? as u32,
                ot: int(1)? as u32,
                alias: int(2)? as u32,
                setcodes: Card::unpack_setcodes(int(3)? as u64),
                type_flags,
                attack: int(5)? as i32,
                defense: if is_link { 0 } else { raw_def },
                link_marker: if is_link { raw_def as u32 } else { 0 },
                level,
                lscale,
                rscale,
                race: int(8)? as u64,
                attribute: int(9)? as u32,
                category: int(10)? as u64,
                name: text(11)?,
                desc: text(12)?,
                strings,
            })
        })?;
        Ok(it.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Build a database from already-decoded cards (tests, custom sources).
    pub fn from_cards(cards: impl IntoIterator<Item = Card>) -> Self {
        let mut cards: Vec<Card> = cards.into_iter().collect();
        cards.sort_unstable_by_key(|c| c.code);
        let by_code: HashMap<u32, u32> =
            cards.iter().enumerate().map(|(i, c)| (c.code, i as u32)).collect();

        let mut by_name: HashMap<String, Vec<u32>> = HashMap::new();
        for (i, c) in cards.iter().enumerate() {
            by_name.entry(normalize_name(&c.name)).or_default().push(i as u32);
        }
        // Best candidate first: deckable, official, base artwork, lowest code.
        for list in by_name.values_mut() {
            list.sort_by_key(|&i| {
                let c = &cards[i as usize];
                (!c.is_deckable(), c.is_rush(), !c.is_official(), c.alias != 0, c.code)
            });
        }
        let mut name_keys: Vec<(String, u32)> = by_name
            .iter()
            .filter(|(k, v)| !k.is_empty() && cards[v[0] as usize].is_deckable())
            .map(|(k, v)| (k.clone(), v[0]))
            .collect();
        name_keys.sort();

        let mut db = CardDatabase { cards, by_code, by_name, name_keys, index: CardIndex::default() };
        let alt: Vec<bool> = db.cards.iter().map(|c| db.is_alternate_art(c)).collect();
        db.index = CardIndex::build(&db.cards, &alt);
        db
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &Card> {
        self.cards.iter()
    }
    pub fn get(&self, code: u32) -> Option<&Card> {
        self.by_code.get(&code).map(|&i| &self.cards[i as usize])
    }
    /// Name for display; falls back to the passcode.
    pub fn name_of(&self, code: u32) -> String {
        self.get(code).map_or_else(|| code.to_string(), |c| c.name.clone())
    }

    /// `true` if `card` is an alternate artwork of another passcode.
    ///
    /// Mirrors ygopro's deck-check rule (`alias` within 10 of the passcode) and
    /// additionally accepts aliases that share the exact same name. Cards that
    /// are merely "treated as" another card (different name, distant alias,
    /// e.g. *Harpie Lady 1*) are **not** alternate arts.
    pub fn is_alternate_art(&self, card: &Card) -> bool {
        card.alias != 0
            && (card.alias.abs_diff(card.code) <= 10
                || self.get(card.alias).is_some_and(|b| b.name == card.name))
    }

    /// Resolve alternate-art passcodes to their base passcode. Copy limits are
    /// always counted on the canonical code.
    pub fn canonical_code(&self, code: u32) -> u32 {
        let mut cur = code;
        for _ in 0..4 {
            match self.get(cur) {
                Some(c) if self.is_alternate_art(c) => cur = c.alias,
                _ => break,
            }
        }
        cur
    }

    /// Exact (normalised, case/punctuation-insensitive) name lookup.
    pub fn find_by_name(&self, name: &str) -> Option<&Card> {
        self.by_name
            .get(&normalize_name(name))
            .and_then(|v| v.first())
            .map(|&i| &self.cards[i as usize])
    }

    /// Fuzzy name lookup over all deckable names. `threshold` is in `0..=1`.
    pub fn fuzzy_find(&self, name: &str, threshold: f64) -> Option<FuzzyMatch> {
        let key = normalize_name(name);
        if key.is_empty() {
            return None;
        }
        let mut best: Option<(f64, u32)> = None;
        let mut second: Option<(f64, u32)> = None;
        for (cand, idx) in &self.name_keys {
            let score = 0.5 * strsim::jaro_winkler(&key, cand)
                + 0.5 * strsim::normalized_damerau_levenshtein(&key, cand);
            if best.is_none_or(|(s, _)| score > s) {
                second = best;
                best = Some((score, *idx));
            } else if second.is_none_or(|(s, _)| score > s) {
                second = Some((score, *idx));
            }
        }
        let (score, idx) = best?;
        (score >= threshold).then(|| FuzzyMatch {
            code: self.cards[idx as usize].code,
            score,
            runner_up: second.map(|(s, i)| (self.cards[i as usize].code, s)),
        })
    }

    /// Evaluate a [`CardFilter`] using the precomputed bitset indices.
    pub fn query(&self, f: &CardFilter) -> Vec<&Card> {
        let ix = &self.index;
        let mut set = BitSet::full(ix.len);
        for b in 0..32 {
            if f.type_all >> b & 1 == 1 {
                set.and_assign(&ix.by_type[b]);
            }
        }
        if f.type_any != 0 {
            set.and_assign(&ix.any_of(&ix.by_type, f.type_any as u64));
        }
        if f.type_none != 0 {
            set.and_not_assign(&ix.any_of(&ix.by_type, f.type_none as u64));
        }
        if f.attribute_any != 0 {
            set.and_assign(&ix.any_of(&ix.by_attribute, f.attribute_any as u64));
        }
        if f.race_any != 0 {
            set.and_assign(&ix.any_of(&ix.by_race, f.race_any));
        }
        if f.scope_any != 0 {
            set.and_assign(&ix.any_of(&ix.by_scope, f.scope_any as u64));
        }
        if !f.include_alternate_arts {
            set.and_not_assign(&ix.alternate_art);
        }
        let needle = f.name_contains.as_deref().map(normalize_name).filter(|n| !n.is_empty());
        set.iter_ones()
            .map(|i| &self.cards[i])
            .filter(|c| needle.as_ref().is_none_or(|n| normalize_name(&c.name).contains(n.as_str())))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CardDatabase {
        let mut alt = Card::new(81439174, "Foolish Burial", TYPE_SPELL);
        alt.alias = 81439173;
        let mut treated = Card::new(91932350, "Harpie Lady 1", TYPE_MONSTER | TYPE_EFFECT);
        treated.alias = 76812113;
        let mut fusion = Card::new(84330567, "Tearlaments Rulkallos", TYPE_MONSTER | TYPE_FUSION);
        fusion.attribute = ATTRIBUTE_WATER;
        fusion.race = RACE_AQUA;
        let mut token = Card::new(99999999, "Foolish Token", TYPE_MONSTER | TYPE_TOKEN);
        token.ot = OT_OCG_TCG;
        CardDatabase::from_cards([
            Card::new(81439173, "Foolish Burial", TYPE_SPELL),
            alt,
            Card::new(76812113, "Harpie Lady", TYPE_MONSTER | TYPE_NORMAL),
            treated,
            fusion,
            Card::new(94145021, "Droll & Lock Bird", TYPE_MONSTER | TYPE_EFFECT),
            token,
        ])
    }

    #[test]
    fn normalization() {
        assert_eq!(normalize_name("Droll & Lock Bird"), "drolllockbird");
        assert_eq!(normalize_name("Tearlaments Kaleido-Heart"), "tearlamentskaleidoheart");
        assert_eq!(normalize_name("Maxx \u{201c}C\u{201d}"), "maxxc");
    }

    #[test]
    fn alias_resolution() {
        let db = sample();
        assert_eq!(db.canonical_code(81439174), 81439173, "alt art collapses");
        assert_eq!(db.canonical_code(91932350), 91932350, "'treated as' cards stay distinct");
        assert_eq!(db.canonical_code(12345), 12345, "unknown codes pass through");
    }

    #[test]
    fn name_lookup_prefers_base_art() {
        let db = sample();
        assert_eq!(db.find_by_name("FOOLISH  burial").unwrap().code, 81439173);
        assert_eq!(db.find_by_name("droll and lock bird"), None);
        let m = db.fuzzy_find("droll and lock bird", 0.8).unwrap();
        assert_eq!(m.code, 94145021);
        assert!(db.fuzzy_find("zzzzzz", 0.8).is_none());
    }

    #[test]
    fn bitset_queries() {
        let db = sample();
        let extra = db.query(&CardFilter { type_any: TYPE_EXTRA_DECK, ..Default::default() });
        assert_eq!(extra.iter().map(|c| c.code).collect::<Vec<_>>(), vec![84330567]);
        let spells = db.query(&CardFilter { type_all: TYPE_SPELL, ..Default::default() });
        assert_eq!(spells.len(), 1, "alternate art hidden by default");
        let spells = db.query(&CardFilter {
            type_all: TYPE_SPELL,
            include_alternate_arts: true,
            ..Default::default()
        });
        assert_eq!(spells.len(), 2);
        let named = db.query(&CardFilter {
            name_contains: Some("lock-bird".into()),
            ..Default::default()
        });
        assert_eq!(named[0].code, 94145021);
        let non_tokens = db.query(&CardFilter {
            type_all: TYPE_MONSTER,
            type_none: TYPE_TOKEN,
            race_any: RACE_AQUA,
            ..Default::default()
        });
        assert_eq!(non_tokens.len(), 1);
    }
}
