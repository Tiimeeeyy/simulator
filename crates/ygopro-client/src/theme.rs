//! Master Duel inspired design system and styling tokens for Yu-Gi-Oh! desktop client.
#![allow(dead_code)]

use bevy_egui::egui::{Color32, Stroke};
use ygopro_data::constants::*;

// Deep space obsidian & glassmorphism backgrounds
pub const BG_DARK: Color32 = Color32::from_rgb(10, 12, 20);
pub const BG_PANEL: Color32 = Color32::from_rgb(16, 21, 34);
pub const BG_CARD: Color32 = Color32::from_rgb(24, 30, 48);
pub const BG_ACCENT: Color32 = Color32::from_rgb(32, 42, 68);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(20, 26, 42);

// Master Duel Accent Highlights
pub const ACCENT_GOLD: Color32 = Color32::from_rgb(245, 175, 40);
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(56, 160, 255);
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(175, 85, 245);
pub const ACCENT_CYAN: Color32 = Color32::from_rgb(45, 225, 235);
pub const ACCENT_RED: Color32 = Color32::from_rgb(240, 75, 75);
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(50, 220, 120);

// Typography colors
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(245, 247, 252);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(140, 155, 185);
pub const TEXT_DARK: Color32 = Color32::from_rgb(15, 15, 18);
pub const TEXT_GOLD: Color32 = Color32::from_rgb(255, 215, 90);

// Borders & Strokes
pub const BORDER_DARK: Color32 = Color32::from_rgb(34, 44, 70);
pub const BORDER_LIGHT: Color32 = Color32::from_rgb(50, 65, 100);

// Card Frame Palette
pub const COLOR_NORMAL_MONSTER: Color32 = Color32::from_rgb(196, 168, 92);
pub const COLOR_EFFECT_MONSTER: Color32 = Color32::from_rgb(195, 100, 45);
pub const COLOR_RITUAL_MONSTER: Color32 = Color32::from_rgb(60, 130, 220);
pub const COLOR_FUSION_MONSTER: Color32 = Color32::from_rgb(145, 60, 185);
pub const COLOR_SYNCHRO_MONSTER: Color32 = Color32::from_rgb(238, 240, 245);
pub const COLOR_XYZ_MONSTER: Color32 = Color32::from_rgb(20, 20, 26);
pub const COLOR_LINK_MONSTER: Color32 = Color32::from_rgb(25, 95, 200);
pub const COLOR_SPELL: Color32 = Color32::from_rgb(24, 155, 115);
pub const COLOR_TRAP: Color32 = Color32::from_rgb(185, 35, 115);

/// Get the primary frame color for a card.
pub fn card_type_color(type_flags: u32) -> Color32 {
    if (type_flags & TYPE_SPELL) != 0 {
        COLOR_SPELL
    } else if (type_flags & TYPE_TRAP) != 0 {
        COLOR_TRAP
    } else if (type_flags & TYPE_LINK) != 0 {
        COLOR_LINK_MONSTER
    } else if (type_flags & TYPE_XYZ) != 0 {
        COLOR_XYZ_MONSTER
    } else if (type_flags & TYPE_SYNCHRO) != 0 {
        COLOR_SYNCHRO_MONSTER
    } else if (type_flags & TYPE_FUSION) != 0 {
        COLOR_FUSION_MONSTER
    } else if (type_flags & TYPE_RITUAL) != 0 {
        COLOR_RITUAL_MONSTER
    } else if (type_flags & TYPE_EFFECT) != 0 {
        COLOR_EFFECT_MONSTER
    } else {
        COLOR_NORMAL_MONSTER
    }
}

pub fn card_type_label(type_flags: u32) -> &'static str {
    if (type_flags & TYPE_SPELL) != 0 {
        "SPELL"
    } else if (type_flags & TYPE_TRAP) != 0 {
        "TRAP"
    } else if (type_flags & TYPE_LINK) != 0 {
        "LINK"
    } else if (type_flags & TYPE_XYZ) != 0 {
        "XYZ"
    } else if (type_flags & TYPE_SYNCHRO) != 0 {
        "SYNCHRO"
    } else if (type_flags & TYPE_FUSION) != 0 {
        "FUSION"
    } else if (type_flags & TYPE_RITUAL) != 0 {
        "RITUAL"
    } else if (type_flags & TYPE_EFFECT) != 0 {
        "EFFECT"
    } else {
        "NORMAL"
    }
}

