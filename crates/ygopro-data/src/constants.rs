//! Engine constants mirrored from ocgcore / EDOPro `common.h`.

// ---- Card types (`datas.type`) -------------------------------------------
pub const TYPE_MONSTER: u32 = 0x1;
pub const TYPE_SPELL: u32 = 0x2;
pub const TYPE_TRAP: u32 = 0x4;
pub const TYPE_NORMAL: u32 = 0x10;
pub const TYPE_EFFECT: u32 = 0x20;
pub const TYPE_FUSION: u32 = 0x40;
pub const TYPE_RITUAL: u32 = 0x80;
pub const TYPE_TRAPMONSTER: u32 = 0x100;
pub const TYPE_SPIRIT: u32 = 0x200;
pub const TYPE_UNION: u32 = 0x400;
pub const TYPE_GEMINI: u32 = 0x800;
pub const TYPE_TUNER: u32 = 0x1000;
pub const TYPE_SYNCHRO: u32 = 0x2000;
pub const TYPE_TOKEN: u32 = 0x4000;
pub const TYPE_QUICKPLAY: u32 = 0x10000;
pub const TYPE_CONTINUOUS: u32 = 0x20000;
pub const TYPE_EQUIP: u32 = 0x40000;
pub const TYPE_FIELD: u32 = 0x80000;
pub const TYPE_COUNTER: u32 = 0x100000;
pub const TYPE_FLIP: u32 = 0x200000;
pub const TYPE_TOON: u32 = 0x400000;
pub const TYPE_XYZ: u32 = 0x800000;
pub const TYPE_PENDULUM: u32 = 0x1000000;
pub const TYPE_SPSUMMON: u32 = 0x2000000;
pub const TYPE_LINK: u32 = 0x4000000;
pub const TYPE_SKILL: u32 = 0x8000000;
pub const TYPE_ACTION: u32 = 0x10000000;

/// Monster types that live in the Extra Deck.
pub const TYPE_EXTRA_DECK: u32 = TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK;

// ---- Scopes / "OT" flags (`datas.ot`) ------------------------------------
pub const OT_OCG: u32 = 0x1;
pub const OT_TCG: u32 = 0x2;
pub const OT_ANIME: u32 = 0x4;
pub const OT_ILLEGAL: u32 = 0x8;
pub const OT_VIDEO_GAME: u32 = 0x10;
pub const OT_CUSTOM: u32 = 0x20;
pub const OT_SPEED: u32 = 0x40;
pub const OT_PRERELEASE: u32 = 0x100;
pub const OT_RUSH: u32 = 0x200;
pub const OT_LEGEND: u32 = 0x400;
pub const OT_HIDDEN: u32 = 0x1000;
pub const OT_OCG_TCG: u32 = OT_OCG | OT_TCG;

// ---- Attributes ------------------------------------------------------------
pub const ATTRIBUTE_EARTH: u32 = 0x01;
pub const ATTRIBUTE_WATER: u32 = 0x02;
pub const ATTRIBUTE_FIRE: u32 = 0x04;
pub const ATTRIBUTE_WIND: u32 = 0x08;
pub const ATTRIBUTE_LIGHT: u32 = 0x10;
pub const ATTRIBUTE_DARK: u32 = 0x20;
pub const ATTRIBUTE_DIVINE: u32 = 0x40;

pub const ATTRIBUTE_NAMES: &[(u32, &str)] = &[
    (ATTRIBUTE_EARTH, "EARTH"),
    (ATTRIBUTE_WATER, "WATER"),
    (ATTRIBUTE_FIRE, "FIRE"),
    (ATTRIBUTE_WIND, "WIND"),
    (ATTRIBUTE_LIGHT, "LIGHT"),
    (ATTRIBUTE_DARK, "DARK"),
    (ATTRIBUTE_DIVINE, "DIVINE"),
];

// ---- Races (EDOPro uses a 64-bit field) ------------------------------------
pub const RACE_WARRIOR: u64 = 0x1;
pub const RACE_SPELLCASTER: u64 = 0x2;
pub const RACE_FAIRY: u64 = 0x4;
pub const RACE_FIEND: u64 = 0x8;
pub const RACE_ZOMBIE: u64 = 0x10;
pub const RACE_MACHINE: u64 = 0x20;
pub const RACE_AQUA: u64 = 0x40;
pub const RACE_PYRO: u64 = 0x80;
pub const RACE_ROCK: u64 = 0x100;
pub const RACE_WINGEDBEAST: u64 = 0x200;
pub const RACE_PLANT: u64 = 0x400;
pub const RACE_INSECT: u64 = 0x800;
pub const RACE_THUNDER: u64 = 0x1000;
pub const RACE_DRAGON: u64 = 0x2000;
pub const RACE_BEAST: u64 = 0x4000;
pub const RACE_BEASTWARRIOR: u64 = 0x8000;
pub const RACE_DINOSAUR: u64 = 0x10000;
pub const RACE_FISH: u64 = 0x20000;
pub const RACE_SEASERPENT: u64 = 0x40000;
pub const RACE_REPTILE: u64 = 0x80000;
pub const RACE_PSYCHIC: u64 = 0x100000;
pub const RACE_DIVINE: u64 = 0x200000;
pub const RACE_CREATORGOD: u64 = 0x400000;
pub const RACE_WYRM: u64 = 0x800000;
pub const RACE_CYBERSE: u64 = 0x1000000;
pub const RACE_ILLUSION: u64 = 0x2000000;

/// Display names, **longest first** so that "Beast-Warrior" / "Winged Beast"
/// are matched before "Beast" when scanning card text.
pub const RACE_NAMES: &[(u64, &str)] = &[
    (RACE_BEASTWARRIOR, "Beast-Warrior"),
    (RACE_DIVINE, "Divine-Beast"),
    (RACE_SPELLCASTER, "Spellcaster"),
    (RACE_WINGEDBEAST, "Winged Beast"),
    (RACE_SEASERPENT, "Sea Serpent"),
    (RACE_CREATORGOD, "Creator God"),
    (RACE_DINOSAUR, "Dinosaur"),
    (RACE_ILLUSION, "Illusion"),
    (RACE_MACHINE, "Machine"),
    (RACE_THUNDER, "Thunder"),
    (RACE_REPTILE, "Reptile"),
    (RACE_PSYCHIC, "Psychic"),
    (RACE_CYBERSE, "Cyberse"),
    (RACE_WARRIOR, "Warrior"),
    (RACE_ZOMBIE, "Zombie"),
    (RACE_INSECT, "Insect"),
    (RACE_DRAGON, "Dragon"),
    (RACE_FAIRY, "Fairy"),
    (RACE_FIEND, "Fiend"),
    (RACE_PLANT, "Plant"),
    (RACE_BEAST, "Beast"),
    (RACE_AQUA, "Aqua"),
    (RACE_PYRO, "Pyro"),
    (RACE_ROCK, "Rock"),
    (RACE_FISH, "Fish"),
    (RACE_WYRM, "Wyrm"),
];

/// Default maximum copies of a card in Main + Extra + Side combined.
pub const DEFAULT_MAX_COPIES: u8 = 3;
