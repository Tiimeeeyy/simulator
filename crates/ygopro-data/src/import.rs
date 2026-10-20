//! Smart plain-text deck importer.
//!
//! Parses decklists pasted from text, websites, or Discord:
//! - Strips arbitrary headers (`== MONSTER CARDS ==`, `Monsters:`, etc.)
//! - Parses `<count> <card_name>` or `<card_name> x<count>`
//! - Uses case-insensitive exact matching with fuzzy fallback against `cards.cdb`
//! - Auto-partitions Extra Deck monsters (`TYPE_FUSION`, `TYPE_SYNCHRO`, `TYPE_XYZ`, `TYPE_LINK`)
//! - Deterministically splits overflow into Side Deck if non-Extra cards exceed 40.

use crate::card::Card;
use crate::database::CardDatabase;
use crate::deck::Deck;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitStrategy {
    /// If non-extra cards exceed `target_main_size` (or 60 max), split overflow into Side Deck.
    #[default]
    Auto,
    /// Strictly keep up to `N` cards in Main Deck, rest to Side Deck.
    TargetMain(usize),
    /// Never split automatically; put all non-extra cards into Main Deck unless explicit side header.
    Never,
}

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub fuzzy_threshold: f64,
    pub target_main_size: usize,
    pub split_strategy: SplitStrategy,
}

impl Default for ImportOptions {
    fn default() -> Self {
        ImportOptions {
            fuzzy_threshold: 0.78,
            target_main_size: 40,
            split_strategy: SplitStrategy::Auto,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ImportedDeck {
    pub deck: Deck,
    pub warnings: Vec<String>,
    pub unresolved: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SectionContext {
    Unknown,
    Main,
    Extra,
    Side,
}

/// Parse a raw text line into `(count, card_name_candidate)`.
fn parse_line_count_and_name(raw: &str) -> Option<(usize, &str)> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }

    // Strip leading bullet characters: '*', '-', '•', '+'
    let s = s.trim_start_matches(|c| c == '*' || c == '-' || c == '•' || c == '+').trim();
    if s.is_empty() {
        return None;
    }

    // Check for leading count: "3x Card", "3 Card", "3* Card"
    let mut split_idx = None;
    let mut count_val = None;
    for (i, c) in s.char_indices() {
        if !c.is_ascii_digit() {
            if i > 0 {
                if let Ok(n) = s[..i].parse::<usize>() {
                    let rest = s[i..].trim_start();
                    let rest = rest.strip_prefix(|ch| ch == 'x' || ch == 'X' || ch == '*' || ch == ':')
                        .unwrap_or(rest)
                        .trim_start();
                    if !rest.is_empty() {
                        split_idx = Some(rest);
                        count_val = Some(n);
                    }
                }
            }
            break;
        }
    }

    if let (Some(name), Some(cnt)) = (split_idx, count_val) {
        return Some((cnt.max(1), name));
    }

    // Check for trailing count: "Card x3", "Card * 3", "Card 3"
    // Find last 'x', 'X', '*', or whitespace before trailing digits
    if let Some(last_space) = s.rfind(|c: char| c == 'x' || c == 'X' || c == '*' || c.is_whitespace()) {
        let after = s[last_space..].trim_start_matches(|c| c == 'x' || c == 'X' || c == '*').trim();
        if let Ok(n) = after.parse::<usize>() {
            let before = s[..last_space].trim();
            if !before.is_empty() {
                return Some((n.max(1), before));
            }
        }
    }

    Some((1, s))
}

/// Detect section header lines.
fn detect_header(line: &str) -> Option<SectionContext> {
    let lower = line.to_ascii_lowercase();
    let trimmed = lower.trim().trim_matches(|c| c == '=' || c == '-' || c == '#' || c == '[' || c == ']');
    let t = trimmed.trim();

    if t.starts_with("side") {
        return Some(SectionContext::Side);
    }
    if t.starts_with("extra") {
        return Some(SectionContext::Extra);
    }
    if t.starts_with("main") {
        return Some(SectionContext::Main);
    }

    // Generic monster/spell/trap headers: e.g. "== MONSTER CARDS (70 cards) =="
    if t.contains("monster") || t.contains("spell") || t.contains("trap") {
        return Some(SectionContext::Unknown);
    }

    None
}

/// Resolve card name in database with exact match or fuzzy match fallback.
fn resolve_card<'a>(name: &str, db: &'a CardDatabase, threshold: f64) -> Option<&'a Card> {
    // 1. Exact match (case & punctuation-insensitive)
    if let Some(card) = db.find_by_name(name) {
        return Some(card);
    }

    // 2. Strip trailing parenthetical e.g. "Card Name (Secret Rare)" or "(DABL-EN012)"
    if let Some(paren_idx) = name.rfind('(') {
        let stripped = name[..paren_idx].trim();
        if let Some(card) = db.find_by_name(stripped) {
            return Some(card);
        }
    }

    // 3. Fuzzy search
    if let Some(fuzzy) = db.fuzzy_find(name, threshold) {
        return db.get(fuzzy.code);
    }