pub fn spell_trap_type_label(type_flags: u32) -> &'static str {
    if (type_flags & TYPE_QUICKPLAY) != 0 {
        "⚡ Quick-Play"
    } else if (type_flags & TYPE_CONTINUOUS) != 0 {
        "♾ Continuous"
    } else if (type_flags & TYPE_FIELD) != 0 {
        "🌐 Field"
    } else if (type_flags & TYPE_EQUIP) != 0 {
        "🗡 Equip"
    } else if (type_flags & TYPE_COUNTER) != 0 {
        "↩ Counter"
    } else if (type_flags & TYPE_RITUAL) != 0 {
        "🕯 Ritual"
    } else if (type_flags & TYPE_SPELL) != 0 {
        "Normal Spell"
    } else if (type_flags & TYPE_TRAP) != 0 {
        "Normal Trap"
    } else {
        ""
    }
}

pub fn attribute_label(attr: u32) -> &'static str {
    match attr {
        ATTRIBUTE_DARK => "DARK",
        ATTRIBUTE_LIGHT => "LIGHT",
        ATTRIBUTE_WATER => "WATER",
        ATTRIBUTE_FIRE => "FIRE",
        ATTRIBUTE_EARTH => "EARTH",
        ATTRIBUTE_WIND => "WIND",
        ATTRIBUTE_DIVINE => "DIVINE",
        _ => "",
    }
}

pub fn attribute_icon(attr: u32) -> &'static str {
    match attr {
        ATTRIBUTE_DARK => "🟣 DARK",
        ATTRIBUTE_LIGHT => "🟡 LIGHT",
        ATTRIBUTE_WATER => "🔵 WATER",
        ATTRIBUTE_FIRE => "🔴 FIRE",
        ATTRIBUTE_EARTH => "🟤 EARTH",
        ATTRIBUTE_WIND => "🟢 WIND",
        ATTRIBUTE_DIVINE => "👑 DIVINE",
        _ => "⚫ UNKNOWN",
    }
}

pub fn attribute_color(attr: u32) -> Color32 {
    match attr {
        ATTRIBUTE_DARK => Color32::from_rgb(155, 75, 210),
        ATTRIBUTE_LIGHT => Color32::from_rgb(245, 220, 70),
        ATTRIBUTE_WATER => Color32::from_rgb(45, 150, 245),
        ATTRIBUTE_FIRE => Color32::from_rgb(240, 75, 40),
        ATTRIBUTE_EARTH => Color32::from_rgb(175, 115, 55),
        ATTRIBUTE_WIND => Color32::from_rgb(50, 215, 125),
        ATTRIBUTE_DIVINE => Color32::from_rgb(240, 180, 25),
        _ => Color32::GRAY,
    }
}

pub fn race_label(race: u64) -> &'static str {
    match race {
        RACE_WARRIOR => "Warrior",
        RACE_SPELLCASTER => "Spellcaster",
        RACE_FAIRY => "Fairy",
        RACE_FIEND => "Fiend",
        RACE_ZOMBIE => "Zombie",
        RACE_MACHINE => "Machine",
        RACE_AQUA => "Aqua",
        RACE_PYRO => "Pyro",
        RACE_ROCK => "Rock",
        RACE_WINGEDBEAST => "Winged Beast",
        RACE_PLANT => "Plant",
        RACE_INSECT => "Insect",
        RACE_THUNDER => "Thunder",
        RACE_DRAGON => "Dragon",
        RACE_BEAST => "Beast",
        RACE_BEASTWARRIOR => "Beast-Warrior",
        RACE_DINOSAUR => "Dinosaur",
        RACE_FISH => "Fish",
        RACE_SEASERPENT => "Sea Serpent",
        RACE_REPTILE => "Reptile",
        RACE_PSYCHIC => "Psychic",
        RACE_DIVINE => "Divine-Beast",
        RACE_CREATORGOD => "Creator God",
        RACE_WYRM => "Wyrm",
        RACE_CYBERSE => "Cyberse",
        RACE_ILLUSION => "Illusion",
        _ => "Monster",
    }
}

pub fn selection_stroke() -> Stroke {
    Stroke::new(2.5, ACCENT_GOLD)
}

pub fn hover_stroke() -> Stroke {
    Stroke::new(2.0, ACCENT_CYAN)
}

pub fn card_border_stroke() -> Stroke {
    Stroke::new(1.0, Color32::from_black_alpha(190))
}
