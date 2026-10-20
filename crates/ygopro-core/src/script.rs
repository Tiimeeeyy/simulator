//! Lua script loader for ocgcore.
//!
//! Searches configured directories for card scripts (`c<id>.lua`) and
//! helper libraries (`constant.lua`, `utility.lua`, `proc_*.lua`).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct ScriptLoader {
    search_dirs: Vec<PathBuf>,
    cache: RwLock<HashMap<String, Vec<u8>>>,
}

impl ScriptLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.add_search_dir(dir);
        self
    }

    pub fn add_search_dir(&mut self, dir: impl Into<PathBuf>) {
        let p = dir.into();
        if !self.search_dirs.contains(&p) {
            self.search_dirs.push(p);
        }
    }

    /// Clean script name: removes leading `./`, `script/`, etc.
    fn clean_name(name: &str) -> &str {
        let s = name.trim();
        let s = s.strip_prefix("./").unwrap_or(s);
        let s = s.strip_prefix("scripts/").unwrap_or(s);
        s.strip_prefix("script/").unwrap_or(s)
    }

    /// Read script bytes for `name`.
    pub fn read_script(&self, name: &str) -> Option<Vec<u8>> {
        let clean = Self::clean_name(name);

        // Check in-memory cache first
        if let Ok(c) = self.cache.read() {
            if let Some(bytes) = c.get(clean) {
                return Some(bytes.clone());
            }
        }

        for base in &self.search_dirs {
            // Direct path
            let direct = base.join(clean);
            if direct.is_file() {
                if let Ok(data) = std::fs::read(&direct) {
                    if let Ok(mut c) = self.cache.write() {
                        c.insert(clean.to_string(), data.clone());
                    }
                    return Some(data);
                }
            }

            // If it's a card script (starts with 'c' followed by digits and .lua), check official/
            if clean.starts_with('c') && clean.ends_with(".lua") {
                let official = base.join("official").join(clean);
                if official.is_file() {
                    if let Ok(data) = std::fs::read(&official) {
                        if let Ok(mut c) = self.cache.write() {
                            c.insert(clean.to_string(), data.clone());
                        }
                        return Some(data);
                    }
                }
            }
        }

        None
    }
}
