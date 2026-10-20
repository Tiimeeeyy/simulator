use std::path::PathBuf;
use std::sync::Arc;

use ygopro_core::{CoreMessage, Duel, DuelConfig, IdleCmdChoice, PlayerResponse, ProcessResult, ScriptLoader};
use ygopro_data::CardDatabase;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn test_duel_lifecycle() {
    let root = repo_root();
    let cdb_path = root.join("data").join("cards.cdb");
    let scripts_dir = root.join("data").join("scripts");

    assert!(cdb_path.exists(), "cards.cdb not found at {:?}", cdb_path);
    assert!(scripts_dir.exists(), "scripts not found at {:?}", scripts_dir);

    let db = Arc::new(CardDatabase::open(&cdb_path).expect("failed to load cdb"));
    let script_loader = Arc::new(ScriptLoader::new().with_dir(&scripts_dir));

    let config = DuelConfig::default();
    let mut duel = Duel::create(config, Arc::clone(&db), Arc::clone(&script_loader))
        .expect("failed to create duel");

    // Load built-in Ishizu Tearlaments deck for both players
    let deck = ygopro_data::builtin::tearlaments_ishizu_deck();
    duel.load_deck(0, &deck);
    duel.load_deck(1, &deck);

    // Verify deck counts in engine before start
    assert_eq!(duel.query_count(0, ocgcore_sys::LOCATION_DECK), 40);
    assert_eq!(duel.query_count(0, ocgcore_sys::LOCATION_EXTRA), 15);
    assert_eq!(duel.query_count(1, ocgcore_sys::LOCATION_DECK), 40);
    assert_eq!(duel.query_count(1, ocgcore_sys::LOCATION_EXTRA), 15);

    // Start duel
    duel.start();

    // Step engine until it awaits player response or ends
    let mut all_msgs = Vec::new();
    let mut res = duel.process().expect("process failed");
    all_msgs.extend(duel.get_messages().expect("failed to parse messages"));
    while res == ProcessResult::Continue {
        res = duel.process().expect("process failed");
        all_msgs.extend(duel.get_messages().expect("failed to parse messages"));
    }
    assert_eq!(res, ProcessResult::Awaiting);

    let msgs = all_msgs;
    println!("Total messages received: {}", msgs.len());
    for (i, m) in msgs.iter().enumerate() {
        println!("Msg #{i}: {m:?}");
    }
    assert!(!msgs.is_empty(), "expected messages upon starting duel");

    // Verify draw message was received
    let has_draw = msgs.iter().any(|m| matches!(m, CoreMessage::Draw { .. }));
    assert!(has_draw, "expected MSG_DRAW");

    // Verify turn 1 was reached
    let has_turn1 = msgs.iter().any(|m| matches!(m, CoreMessage::NewTurn { player: 0, .. }));
    assert!(has_turn1, "expected MSG_NEW_TURN for player 0");

    // Verify player 0 received IdleCmd prompt
    let prompt = msgs.iter().find(|m| matches!(m, CoreMessage::SelectIdleCmd { .. }));
    assert!(prompt.is_some(), "expected SelectIdleCmd prompt for turn player");

    // Player 0 responds: pass to End Phase
    duel.set_response(&PlayerResponse::IdleCmd(IdleCmdChoice::ToEndPhase));

    let mut msgs2 = Vec::new();
    let mut res2 = duel.process().expect("process after pass failed");
    msgs2.extend(duel.get_messages().expect("failed to parse messages"));
    while res2 == ProcessResult::Continue {
        res2 = duel.process().expect("process failed");
        msgs2.extend(duel.get_messages().expect("failed to parse messages"));
    }
    assert_eq!(res2, ProcessResult::Awaiting);

    println!("msgs2 count: {}", msgs2.len());
    for (i, m) in msgs2.iter().enumerate() {
        println!("msgs2 #{i}: {m:?}");
    }

    // Opponent is prompted for priority (SelectChain with empty/passable chains)
    let p1_chain = msgs2.iter().find(|m| matches!(m, CoreMessage::SelectChain { player: 1, .. }));
    assert!(p1_chain.is_some(), "expected SelectChain priority check for player 1");

    // Player 1 passes chain priority
    duel.set_response(&PlayerResponse::Chain(None));

    let mut msgs3 = Vec::new();
    let mut res3 = duel.process().expect("process after p1 pass failed");
    msgs3.extend(duel.get_messages().expect("failed to parse messages"));
    while res3 == ProcessResult::Continue {
        res3 = duel.process().expect("process failed");
        msgs3.extend(duel.get_messages().expect("failed to parse messages"));
    }
    println!("msgs3 count: {}", msgs3.len());
    for (i, m) in msgs3.iter().enumerate() {
        println!("msgs3 #{i}: {m:?}");
    }

    // Verify turn 2 was reached for player 1
    let has_turn2 = msgs3.iter().any(|m| matches!(m, CoreMessage::NewTurn { player: 1, .. }));
    assert!(has_turn2, "expected MSG_NEW_TURN for player 1");
}
