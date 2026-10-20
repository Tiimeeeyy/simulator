//! Date-aware ("time-travel") Forbidden & Limited list registry.
//!
//! `lflist.conf` files are static snapshots. The [`BanlistRegistry`] indexes
//! every list it loads by **region** and **effective date**, so a
//! [`BanlistQuery`] such as *"TCG as of 2023-03-01"* resolves to the list that
//! was in force that day (`!2023.02 TCG`).
//!
//! ## Accepted input
//!
//! Standard ygopro / EDOPro / ProjectIgnis `lflist.conf` syntax; a single file
//! may contain many lists:
//!
//! ```text
//! #[2023.02 TCG]
//! !2023.02 TCG
//! #effective: 2023-02-13      <- optional exact date (default: 1st of month)
//! $whitelist                   <- optional: unlisted cards are forbidden
//! 92731385 0 --Tearlaments Kitkallos
//! 572850 1 --Tearlaments Scheiren
//! ```
//!
//! Headers of the form `!YYYY.MM[.DD] TCG|OCG` are placed on the TCG / OCG
//! timelines. Anything else (`!Traditional`, `!GOAT`, …) is kept as a named
//! custom list retrievable through [`BanlistRegistry::get`].

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, LazyLock};

use chrono::NaiveDate;
use regex::Regex;

use crate::constants::{DEFAULT_MAX_COPIES, OT_OCG, OT_TCG};
use crate::database::CardDatabase;
use crate::error::{Error, Result};

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FormatRegion {
    Tcg,
    Ocg,
    /// TCG Traditional: the TCG list with every Forbidden card treated as Limited.
    Traditional,
    /// No Forbidden/Limited list at all.
    Unlimited,
}

impl FormatRegion {
    /// Which dated timeline backs this region.
    pub fn timeline(self) -> Option<FormatRegion> {
        match self {
            FormatRegion::Tcg | FormatRegion::Traditional => Some(FormatRegion::Tcg),
            FormatRegion::Ocg => Some(FormatRegion::Ocg),
            FormatRegion::Unlimited => None,
        }
    }

    /// `OT_*` scope bit a card needs to be legal in this region (0 = any).
    pub fn scope_flag(self) -> u32 {
        match self {
            FormatRegion::Tcg | FormatRegion::Traditional => OT_TCG,
            FormatRegion::Ocg => OT_OCG,
            FormatRegion::Unlimited => 0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FormatRegion::Tcg => "TCG",
            FormatRegion::Ocg => "OCG",
            FormatRegion::Traditional => "Traditional",
            FormatRegion::Unlimited => "Unlimited",
        }
    }
}

impl fmt::Display for FormatRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for FormatRegion {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "tcg" => Ok(FormatRegion::Tcg),
            "ocg" => Ok(FormatRegion::Ocg),
            "traditional" | "trad" => Ok(FormatRegion::Traditional),
            "unlimited" | "none" | "nobanlist" => Ok(FormatRegion::Unlimited),
            other => Err(format!("unknown format region '{other}'")),
        }
    }
}

/// "Which list applied in `region` on `as_of_date`?"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BanlistQuery {
    pub region: FormatRegion,
    pub as_of_date: NaiveDate,
    /// Also reject cards not yet released in `region` on `as_of_date`
    /// (uses `OT_*` scope flags and, when available, a
    /// [`ReleaseDateIndex`](crate::ReleaseDateIndex)).
    pub enforce_release_dates: bool,
}

impl BanlistQuery {
    pub fn new(region: FormatRegion, as_of_date: NaiveDate) -> Self {
        BanlistQuery { region, as_of_date, enforce_release_dates: false }
    }
    pub fn tcg(as_of_date: NaiveDate) -> Self {
        Self::new(FormatRegion::Tcg, as_of_date)
    }
    pub fn ocg(as_of_date: NaiveDate) -> Self {
        Self::new(FormatRegion::Ocg, as_of_date)
    }
    pub fn with_release_dates(mut self, enforce: bool) -> Self {
        self.enforce_release_dates = enforce;
        self
    }
}

/// What the user picked in the UI: a historical point in time, "whatever is
/// current", or no list at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BanlistSelection {
    NoBanlist,
    AsOf(BanlistQuery),
    Latest { region: FormatRegion, enforce_release_dates: bool },
}

// ---------------------------------------------------------------------------
// Banlist
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Banlist {
    pub name: String,
    pub region: Option<FormatRegion>,
    pub effective_date: Option<NaiveDate>,
    /// If set, any card *not* listed is Forbidden.
    pub whitelist: bool,
    entries: HashMap<u32, u8>,
}

static HEADER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(\d{4})\.(\d{1,2})(?:\.(\d{1,2}))?\s+(TCG|OCG)\b").unwrap()
});

