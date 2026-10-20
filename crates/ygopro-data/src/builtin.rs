//! Built-in deck fixtures and constants.
//!
//! Includes the canonical 70-card **Ishizu Tearlaments** deck (Spring 2023 TCG post-PHHY format)
//! configured as 40 Main / 15 Extra / 15 Side.

use crate::deck::Deck;

/// The canonical 70-card Ishizu Tearlaments decklist in plain text format,
/// deliberately dumped under a single header to exercise the smart importer
/// and auto-partitioning.
pub const TEARLAMENTS_ISHIZU_PLAINTEXT: &str = "\
== MONSTER CARDS (70 cards) ==
3x Tearlaments Reinoheart
3x Tearlaments Kashtira
1x Tearlaments Scheiren
1x Tearlaments Havnis
1x Tearlaments Merrli
1x Kelbek the Ancient Vanguard
1x Agido the Ancient Sentinel
1x Keldo the Sacred Protector
1x Mudora the Sword Oracle
2x Kashtira Fenrir
3x Diviner of the Herald
2x King of the Swamp
1x Beastking of the Swamps
1x Shaddoll Beast
3x Primeval Planet Perlereino
3x Tearlaments Scream
1x Tearlaments Grief
1x Foolish Burial
1x Foolish Burial Goods
3x Super Polymerization
1x Called by the Grave
2x Tearlaments Sulliek
1x Tearlaments Cryme
1x Tearlaments Metanoise
1x Trivikarma
1x Tearlaments Rulkallos
1x Tearlaments Kaleido-Heart
1x Predaplant Dragostapelia
1x Grapha, Dragon Overlord of Dark World
1x El Shaddoll Winda
1x Garura, Wings of Resonant Life
1x Mudragon of the Swamp
1x Baronne de Fleur
1x Time Thief Redoer
1x Bahamut Shark
1x Toadally Awesome
1x Mecha Phantom Beast Dracossack
1x Spright Sprind
1x Cross-Sheep
1x Cherubini, Ebon Angel of the Burning Abyss
2x Lava Golem
3x Droll & Lock Bird
2x Bystial Druiswurm
3x Evenly Matched
2x There Can Be Only One
2x Forbidden Droplet
1x Volcanic Scattershot
";

/// Normalized built-in Ishizu Tearlaments deck (40 Main / 15 Extra / 15 Side).
pub fn tearlaments_ishizu_deck() -> Deck {
    Deck {
        name: "Ishizu Tearlaments (2023 TCG)".into(),
        main: vec![
            // Monsters (22)
            73956664, 73956664, 73956664, // 3x Tearlaments Reinoheart
            4928565, 4928565, 4928565,    // 3x Tearlaments Kashtira
            572850,                       // 1x Tearlaments Scheiren
            37961969,                     // 1x Tearlaments Havnis
            74078255,                     // 1x Tearlaments Merrli
            25926710,                     // 1x Kelbek the Ancient Vanguard
            62320425,                     // 1x Agido the Ancient Sentinel
            63542003,                     // 1x Keldo the Sacred Protector
            99937011,                     // 1x Mudora the Sword Oracle
            32909498, 32909498,           // 2x Kashtira Fenrir
            92919429, 92919429, 92919429, // 3x Diviner of the Herald
            79109599, 79109599,           // 2x King of the Swamp
            99426834,                     // 1x Beastking of the Swamps
            3717252,                      // 1x Shaddoll Beast
            // Spells (13)
            77103950, 77103950, 77103950, // 3x Primeval Planet Perlereino
            6767771, 6767771, 6767771,    // 3x Tearlaments Scream
            33878367,                     // 1x Tearlaments Grief
            81439173,                     // 1x Foolish Burial
            35726888,                     // 1x Foolish Burial Goods
            48130397, 48130397, 48130397, // 3x Super Polymerization
            24224830,                     // 1x Called by the Grave
            // Traps (5)
            74920585, 74920585,           // 2x Tearlaments Sulliek
            1329620,                      // 1x Tearlaments Cryme
            38436986,                     // 1x Tearlaments Metanoise
            7436169,                      // 1x Trivikarma
        ],
        extra: vec![
            // Fusions (7)
            84330567, // Tearlaments Rulkallos
            28226490, // Tearlaments Kaleido-Heart
            69946549, // Predaplant Dragostapelia
            39552584, // Grapha, Dragon Overlord of Dark World
            94977269, // El Shaddoll Winda
            11765832, // Garura, Wings of Resonant Life
            54757758, // Mudragon of the Swamp
            // Synchro (1)
            84815190, // Baronne de Fleur
            // Xyz (4)
            55285840, // Time Thief Redoer
            440556,   // Bahamut Shark
            90809975, // Toadally Awesome
            22110647, // Mecha Phantom Beast Dracossack
            // Link (3)
            72329844, // Spright Sprind
            50277355, // Cross-Sheep
            58699500, // Cherubini, Ebon Angel of the Burning Abyss
        ],
        side: vec![
            102380, 102380,               // 2x Lava Golem
            94145021, 94145021, 94145021, // 3x Droll & Lock Bird
            6637331, 6637331,             // 2x Bystial Druiswurm
            15693423, 15693423, 15693423, // 3x Evenly Matched
            24207889, 24207889,           // 2x There Can Be Only One
            24299458, 24299458,           // 2x Forbidden Droplet
            69750546,                     // 1x Volcanic Scattershot
        ],
    }
}
