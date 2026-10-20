//! Per-passcode TCG / OCG release date index.
//!
//! Used by [`ActiveBanlist`] and [`DeckValidator`] when
//! `enforce_release_dates = true` to check whether a card was actually released
//! in a given region on or before `as_of_date`.

use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDate;

use crate::banlist::FormatRegion;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReleaseDates {
    pub tcg: Option<NaiveDate>,
    pub ocg: Option<NaiveDate>,
}

impl ReleaseDates {
    pub fn date_for(&self, region: FormatRegion) -> Option<NaiveDate> {
        match region {
            FormatRegion::Tcg | FormatRegion::Traditional => self.tcg,
            FormatRegion::Ocg => self.ocg,
            FormatRegion::Unlimited => None,
        }
    }

    pub fn is_released_on(&self, region: FormatRegion, as_of_date: NaiveDate) -> bool {
        match self.date_for(region) {
            Some(release_date) => release_date <= as_of_date,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReleaseDateIndex {
    entries: HashMap<u32, ReleaseDates>,
}

impl ReleaseDateIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        Self::load_str(&content)
    }

    pub fn load_str(content: &str) -> Result<Self> {
        let mut entries = HashMap::new();
        for (line_idx, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split(',');
            let Some(code_str) = parts.next() else { continue };
            let code: u32 = code_str.trim().parse().map_err(|_| Error::Parse {
                line: line_idx + 1,
                message: format!("invalid passcode: '{code_str}'"),
            })?;

            let tcg_str = parts.next().map(str::trim).unwrap_or("");
            let ocg_str = parts.next().map(str::trim).unwrap_or("");

            let tcg = if !tcg_str.is_empty() {
                NaiveDate::parse_from_str(tcg_str, "%Y-%m-%d").ok()
            } else {
                None
            };
            let ocg = if !ocg_str.is_empty() {
                NaiveDate::parse_from_str(ocg_str, "%Y-%m-%d").ok()
            } else {
                None
            };

            entries.insert(code, ReleaseDates { tcg, ocg });
        }
        Ok(ReleaseDateIndex { entries })
    }

    pub fn get(&self, code: u32) -> Option<&ReleaseDates> {
        self.entries.get(&code)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn is_released(
        &self,
        code: u32,
        region: FormatRegion,
        as_of_date: NaiveDate,
    ) -> Option<bool> {
        self.get(code).map(|dates| dates.is_released_on(region, as_of_date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_release_dates_csv() {
        let csv = "\
# passcode,tcg_date,ocg_date
4928565,2023-02-09,2022-10-15
7436169,2023-02-09,2022-10-15
";
        let idx = ReleaseDateIndex::load_str(csv).unwrap();
        assert_eq!(idx.len(), 2);

        let d_feb8 = NaiveDate::from_ymd_opt(2023, 2, 8).unwrap();
        let d_feb9 = NaiveDate::from_ymd_opt(2023, 2, 9).unwrap();

        assert_eq!(idx.is_released(4928565, FormatRegion::Tcg, d_feb8), Some(false));
        assert_eq!(idx.is_released(4928565, FormatRegion::Tcg, d_feb9), Some(true));
        assert_eq!(idx.is_released(4928565, FormatRegion::Ocg, d_feb8), Some(true));
    }
}