impl Banlist {
    /// Create an empty list; region/date are inferred from the name if it
    /// follows the `YYYY.MM[.DD] TCG|OCG` convention.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let (region, effective_date) = Self::parse_header(&name);
        Banlist { name, region, effective_date, whitelist: false, entries: HashMap::new() }
    }

    /// `"2023.02 TCG"` → `(Some(Tcg), Some(2023-02-01))`.
    pub fn parse_header(name: &str) -> (Option<FormatRegion>, Option<NaiveDate>) {
        let Some(c) = HEADER_RE.captures(name.trim()) else { return (None, None) };
        let y: i32 = c[1].parse().unwrap_or(0);
        let m: u32 = c[2].parse().unwrap_or(0);
        let d: u32 = c.get(3).map_or(1, |d| d.as_str().parse().unwrap_or(1));
        let region = if c[4].eq_ignore_ascii_case("TCG") { FormatRegion::Tcg } else { FormatRegion::Ocg };
        match NaiveDate::from_ymd_opt(y, m, d) {
            Some(date) => (Some(region), Some(date)),
            None => (None, None),
        }
    }

    pub fn set(&mut self, code: u32, limit: u8) {
        self.entries.insert(code, limit.min(DEFAULT_MAX_COPIES));
    }

    /// Explicit entry for `code`, if listed.
    pub fn entry(&self, code: u32) -> Option<u8> {
        self.entries.get(&code).copied()
    }

    /// Effective limit for `code` (no alias handling; see [`ActiveBanlist::limit_for`]).
    pub fn limit(&self, code: u32) -> u8 {
        self.entry(code).unwrap_or(if self.whitelist { 0 } else { DEFAULT_MAX_COPIES })
    }

    pub fn entries(&self) -> impl Iterator<Item = (u32, u8)> + '_ {
        self.entries.iter().map(|(&c, &l)| (c, l))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Parse an `lflist.conf` (possibly containing many lists).
pub fn parse_lflist(text: &str) -> Result<Vec<Banlist>> {
    let mut lists: Vec<Banlist> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line_no = i + 1;
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_prefix('!') {
            lists.push(Banlist::new(name.trim()));
            continue;
        }
        if let Some(rest) = line.strip_prefix('#') {
            // Optional exact-date directive: `#effective: 2023-02-13`
            let rest = rest.trim();
            if let Some(date) = rest.strip_prefix("effective") {
                let date = date.trim_start_matches(':').trim();
                let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|e| Error::Parse {
                    line: line_no,
                    message: format!("bad #effective date '{date}': {e}"),
                })?;
                if let Some(cur) = lists.last_mut() {
                    cur.effective_date = Some(parsed);
                }
            }
            continue;
        }
        let Some(cur) = lists.last_mut() else {
            return Err(Error::Parse { line: line_no, message: "entry before any '!name' header".into() });
        };
        if let Some(directive) = line.strip_prefix('$') {
            if directive.trim().eq_ignore_ascii_case("whitelist") {
                cur.whitelist = true;
            }
            continue;
        }
        let content = if let Some(idx) = line.find("--") {
            &line[..idx]
        } else if let Some(idx) = line.find('#') {
            &line[..idx]
        } else {
            line
        }.trim();
        if content.is_empty() {
            continue;
        }
        let mut tok = content.split_whitespace();
        let (Some(code), Some(count)) = (tok.next(), tok.next()) else {
            return Err(Error::Parse { line: line_no, message: format!("expected '<code> <count>', got '{line}'") });
        };
        let code: u32 = code
            .parse()
            .map_err(|_| Error::Parse { line: line_no, message: format!("bad passcode '{code}'") })?;
        let count: i32 = count
            .parse()
            .map_err(|_| Error::Parse { line: line_no, message: format!("bad count '{count}'") })?;
        cur.set(code, count.clamp(0, 3) as u8);
    }
    Ok(lists)
}

// ---------------------------------------------------------------------------
// Active (resolved) list
// ---------------------------------------------------------------------------

/// The outcome of resolving a query: the list (if any) plus the region rules
/// that modify it (Traditional turns Forbidden into Limited).
#[derive(Debug, Clone)]
pub struct ActiveBanlist {
    pub query: BanlistQuery,
    pub list: Option<Arc<Banlist>>,
    pub forbidden_as_limited: bool,
}

impl ActiveBanlist {
    pub fn unlimited(as_of_date: NaiveDate) -> Self {
        ActiveBanlist {
            query: BanlistQuery::new(FormatRegion::Unlimited, as_of_date),
            list: None,
            forbidden_as_limited: false,
        }
    }

    pub fn name(&self) -> String {
        match (&self.list, self.forbidden_as_limited) {
            (None, _) => "No banlist".to_string(),
            (Some(l), true) => format!("Traditional ({})", l.name),
            (Some(l), false) => l.name.clone(),
        }
    }

