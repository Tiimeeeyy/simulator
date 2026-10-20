use std::path::PathBuf;

use chrono::NaiveDate;
use ygopro_data::builtin::{TEARLAMENTS_ISHIZU_PLAINTEXT, tearlaments_ishizu_deck};
use ygopro_data::{
    BanlistQuery, BanlistRegistry, CardDatabase, Deck, DeckValidator,
    ReleaseDateIndex, Violation, parse_plaintext_deck,
};

fn project_data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

#[test]
fn test_tearlaments_70_card_split_and_banlist_validation() {
    let data_dir = project_data_path();
    let cdb_path = data_dir.join("cards.cdb");
    let lflists_dir = data_dir.join("lflists");
    let rel_path = data_dir.join("release_dates.csv");

    assert!(cdb_path.exists(), "cards.cdb must exist at {}", cdb_path.display());
    assert!(lflists_dir.exists(), "lflists must exist at {}", lflists_dir.display());

    let db = CardDatabase::open(&cdb_path).expect("failed to open cards.cdb");
    let reg = BanlistRegistry::load_dir(&lflists_dir).expect("failed to load banlists");
    let rel_idx = ReleaseDateIndex::load_file(&rel_path).expect("failed to load release dates");

    // 1. Parse the 70-card plain-text list where all cards were dumped under a single header
    let parsed_deck = parse_plaintext_deck(TEARLAMENTS_ISHIZU_PLAINTEXT, &db)
        .expect("smart plain-text parser failed to parse 70-card list");

    // 2. Validate exact 40 Main / 15 Extra / 15 Side split
    assert_eq!(parsed_deck.main.len(), 40, "Main Deck must have 40 cards");
    assert_eq!(parsed_deck.extra.len(), 15, "Extra Deck must have 15 cards");
    assert_eq!(parsed_deck.side.len(), 15, "Side Deck must have 15 cards");
    assert_eq!(parsed_deck.total_count(), 70, "Total cards must be 70");

    let canonical = tearlaments_ishizu_deck();
    assert_eq!(parsed_deck.main, canonical.main, "Main deck cards mismatch");
    assert_eq!(parsed_deck.extra, canonical.extra, "Extra deck cards mismatch");
    assert_eq!(parsed_deck.side, canonical.side, "Side deck cards mismatch");

    // 3. Test round-trip serialization to .ydk and ydke://
    let ydk_str = parsed_deck.to_ydk();
    let from_ydk = Deck::parse_ydk(&ydk_str).expect("failed to re-parse YDK");
    assert_eq!(from_ydk.main, parsed_deck.main);
    assert_eq!(from_ydk.extra, parsed_deck.extra);
    assert_eq!(from_ydk.side, parsed_deck.side);

    let ydke_url = parsed_deck.to_ydke();
    assert!(ydke_url.starts_with("ydke://"));
    let from_ydke = Deck::from_ydke(&ydke_url).expect("failed to re-parse YDKE");
    assert_eq!(from_ydke.main, parsed_deck.main);
    assert_eq!(from_ydke.extra, parsed_deck.extra);
    assert_eq!(from_ydke.side, parsed_deck.side);

    let validator = DeckValidator::new(&db).with_release_dates(&rel_idx);

    // 4. Validate against 2023-03-01 TCG Banlist (Post-PHHY Kitkallos Banned Format)
    let date_2023 = NaiveDate::from_ymd_opt(2023, 3, 1).unwrap();
    let query_2023 = BanlistQuery::tcg(date_2023).with_release_dates(true);
    let active_2023 = reg.resolve(&query_2023).expect("resolve 2023 banlist");

    assert_eq!(active_2023.name(), "2023.02 TCG");
    let report_2023 = validator.validate(&parsed_deck, &active_2023);
    if !report_2023.is_legal {
        for v in &report_2023.violations {
            eprintln!("2023 violation: {v}");
        }
    }
    assert!(
        report_2023.is_legal,
        "The Ishizu Tearlaments deck must be completely legal in the 2023-03-01 TCG format"
    );
    assert!(report_2023.violations.is_empty());

    // 5. Validate against 2026-10-20 TCG Banlist (Current Format)
    let date_2026 = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();
    let query_2026 = BanlistQuery::tcg(date_2026).with_release_dates(false);
    let active_2026 = reg.resolve(&query_2026).expect("resolve 2026 banlist");

    assert_eq!(active_2026.name(), "2026.10 TCG");
    let report_2026 = validator.validate(&parsed_deck, &active_2026);
    assert!(
        !report_2026.is_legal,
        "The 2023 Ishizu Tearlaments deck must be ILLEGAL under 2026.10 TCG"
    );

    // Check expected Forbidden and Limited cards under 2026.10 TCG
    let forbidden_names: Vec<&str> = report_2026
        .violations
        .iter()
        .filter_map(|v| match v {
            Violation::ForbiddenInBanlist { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    assert!(forbidden_names.iter().any(|n| n.contains("Agido")), "Agido must be Forbidden in 2026.10");
    assert!(forbidden_names.iter().any(|n| n.contains("Kelbek")), "Kelbek must be Forbidden in 2026.10");
    assert!(forbidden_names.iter().any(|n| n.contains("Baronne")), "Baronne must be Forbidden in 2026.10");
    assert!(forbidden_names.iter().any(|n| n.contains("Bahamut Shark")), "Bahamut Shark must be Forbidden in 2026.10");
    assert!(forbidden_names.iter().any(|n| n.contains("Called by the Grave")), "Called by the Grave must be Forbidden in 2026.10");
    assert!(forbidden_names.iter().any(|n| n.contains("Cross-Sheep")), "Cross-Sheep must be Forbidden in 2026.10");
    assert!(forbidden_names.iter().any(|n| n.contains("There Can Be Only One")), "TCBOO must be Forbidden in 2026.10");

    let exceeds_limits: Vec<(&str, usize, u8)> = report_2026
        .violations
        .iter()
        .filter_map(|v| match v {
            Violation::ExceedsLimit { name, count, limit, .. } => Some((name.as_str(), *count, *limit)),
            _ => None,
        })
        .collect();

    assert!(
        exceeds_limits.iter().any(|(n, c, l)| n.contains("Droll & Lock Bird") && *c == 3 && *l == 1),
        "Droll & Lock Bird must exceed limit (3 vs 1) in 2026.10 TCG"
    );
    assert!(
        exceeds_limits.iter().any(|(n, c, l)| n.contains("Bystial Druiswurm") && *c == 2 && *l == 1),
        "Bystial Druiswurm must exceed limit (2 vs 1) in 2026.10 TCG"
    );
}
