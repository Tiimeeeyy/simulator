//! Deck legality validation against Banlists, Release Dates, and Deck Construction Rules.

use std::collections::HashMap;
use std::fmt;

use chrono::NaiveDate;

use crate::banlist::{ActiveBanlist, FormatRegion};
use crate::database::CardDatabase;
use crate::deck::{Deck, DeckSection};
use crate::release::ReleaseDateIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    MainUnderflow { count: usize, min: usize },
    MainOverflow { count: usize, max: usize },
    ExtraOverflow { count: usize, max: usize },
    SideOverflow { count: usize, max: usize },
    NotDeckable { code: u32, name: String },
    ForbiddenInBanlist { code: u32, name: String },
    ExceedsLimit { code: u32, name: String, count: usize, limit: u8 },
    NotReleasedInRegion {
        code: u32,
        name: String,
        region: FormatRegion,
        as_of_date: NaiveDate,
        release_date: Option<NaiveDate>,
    },
    IllegalScope { code: u32, name: String, ot: u32, required: u32 },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::MainUnderflow { count, min } => {
                write!(f, "Main Deck has {count} cards (minimum is {min})")
            }
            Violation::MainOverflow { count, max } => {
                write!(f, "Main Deck has {count} cards (maximum is {max})")
            }
            Violation::ExtraOverflow { count, max } => {
                write!(f, "Extra Deck has {count} cards (maximum is {max})")
            }
            Violation::SideOverflow { count, max } => {
                write!(f, "Side Deck has {count} cards (maximum is {max})")
            }
            Violation::NotDeckable { name, code } => {
                write!(f, "Card '{name}' ({code}) is not allowed in a deck")
            }
            Violation::ForbiddenInBanlist { name, code } => {
                write!(f, "Card '{name}' ({code}) is Forbidden (0 allowed)")
            }
            Violation::ExceedsLimit { name, code, count, limit } => {
                write!(f, "Card '{name}' ({code}) has {count} copies (limit is {limit})")
            }
            Violation::NotReleasedInRegion { name, code, region, as_of_date, release_date } => {
                if let Some(rel) = release_date {
                    write!(
                        f,
                        "Card '{name}' ({code}) was released in {region} on {rel}, after target date {as_of_date}"
                    )
                } else {
                    write!(
                        f,
                        "Card '{name}' ({code}) was not released in {region} as of {as_of_date}"
                    )
                }
            }
            Violation::IllegalScope { name, code, ot, required } => {
                write!(f, "Card '{name}' ({code}) has OT {ot:#x}, missing required {required:#x}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeckRules {
    pub min_main: usize,
    pub max_main: usize,
    pub max_extra: usize,
    pub max_side: usize,
    pub enforce_deckable: bool,
}

impl Default for DeckRules {
    fn default() -> Self {
        DeckRules {
            min_main: 40,
            max_main: 60,
            max_extra: 15,
            max_side: 15,
            enforce_deckable: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeckReport {
    pub is_legal: bool,
    pub violations: Vec<Violation>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DeckValidator<'a> {
    db: &'a CardDatabase,
    rules: DeckRules,
    release_dates: Option<&'a ReleaseDateIndex>,
}

impl<'a> DeckValidator<'a> {
    pub fn new(db: &'a CardDatabase) -> Self {
        DeckValidator {
            db,
            rules: DeckRules::default(),
            release_dates: None,
        }
    }

    pub fn with_rules(mut self, rules: DeckRules) -> Self {
        self.rules = rules;
        self
    }

    pub fn with_release_dates(mut self, index: &'a ReleaseDateIndex) -> Self {
        self.release_dates = Some(index);
        self
    }

    pub fn validate(&self, deck: &Deck, active_banlist: &ActiveBanlist) -> DeckReport {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // 1. Check section counts
        if deck.main.len() < self.rules.min_main {
            violations.push(Violation::MainUnderflow {
                count: deck.main.len(),
                min: self.rules.min_main,
            });
        }
        if deck.main.len() > self.rules.max_main {
            violations.push(Violation::MainOverflow {
                count: deck.main.len(),
                max: self.rules.max_main,
            });
        }
        if deck.extra.len() > self.rules.max_extra {
            violations.push(Violation::ExtraOverflow {
                count: deck.extra.len(),
                max: self.rules.max_extra,
            });
        }
        if deck.side.len() > self.rules.max_side {
            violations.push(Violation::SideOverflow {
                count: deck.side.len(),
                max: self.rules.max_side,
            });
        }

        // 2. Check each card validity (deckable, scope, release date)
        let mut check_card = |code: u32, section: DeckSection, violations: &mut Vec<Violation>| {
            if let Some(card) = self.db.get(code) {
                if self.rules.enforce_deckable && !card.is_deckable() {
                    violations.push(Violation::NotDeckable {
                        code,
                        name: card.name.clone(),
                    });
                }

                // Check extra deck monster in wrong place (e.g. Extra monster in Main)
                if section == DeckSection::Main && card.is_extra_deck_monster() {
                    violations.push(Violation::NotDeckable {
                        code,
                        name: format!("{} (Extra Deck monster in Main Deck)", card.name),
                    });
                }
                if section == DeckSection::Extra && !card.is_extra_deck_monster() {
                    violations.push(Violation::NotDeckable {
                        code,
                        name: format!("{} (Main Deck card in Extra Deck)", card.name),
                    });
                }

                // Scope checks
                let scope_flag = active_banlist.query.region.scope_flag();
                if scope_flag != 0 && (card.ot & scope_flag == 0) {
                    violations.push(Violation::IllegalScope {
                        code,
                        name: card.name.clone(),
                        ot: card.ot,
                        required: scope_flag,
                    });
                }

                // Release date checks if enabled
                if active_banlist.query.enforce_release_dates {
                    if let Some(rel_index) = self.release_dates {
                        let region = active_banlist.query.region;
                        let as_of = active_banlist.query.as_of_date;
                        if let Some(dates) = rel_index.get(code) {
                            if !dates.is_released_on(region, as_of) {
                                violations.push(Violation::NotReleasedInRegion {
                                    code,
                                    name: card.name.clone(),
                                    region,
                                    as_of_date: as_of,
                                    release_date: dates.date_for(region),
                                });
                            }
                        } else {
                            // Unrecorded card release date
                            violations.push(Violation::NotReleasedInRegion {
                                code,
                                name: card.name.clone(),
                                region,
                                as_of_date: as_of,
                                release_date: None,
                            });
                        }
                    }
                }
            } else {
                warnings.push(format!("Unknown card passcode: {code}"));
            }
        };

        for &c in &deck.main {
            check_card(c, DeckSection::Main, &mut violations);
        }
        for &c in &deck.extra {
            check_card(c, DeckSection::Extra, &mut violations);
        }
        for &c in &deck.side {
            check_card(c, DeckSection::Side, &mut violations);
        }

        // 3. Count canonical copies across all sections and check banlist limits
        let mut canonical_counts: HashMap<u32, usize> = HashMap::new();
        for code in deck.all_cards() {
            let canonical = self.db.canonical_code(code);
            *canonical_counts.entry(canonical).or_insert(0) += 1;
        }

        for (canonical, count) in canonical_counts {
            let limit = active_banlist.limit_for(self.db, canonical);
            if count > limit as usize {
                let name = self.db.name_of(canonical);
                if limit == 0 {
                    violations.push(Violation::ForbiddenInBanlist {
                        code: canonical,
                        name,
                    });
                } else {
                    violations.push(Violation::ExceedsLimit {
                        code: canonical,
                        name,
                        count,
                        limit,
                    });
                }
            }
        }

        let is_legal = violations.is_empty();
        DeckReport {
            is_legal,
            violations,
            warnings,
        }
    }
}