    fn adjust(&self, limit: u8) -> u8 {
        if self.forbidden_as_limited && limit == 0 { 1 } else { limit }
    }

    /// Limit for an exact passcode (no alias normalisation).
    pub fn limit(&self, code: u32) -> u8 {
        match &self.list {
            None => DEFAULT_MAX_COPIES,
            Some(l) => match l.entry(code) {
                Some(n) => self.adjust(n),
                None => l.limit(code), // whitelist absence stays 0 even in Traditional
            },
        }
    }

    /// Limit for a card, normalising alternate artworks first. The base
    /// passcode's entry wins; an entry on the alternate passcode is used as a
    /// fallback (some lists only name one of the artworks).
    pub fn limit_for(&self, db: &CardDatabase, code: u32) -> u8 {
        let Some(list) = &self.list else { return DEFAULT_MAX_COPIES };
        let base = db.canonical_code(code);
        match list.entry(base).or_else(|| list.entry(code)) {
            Some(n) => self.adjust(n),
            None => list.limit(base),
        }
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct BanlistRegistry {
    timelines: HashMap<FormatRegion, BTreeMap<NaiveDate, Arc<Banlist>>>,
    custom: BTreeMap<String, Arc<Banlist>>,
}

impl BanlistRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a list. Dated TCG/OCG lists go on their timeline (a later insert
    /// with the same date replaces the earlier one); others become custom lists.
    pub fn insert(&mut self, list: Banlist) {
        match (list.region, list.effective_date) {
            (Some(region), Some(date)) => {
                self.timelines.entry(region).or_default().insert(date, Arc::new(list));
            }
            _ => {
                self.custom.insert(list.name.clone(), Arc::new(list));
            }
        }
    }

    pub fn load_str(&mut self, text: &str) -> Result<usize> {
        let lists = parse_lflist(text)?;
        let n = lists.len();
        lists.into_iter().for_each(|l| self.insert(l));
        Ok(n)
    }

    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<usize> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        self.load_str(&text).map_err(|e| match e {
            Error::Parse { line, message } => {
                Error::Parse { line, message: format!("{}: {message}", path.display()) }
            }
            other => other,
        })
    }

    /// Load every `*.conf` file in `dir` (sorted by file name, non-recursive).
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        let mut files: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| Error::io(dir, e))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "conf"))
            .collect();
        files.sort();
        let mut reg = Self::new();
        for f in files {
            reg.load_file(&f)?;
        }
        Ok(reg)
    }

    /// Number of dated (timeline) lists.
    pub fn len(&self) -> usize {
        self.timelines.values().map(BTreeMap::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0 && self.custom.is_empty()
    }

    /// All dated lists of a region, oldest first.
    pub fn timeline(&self, region: FormatRegion) -> impl Iterator<Item = &Arc<Banlist>> {
        region
            .timeline()
            .and_then(|r| self.timelines.get(&r))
            .into_iter()
            .flat_map(|t| t.values())
    }

    pub fn earliest(&self, region: FormatRegion) -> Option<NaiveDate> {
        region.timeline().and_then(|r| self.timelines.get(&r)?.keys().next().copied())
    }

    pub fn latest(&self, region: FormatRegion) -> Option<NaiveDate> {
        region.timeline().and_then(|r| self.timelines.get(&r)?.keys().next_back().copied())
    }

    /// Look up any list (dated or custom) by its exact name.
    pub fn get(&self, name: &str) -> Option<Arc<Banlist>> {
        self.custom.get(name).cloned().or_else(|| {
            self.timelines.values().flat_map(|t| t.values()).find(|l| l.name == name).cloned()
        })
    }

    /// All banlist names.
    pub fn names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for list in self.timelines.values().flat_map(|t| t.values()) {
            names.push(list.name.clone());
        }
        for name in self.custom.keys() {
            names.push(name.clone());
        }
        names
    }

    /// Resolve the list in force for `query`: the newest list of the region
    /// whose effective date is `<= as_of_date`.
    pub fn resolve(&self, query: &BanlistQuery) -> Result<ActiveBanlist> {
        let Some(timeline_region) = query.region.timeline() else {
            return Ok(ActiveBanlist { query: *query, list: None, forbidden_as_limited: false });
        };
        let list = self
            .timelines
            .get(&timeline_region)
            .and_then(|t| t.range(..=query.as_of_date).next_back())
            .map(|(_, l)| l.clone())
            .ok_or(Error::NoBanlistForDate {
                region: query.region,
                date: query.as_of_date,
                earliest: self.earliest(query.region),
            })?;
        Ok(ActiveBanlist {
            query: *query,
            list: Some(list),
            forbidden_as_limited: query.region == FormatRegion::Traditional,
        })
    }

    /// Resolve a UI selection; `today` anchors [`BanlistSelection::Latest`].
    pub fn resolve_selection(&self, sel: &BanlistSelection, today: NaiveDate) -> Result<ActiveBanlist> {
        match *sel {
            BanlistSelection::NoBanlist => Ok(ActiveBanlist::unlimited(today)),
            BanlistSelection::AsOf(q) => self.resolve(&q),
            BanlistSelection::Latest { region, enforce_release_dates } => self.resolve(&BanlistQuery {
                region,
                as_of_date: today,
                enforce_release_dates,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    const SAMPLE: &str = "\
#[2022.12 TCG]
!2022.12 TCG
#Forbidden
92731385 1 --Tearlaments Kitkallos
!2023.02 TCG
#effective: 2023-02-13
92731385 0 --Tearlaments Kitkallos
572850 1 --Tearlaments Scheiren
!2023.01 OCG
92731385 0 --Tearlaments Kitkallos
!2026.10 TCG
62320425 0 --Agido
!GOAT
$whitelist
12345 3
";

    fn registry() -> BanlistRegistry {
        let mut r = BanlistRegistry::new();
        assert_eq!(r.load_str(SAMPLE).unwrap(), 5);
        r
    }

    #[test]
    fn header_parsing() {
        assert_eq!(Banlist::parse_header("2023.02 TCG"), (Some(FormatRegion::Tcg), Some(d(2023, 2, 1))));
        assert_eq!(Banlist::parse_header("2026.10.20 OCG"), (Some(FormatRegion::Ocg), Some(d(2026, 10, 20))));
        assert_eq!(Banlist::parse_header("Traditional"), (None, None));
        assert_eq!(Banlist::parse_header("2023.13 TCG"), (None, None));
    }

    #[test]
    fn parse_with_directives() {
        let lists = parse_lflist(SAMPLE).unwrap();
        assert_eq!(lists[1].effective_date, Some(d(2023, 2, 13)));
        assert_eq!(lists[1].limit(92731385), 0);
        assert_eq!(lists[1].limit(1), 3);
        let goat = &lists[4];
        assert!(goat.whitelist);
        assert_eq!(goat.limit(12345), 3);
        assert_eq!(goat.limit(1), 0, "whitelist: unlisted = forbidden");
        assert!(parse_lflist("123 1\n").is_err(), "entry without header");
        assert!(parse_lflist("!x\nabc 1\n").is_err());
    }

    #[test]
    fn point_in_time_resolution() {
        let r = registry();
        let name = |q: BanlistQuery| r.resolve(&q).unwrap().name();
        assert_eq!(name(BanlistQuery::tcg(d(2023, 1, 15))), "2022.12 TCG");
        assert_eq!(name(BanlistQuery::tcg(d(2023, 2, 12))), "2022.12 TCG", "exact #effective date honoured");
        assert_eq!(name(BanlistQuery::tcg(d(2023, 2, 13))), "2023.02 TCG");
        assert_eq!(name(BanlistQuery::tcg(d(2023, 3, 1))), "2023.02 TCG");
        assert_eq!(name(BanlistQuery::tcg(d(2026, 10, 20))), "2026.10 TCG");
        assert_eq!(name(BanlistQuery::ocg(d(2026, 10, 20))), "2023.01 OCG");
        assert!(matches!(
            r.resolve(&BanlistQuery::tcg(d(2020, 1, 1))),
            Err(Error::NoBanlistForDate { earliest: Some(e), .. }) if e == d(2022, 12, 1)
        ));
        assert_eq!(r.len(), 4);
        assert!(r.get("GOAT").unwrap().whitelist);
    }

    #[test]
    fn traditional_and_unlimited() {
        let r = registry();
        let q = BanlistQuery::new(FormatRegion::Traditional, d(2023, 3, 1));
        let trad = r.resolve(&q).unwrap();
        assert_eq!(trad.limit(92731385), 1, "forbidden → limited");
        assert_eq!(trad.limit(572850), 1);
        assert_eq!(trad.name(), "Traditional (2023.02 TCG)");
        let unl = r.resolve(&BanlistQuery::new(FormatRegion::Unlimited, d(2023, 3, 1))).unwrap();
        assert_eq!(unl.limit(92731385), 3);
        let latest = r
            .resolve_selection(
                &BanlistSelection::Latest { region: FormatRegion::Tcg, enforce_release_dates: false },
                d(2026, 10, 20),
            )
            .unwrap();
        assert_eq!(latest.name(), "2026.10 TCG");
        assert_eq!(r.resolve_selection(&BanlistSelection::NoBanlist, d(2026, 1, 1)).unwrap().name(), "No banlist");
    }
}