    None
}

/// Import a plain-text decklist using the specified options.
pub fn import_plaintext(
    text: &str,
    db: &CardDatabase,
    opts: &ImportOptions,
) -> Result<ImportedDeck> {
    let mut imported = ImportedDeck::default();
    let mut explicit_context = SectionContext::Unknown;
    let mut has_explicit_side = false;

    // Temporary storage:
    // extra cards are placed in `deck.extra`
    // explicit side cards are placed in `deck.side`
    // non-extra cards with Unknown/Main context go into `candidate_main`
    let mut candidate_main: Vec<u32> = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        // Check if line is a section header
        if let Some(header_ctx) = detect_header(line) {
            explicit_context = header_ctx;
            if header_ctx == SectionContext::Side {
                has_explicit_side = true;
            }
            continue;
        }

        let Some((count, card_name)) = parse_line_count_and_name(line) else {
            continue;
        };

        // Try to match card
        let Some(card) = resolve_card(card_name, db, opts.fuzzy_threshold) else {
            imported.unresolved.push(line.to_string());
            continue;
        };

        // Route card
        for _ in 0..count {
            match explicit_context {
                SectionContext::Extra => {
                    imported.deck.extra.push(card.code);
                }
                SectionContext::Side => {
                    imported.deck.side.push(card.code);
                }
                SectionContext::Main | SectionContext::Unknown => {
                    // Auto-partition Extra deck monsters
                    if card.is_extra_deck_monster() {
                        imported.deck.extra.push(card.code);
                    } else {
                        candidate_main.push(card.code);
                    }
                }
            }
        }
    }

    // Handle splitting candidate_main into Main and Side if overflow occurs
    let target_main = match opts.split_strategy {
        SplitStrategy::TargetMain(n) => Some(n),
        SplitStrategy::Auto => {
            if has_explicit_side {
                None // User already had an explicit Side section
            } else if candidate_main.len() > 60 {
                Some(opts.target_main_size.clamp(40, 60))
            } else if candidate_main.len() > opts.target_main_size {
                Some(opts.target_main_size)
            } else {
                None
            }
        }
        SplitStrategy::Never => None,
    };

    if let Some(target) = target_main {
        if candidate_main.len() > target {
            let overflow_count = candidate_main.len() - target;
            let main_part = &candidate_main[..target];
            let side_part = &candidate_main[target..];

            imported.deck.main.extend_from_slice(main_part);
            imported.deck.side.extend_from_slice(side_part);
            imported.warnings.push(format!(
                "Split {overflow_count} non-Extra cards into Side Deck (Main: {target}, Side: {})",
                imported.deck.side.len()
            ));
        } else {
            imported.deck.main.extend_from_slice(&candidate_main);
        }
    } else {
        imported.deck.main.extend_from_slice(&candidate_main);
    }

    Ok(imported)
}

/// Smart plain-text deck parser with default options.
/// Returns an error if any card lines could not be resolved.
pub fn parse_plaintext_deck(text: &str, db: &CardDatabase) -> Result<Deck> {
    let imported = import_plaintext(text, db, &ImportOptions::default())?;
    if !imported.unresolved.is_empty() {
        return Err(Error::UnresolvedCards(imported.unresolved));
    }
    Ok(imported.deck)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;

    fn mock_db() -> CardDatabase {
        let mut reino = Card::new(73956664, "Tearlaments Reinoheart", TYPE_MONSTER | TYPE_EFFECT);
        reino.race = RACE_AQUA;
        reino.attribute = ATTRIBUTE_WATER;

        let mut rulkallos = Card::new(84330567, "Tearlaments Rulkallos", TYPE_MONSTER | TYPE_FUSION | TYPE_EFFECT);
        rulkallos.race = RACE_AQUA;
        rulkallos.attribute = ATTRIBUTE_WATER;

        let droll = Card::new(94145021, "Droll & Lock Bird", TYPE_MONSTER | TYPE_EFFECT);

        CardDatabase::from_cards([reino, rulkallos, droll])
    }

    #[test]
    fn test_parse_line() {
        assert_eq!(parse_line_count_and_name("3x Tearlaments Reinoheart"), Some((3, "Tearlaments Reinoheart")));
        assert_eq!(parse_line_count_and_name("3 Tearlaments Reinoheart"), Some((3, "Tearlaments Reinoheart")));
        assert_eq!(parse_line_count_and_name("Tearlaments Reinoheart x3"), Some((3, "Tearlaments Reinoheart")));
        assert_eq!(parse_line_count_and_name("Tearlaments Reinoheart"), Some((1, "Tearlaments Reinoheart")));
    }

    #[test]
    fn test_auto_partition_extra() {
        let db = mock_db();
        let input = "\
3x Tearlaments Reinoheart
1x Tearlaments Rulkallos
";
        let deck = parse_plaintext_deck(input, &db).unwrap();
        assert_eq!(deck.main.len(), 3);
        assert_eq!(deck.extra.len(), 1);
        assert_eq!(deck.extra[0], 84330567);
    }
}
