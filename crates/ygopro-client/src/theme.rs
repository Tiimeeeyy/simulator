//! Design system and styling tokens for Yu-Gi-Oh! desktop client.

use bevy_egui::egui::{Color32, Stroke};
use ygopro_data::constants::*;

pub const BG_DARK: Color32 = Color32::from_rgb(15, 17, 23);
pub const BG_PANEL: Color32 = Color32::from_rgb(22, 26, 35);
#[allow(dead_code)]
pub const BG_CARD: Color32 = Color32::from_rgb(28, 33, 46);
#[allow(dead_code)]
pub const BG_ACCENT: Color32 = Color32::from_rgb(38, 45, 64);

pub const ACCENT_GOLD: Color32 = Color32::from_rgb(230, 180, 60);
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(60, 140, 240);
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(160, 70, 220);
pub const ACCENT_CYAN: Color32 = Color32::from_rgb(50, 210, 210);
pub const ACCENT_RED: Color32 = Color32::from_rgb(230, 70, 70);
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(60, 200, 100);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 243, 248);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(150, 160, 180);
#[allow(dead_code)]
pub const TEXT_DARK: Color32 = Color32::from_rgb(20, 20, 20);

// Card Frame Colors
pub const COLOR_NORMAL_MONSTER: Color32 = Color32::from_rgb(196, 172, 98);
pub const COLOR_EFFECT_MONSTER: Color32 = Color32::from_rgb(190, 105, 50);
pub const COLOR_RITUAL_MONSTER: Color32 = Color32::from_rgb(85, 136, 187);
pub const COLOR_FUSION_MONSTER: Color32 = Color32::from_rgb(140, 60, 170);
pub const COLOR_SYNCHRO_MONSTER: Color32 = Color32::from_rgb(235, 238, 240);
pub const COLOR_XYZ_MONSTER: Color32 = Color32::from_rgb(25, 25, 30);
pub const COLOR_LINK_MONSTER: Color32 = Color32::from_rgb(20, 80, 160);
pub const COLOR_SPELL: Color32 = Color32::from_rgb(29, 158, 116);
pub const COLOR_TRAP: Color32 = Color32::from_rgb(180, 40, 110);

/// Get the theme color for a specific card type.
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

pub fn attribute_color(attr: u32) -> Color32 {
    match attr {
        ATTRIBUTE_DARK => Color32::from_rgb(140, 70, 190),
        ATTRIBUTE_LIGHT => Color32::from_rgb(230, 210, 80),
        ATTRIBUTE_WATER => Color32::from_rgb(50, 140, 230),
        ATTRIBUTE_FIRE => Color32::from_rgb(230, 70, 40),
        ATTRIBUTE_EARTH => Color32::from_rgb(160, 110, 50),
        ATTRIBUTE_WIND => Color32::from_rgb(60, 200, 120),
        ATTRIBUTE_DIVINE => Color32::from_rgb(220, 170, 30),
        _ => Color32::GRAY,
    }
}

pub fn selection_stroke() -> Stroke {
    Stroke::new(2.5, ACCENT_GOLD)
}

pub fn hover_stroke() -> Stroke {
    Stroke::new(1.5, ACCENT_CYAN)
}
